//! NTFS MFT 大文件扫描器：直接读取 $MFT 实现快速文件检索

use serde::Serialize;
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::os::windows::fs::OpenOptionsExt;
use std::path::{Component, Path, PathBuf};

const FILE_SHARE_ALL: u32 = 0x7;
const ROOT_RECORD: usize = 5;
const INVALID_INDEX: usize = usize::MAX;

const ATTR_FILE_NAME: u32 = 0x30;
const ATTR_DATA: u32 = 0x80;
const ATTR_REPARSE_POINT: u32 = 0xC0;
const ATTR_END: u32 = 0xFFFF_FFFF;

const FILE_RECORD_IN_USE: u16 = 0x0001;
const FILE_RECORD_DIRECTORY: u16 = 0x0002;
const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;

const CHUNK_RECORDS: usize = 16 * 1024;
const READ_CHUNK_SIZE: u64 = 1 << 20;

#[derive(Debug, Clone, Serialize)]
pub struct LargeFile {
    pub path: String,
    pub size: u64,
    pub name: String,
}

#[derive(Debug, Clone)]
struct FastRecord {
    active: bool,
    parent: u64,
    base_ref: u64,
    name: String,
    is_dir: bool,
    is_reparse: bool,
    size: u64,
}

impl Default for FastRecord {
    fn default() -> Self {
        Self {
            active: false,
            parent: 0,
            base_ref: 0,
            name: String::new(),
            is_dir: false,
            is_reparse: false,
            size: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct NtfsLayout {
    bytes_per_sector: usize,
    cluster_size: u64,
    record_size: usize,
}

#[derive(Debug, Clone, Copy)]
struct DataRun {
    lcn: i64,
    clusters: u64,
}

/// Scan for large files using NTFS MFT
pub fn scan_large_files(
    root: &str,
    min_bytes: u64,
    limit: u32,
    skip_optional: bool,
    excludes: &[String],
) -> Result<Vec<LargeFile>, String> {
    let drive = drive_letter(root).ok_or_else(|| {
        format!("MFT 扫描仅支持驱动器根目录，收到: {}", root)
    })?;

    let drive = drive.to_ascii_uppercase();
    let drive_root = PathBuf::from(format!("{}:\\", drive));
    let exclude_paths: Vec<PathBuf> = excludes.iter().map(PathBuf::from).collect();

    let volume = open_volume(drive).map_err(|e| format!("打开卷失败: {}", e))?;
    let mut reader = SectorReader::new(volume);

    let (layout, mft_runs, mft_size) =
        load_layout_and_mft_runs(&mut reader).map_err(|e| format!("读取MFT布局失败: {}", e))?;

    let entry_count = (mft_size as usize + layout.record_size - 1) / layout.record_size;
    if entry_count <= ROOT_RECORD {
        return Ok(Vec::new());
    }

    let mut records = vec![FastRecord::default(); entry_count];
    read_records_from_runs(&mut reader, &mft_runs, layout, &mut records)
        .map_err(|e| format!("读取MFT记录失败: {}", e))?;
    merge_extension_records(&mut records);

    if ROOT_RECORD < records.len() {
        records[ROOT_RECORD].active = true;
        records[ROOT_RECORD].is_dir = true;
        records[ROOT_RECORD].parent = ROOT_RECORD as u64;
    }

    Ok(build_large_file_entries(
        &records,
        &drive_root,
        min_bytes,
        limit as usize,
        skip_optional,
        &exclude_paths,
    ))
}

fn drive_letter(path_str: &str) -> Option<char> {
    let raw = path_str.replace('/', "\\");
    let mut chars = raw.chars();
    let drive = chars.next()?;
    if !drive.is_ascii_alphabetic() || chars.next()? != ':' {
        return None;
    }
    let rest: String = chars.collect();
    if rest.is_empty() || rest.chars().all(|c| c == '\\') {
        Some(drive)
    } else {
        None
    }
}

fn open_volume(drive: char) -> io::Result<File> {
    let volume_path = format!("\\\\.\\{}:", drive);
    File::options()
        .read(true)
        .share_mode(FILE_SHARE_ALL)
        .open(volume_path)
}

fn load_layout_and_mft_runs<R: Read + Seek>(
    reader: &mut R,
) -> io::Result<(NtfsLayout, Vec<DataRun>, u64)> {
    let mut boot = [0u8; 512];
    reader.seek(SeekFrom::Start(0))?;
    reader.read_exact(&mut boot)?;

    if &boot[3..11] != b"NTFS    " {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "卷不是NTFS格式"));
    }

    let bytes_per_sector = read_u16(&boot, 11) as usize;
    let sectors_per_cluster = boot[13] as u64;
    if bytes_per_sector == 0 || sectors_per_cluster == 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "无效的NTFS引导扇区"));
    }

    let cluster_size = bytes_per_sector as u64 * sectors_per_cluster;
    let clusters_per_record = boot[64] as i8;
    let record_size = if clusters_per_record < 0 {
        let exponent = (-i32::from(clusters_per_record)) as u32;
        1usize.checked_shl(exponent).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "无效的MFT记录大小")
        })?
    } else {
        (cluster_size * clusters_per_record as u64) as usize
    };

    if record_size == 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "无效的MFT记录大小"));
    }

    let mft_lcn = read_i64(&boot, 48);
    if mft_lcn < 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "无效的MFT LCN"));
    }

    let layout = NtfsLayout {
        bytes_per_sector,
        cluster_size,
        record_size,
    };

    let mut mft_record = vec![0u8; record_size];
    let mft_offset = mft_lcn as u64 * cluster_size;
    reader.seek(SeekFrom::Start(mft_offset))?;
    reader.read_exact(&mut mft_record)?;
    apply_fixup(&mut mft_record, bytes_per_sector)?;
    let (runs, mft_size) = parse_mft_data_runs(&mft_record)?;
    Ok((layout, runs, mft_size))
}

fn read_records_from_runs<R: Read + Seek>(
    reader: &mut R,
    runs: &[DataRun],
    layout: NtfsLayout,
    records: &mut [FastRecord],
) -> io::Result<()> {
    let mut buffer = vec![0u8; layout.record_size * CHUNK_RECORDS];
    let mut logical_offset = 0u64;

    for run in runs {
        let run_bytes = run.clusters * layout.cluster_size;
        if run.lcn < 0 {
            logical_offset += run_bytes;
            continue;
        }

        let mut run_offset = 0u64;
        while run_offset < run_bytes {
            let remaining = (run_bytes - run_offset) as usize;
            let mut read_len = remaining.min(buffer.len());
            read_len = (read_len / layout.record_size) * layout.record_size;
            if read_len == 0 {
                break;
            }

            let disk_offset = run.lcn as u64 * layout.cluster_size + run_offset;
            reader.seek(SeekFrom::Start(disk_offset))?;
            reader.read_exact(&mut buffer[..read_len])?;

            let base_record = ((logical_offset + run_offset) / layout.record_size as u64) as usize;
            for (record_offset, record_bytes) in buffer[..read_len]
                .chunks_exact(layout.record_size)
                .enumerate()
            {
                let record_num = base_record + record_offset;
                if record_num >= records.len() {
                    break;
                }

                let mut temp = record_bytes.to_vec();
                if let Some(record) =
                    parse_file_record(record_num as u64, &mut temp, layout.bytes_per_sector)
                {
                    records[record_num] = record;
                }
            }

            run_offset += read_len as u64;
        }

        logical_offset += run_bytes;
    }

    Ok(())
}

fn parse_file_record(
    record_num: u64,
    record: &mut [u8],
    bytes_per_sector: usize,
) -> Option<FastRecord> {
    if record.len() < 64 || &record[0..4] != b"FILE" {
        return None;
    }
    if apply_fixup(record, bytes_per_sector).is_err() {
        return None;
    }

    let attr_offset = read_u16(record, 20) as usize;
    let flags = read_u16(record, 22);
    if flags & FILE_RECORD_IN_USE == 0 {
        return None;
    }
    let base_ref = read_u64(record, 32) & 0x0000_FFFF_FFFF_FFFF;

    let mut offset = attr_offset;
    let mut best_name: Option<(u8, u64, String, u32)> = None;
    let mut size = 0u64;
    let mut is_reparse = false;

    while offset + 16 <= record.len() {
        let attr_type = read_u32(record, offset);
        if attr_type == ATTR_END {
            break;
        }
        let attr_len = read_u32(record, offset + 4) as usize;
        if attr_len < 16 || offset + attr_len > record.len() {
            break;
        }

        let non_resident = record[offset + 8] != 0;
        let name_len = record[offset + 9];
        match attr_type {
            ATTR_FILE_NAME => {
                if !non_resident && offset + 22 <= record.len() {
                    let value_len = read_u32(record, offset + 16) as usize;
                    let value_offset = read_u16(record, offset + 20) as usize;
                    let start = offset + value_offset;
                    if start + value_len <= record.len() {
                        if let Some((score, parent, name, file_attrs)) =
                            parse_file_name_value(&record[start..start + value_len])
                        {
                            let replace = best_name
                                .as_ref()
                                .map(|(best_score, _, _, _)| score > *best_score)
                                .unwrap_or(true);
                            if replace {
                                best_name = Some((score, parent, name, file_attrs));
                            }
                        }
                    }
                }
            }
            ATTR_DATA => {
                if name_len == 0 && flags & FILE_RECORD_DIRECTORY == 0 {
                    if non_resident && offset + 56 <= record.len() {
                        size = read_u64(record, offset + 48);
                    } else if !non_resident && offset + 20 <= record.len() {
                        size = read_u32(record, offset + 16) as u64;
                    }
                }
            }
            ATTR_REPARSE_POINT => {
                is_reparse = true;
            }
            _ => {}
        }

        offset += attr_len;
    }

    let (parent, name, file_attrs) = match best_name {
        Some((_score, parent, name, file_attrs)) => (parent, name, file_attrs),
        None if record_num == ROOT_RECORD as u64 => (ROOT_RECORD as u64, String::new(), 0),
        None if base_ref != 0 && size > 0 => (INVALID_INDEX as u64, String::new(), 0),
        None => return None,
    };

    is_reparse |= file_attrs & FILE_ATTRIBUTE_REPARSE_POINT != 0;
    Some(FastRecord {
        active: true,
        parent,
        base_ref,
        name,
        is_dir: flags & FILE_RECORD_DIRECTORY != 0,
        is_reparse,
        size,
    })
}

fn merge_extension_records(records: &mut [FastRecord]) {
    let mut extensions = Vec::new();
    for (idx, record) in records.iter().enumerate() {
        if record.active && record.base_ref != 0 && record.size > 0 {
            extensions.push((idx, record.base_ref as usize, record.size));
        }
    }

    for (idx, base_idx, size) in extensions {
        if base_idx < records.len() && records[base_idx].active {
            records[base_idx].size = records[base_idx].size.max(size);
        }
        records[idx].active = false;
        records[idx].size = 0;
    }
}

fn parse_file_name_value(value: &[u8]) -> Option<(u8, u64, String, u32)> {
    if value.len() < 66 {
        return None;
    }
    let parent = read_u64(value, 0) & 0x0000_FFFF_FFFF_FFFF;
    let file_attrs = read_u32(value, 56);
    let name_len = value[64] as usize;
    let namespace = value[65];
    let name_start = 66usize;
    let name_bytes = name_len.checked_mul(2)?;
    if name_start + name_bytes > value.len() {
        return None;
    }

    let mut chars = Vec::with_capacity(name_len);
    for chunk in value[name_start..name_start + name_bytes].chunks_exact(2) {
        chars.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    let name = String::from_utf16_lossy(&chars);
    let score = match namespace {
        1 | 3 => 3,
        0 => 2,
        _ => 1,
    };
    Some((score, parent, name, file_attrs))
}

fn parse_mft_data_runs(record: &[u8]) -> io::Result<(Vec<DataRun>, u64)> {
    let attr_offset = read_u16(record, 20) as usize;
    let mut offset = attr_offset;

    while offset + 64 <= record.len() {
        let attr_type = read_u32(record, offset);
        if attr_type == ATTR_END {
            break;
        }
        let attr_len = read_u32(record, offset + 4) as usize;
        if attr_len < 16 || offset + attr_len > record.len() {
            break;
        }
        let non_resident = record[offset + 8] != 0;
        let name_len = record[offset + 9];
        if attr_type == ATTR_DATA && non_resident && name_len == 0 {
            let run_offset = read_u16(record, offset + 32) as usize;
            let real_size = read_u64(record, offset + 48);
            let runs = parse_runlist(&record[offset + run_offset..offset + attr_len]);
            return Ok((runs, real_size));
        }
        offset += attr_len;
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "未找到$MFT数据属性",
    ))
}

fn parse_runlist(data: &[u8]) -> Vec<DataRun> {
    let mut runs = Vec::new();
    let mut idx = 0usize;
    let mut current_lcn = 0i64;

    while idx < data.len() {
        let header = data[idx];
        idx += 1;
        if header == 0 {
            break;
        }
        let len_size = (header & 0x0F) as usize;
        let off_size = (header >> 4) as usize;
        if len_size == 0 || idx + len_size + off_size > data.len() {
            break;
        }

        let clusters = read_uint_var(&data[idx..idx + len_size]);
        idx += len_size;
        let lcn_delta = read_int_var(&data[idx..idx + off_size]);
        idx += off_size;
        current_lcn += lcn_delta;
        runs.push(DataRun {
            lcn: current_lcn,
            clusters,
        });
    }

    runs
}

fn apply_fixup(record: &mut [u8], bytes_per_sector: usize) -> io::Result<()> {
    if record.len() < 8 || bytes_per_sector == 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "无效的文件记录"));
    }
    let usa_offset = read_u16(record, 4) as usize;
    let usa_count = read_u16(record, 6) as usize;
    if usa_count == 0 || usa_offset + usa_count * 2 > record.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "无效的更新序列数组",
        ));
    }

    for i in 1..usa_count {
        let sector_tail = i * bytes_per_sector - 2;
        if sector_tail + 2 > record.len() {
            break;
        }
        let replacement = usa_offset + i * 2;
        let replacement_bytes = [record[replacement], record[replacement + 1]];
        record[sector_tail..sector_tail + 2].copy_from_slice(&replacement_bytes);
    }
    Ok(())
}

fn build_large_file_entries(
    records: &[FastRecord],
    drive_root: &Path,
    min_bytes: u64,
    limit: usize,
    skip_optional: bool,
    exclude_prefixes: &[PathBuf],
) -> Vec<LargeFile> {
    let mut refs: Vec<(usize, u64)> = records
        .iter()
        .enumerate()
        .filter_map(|(idx, rec)| {
            if rec.active && !rec.is_dir && !rec.is_reparse && rec.size >= min_bytes {
                Some((idx, rec.size))
            } else {
                None
            }
        })
        .collect();
    refs.sort_by(|a, b| b.1.cmp(&a.1));

    let mut rows = Vec::new();
    let mut seen = HashSet::new();
    for (idx, size) in refs {
        if limit > 0 && rows.len() >= limit {
            break;
        }
        let Some(path) = build_path(records, idx, drive_root) else {
            continue;
        };
        if is_ntfs_metadata_or_protected_path(&path, drive_root) {
            continue;
        }
        if should_skip_large_file_path(&path, skip_optional) {
            continue;
        }
        if is_excluded_large_file_path(&path, exclude_prefixes) {
            continue;
        }

        let key = normalize_path(&path);
        if seen.insert(key) {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            rows.push(LargeFile {
                path: path.to_string_lossy().to_string(),
                size,
                name,
            });
        }
    }

    rows
}

fn build_path(records: &[FastRecord], idx: usize, drive_root: &Path) -> Option<PathBuf> {
    let mut current = idx;
    let mut segments = Vec::new();
    let mut guard = 0usize;

    while current != ROOT_RECORD {
        guard += 1;
        if guard > records.len() {
            return None;
        }
        let rec = records.get(current)?;
        if !rec.active || rec.name.is_empty() {
            return None;
        }
        segments.push(rec.name.clone());
        let parent = rec.parent as usize;
        if parent >= records.len() {
            return None;
        }
        current = parent;
    }

    segments.reverse();
    let mut path = drive_root.to_path_buf();
    for segment in segments {
        path.push(segment);
    }
    Some(path)
}

fn should_skip_large_file_path(path: &Path, skip_optional: bool) -> bool {
    let Some(name) = path
        .file_name()
        .map(|name| name.to_string_lossy().to_ascii_lowercase())
    else {
        return false;
    };
    let ext = path
        .extension()
        .map(|ext| format!(".{}", ext.to_string_lossy().to_ascii_lowercase()))
        .unwrap_or_default();

    if ext == ".sys" {
        return true;
    }
    if !skip_optional {
        return false;
    }
    if matches!(
        name.as_str(),
        "pagefile.sys" | "hiberfil.sys" | "swapfile.sys" | "memory.dmp"
    ) {
        return true;
    }
    matches!(
        ext.as_str(),
        ".vhd" | ".vhdx" | ".avhd" | ".avhdx" | ".vmdk" | ".vdi" | ".qcow" | ".qcow2" | ".ova"
            | ".ovf"
    )
}

fn is_ntfs_metadata_or_protected_path(path: &Path, drive_root: &Path) -> bool {
    let components = relative_components(path, drive_root);
    if components.is_empty() {
        return false;
    }
    if components[0].starts_with('$') {
        return true;
    }

    if components.len() == 1
        && matches!(
            components[0].as_str(),
            "PAGEFILE.SYS" | "SWAPFILE.SYS" | "HIBERFIL.SYS" | "DUMPSTACK.LOG" | "DUMPSTACK.LOG.TMP"
        )
    {
        return true;
    }

    const PREFIXES: &[&[&str]] = &[
        &["SYSTEM VOLUME INFORMATION"],
        &["RECOVERY"],
        &["$RECYCLE.BIN"],
        &["WINDOWS", "SYSTEM32", "CONFIG"],
        &["WINDOWS", "SYSTEM32", "TASKS"],
        &["WINDOWS", "SYSTEM32", "SRU"],
        &["WINDOWS", "SYSTEM32", "WDI"],
        &["WINDOWS", "SYSTEM32", "WBEM", "MOF"],
        &["WINDOWS", "SYSWOW64", "CONFIG"],
        &["WINDOWS", "SYSWOW64", "TASKS"],
        &["WINDOWS", "SYSWOW64", "SRU"],
        &["WINDOWS", "SYSWOW64", "NETWORKLIST"],
        &["WINDOWS", "SYSWOW64", "MSDTC"],
        &["WINDOWS", "SYSWOW64", "INETSRV", "CONFIG"],
        &["WINDOWS", "SYSWOW64", "CONFIGURATION"],
        &["WINDOWS", "SYSWOW64", "COM", "DMP"],
        &["WINDOWS", "SYSTEM", "WIM"],
    ];

    PREFIXES.iter().any(|prefix| {
        components.len() >= prefix.len()
            && prefix
                .iter()
                .zip(components.iter())
                .all(|(expected, actual)| expected == actual)
    })
}

fn relative_components(path: &Path, drive_root: &Path) -> Vec<String> {
    let Ok(relative) = path.strip_prefix(drive_root) else {
        return Vec::new();
    };
    relative
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().to_ascii_uppercase()),
            _ => None,
        })
        .collect()
}

fn is_excluded_large_file_path(path: &Path, exclude_prefixes: &[PathBuf]) -> bool {
    let path_norm = normalize_path(path);
    exclude_prefixes.iter().any(|prefix| {
        let prefix_norm = normalize_path(prefix);
        if prefix_norm.is_empty() {
            return false;
        }
        path_norm == prefix_norm
            || path_norm
                .strip_prefix(&prefix_norm)
                .map(|rest| rest.starts_with('\\'))
                .unwrap_or(false)
    })
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_ascii_lowercase()
}

fn read_u16(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

fn read_u64(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ])
}

fn read_i64(data: &[u8], offset: usize) -> i64 {
    i64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ])
}

fn read_uint_var(data: &[u8]) -> u64 {
    let mut value = 0u64;
    for (i, byte) in data.iter().enumerate() {
        value |= (*byte as u64) << (i * 8);
    }
    value
}

fn read_int_var(data: &[u8]) -> i64 {
    if data.is_empty() {
        return 0;
    }
    let mut value = 0i64;
    for (i, byte) in data.iter().enumerate() {
        value |= (*byte as i64) << (i * 8);
    }
    if data.last().copied().unwrap_or(0) & 0x80 != 0 {
        value |= -1i64 << (data.len() * 8);
    }
    value
}

struct SectorReader<R> {
    inner: R,
    buffer: Vec<u8>,
    chunk_start: u64,
    buffer_len: usize,
    has_chunk: bool,
    cursor: u64,
}

impl<R: Read + Seek> SectorReader<R> {
    fn new(inner: R) -> Self {
        Self {
            inner,
            buffer: vec![0u8; READ_CHUNK_SIZE as usize],
            chunk_start: 0,
            buffer_len: 0,
            has_chunk: false,
            cursor: 0,
        }
    }

    fn chunk_for(offset: u64) -> u64 {
        (offset / READ_CHUNK_SIZE) * READ_CHUNK_SIZE
    }

    fn ensure_chunk(&mut self) -> io::Result<()> {
        let target = Self::chunk_for(self.cursor);
        if self.has_chunk && target == self.chunk_start {
            return Ok(());
        }
        self.inner.seek(SeekFrom::Start(target))?;
        self.buffer_len = 0;
        while self.buffer_len < self.buffer.len() {
            match self.inner.read(&mut self.buffer[self.buffer_len..])? {
                0 => break,
                n => self.buffer_len += n,
            }
        }
        self.chunk_start = target;
        self.has_chunk = true;
        Ok(())
    }
}

impl<R: Read + Seek> Read for SectorReader<R> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if out.is_empty() {
            return Ok(0);
        }
        self.ensure_chunk()?;
        let offset_in_chunk = (self.cursor - self.chunk_start) as usize;
        if offset_in_chunk >= self.buffer_len {
            return Ok(0);
        }
        let available = self.buffer_len - offset_in_chunk;
        let n = available.min(out.len());
        out[..n].copy_from_slice(&self.buffer[offset_in_chunk..offset_in_chunk + n]);
        self.cursor += n as u64;
        Ok(n)
    }
}

impl<R: Read + Seek> Seek for SectorReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_cursor = match pos {
            SeekFrom::Start(p) => p,
            SeekFrom::Current(d) => {
                if d < 0 {
                    self.cursor.saturating_sub((-d) as u64)
                } else {
                    self.cursor.saturating_add(d as u64)
                }
            }
            SeekFrom::End(d) => {
                let end = self.inner.seek(SeekFrom::End(0))?;
                if d < 0 {
                    end.saturating_sub((-d) as u64)
                } else {
                    end.saturating_add(d as u64)
                }
            }
        };
        self.cursor = new_cursor;
        Ok(new_cursor)
    }
}

/// 基于文件系统遍历的备用大文件扫描（无需管理员权限）
pub fn scan_large_files_fallback(
    root: &str,
    min_bytes: u64,
    limit: usize,
    skip_optional: bool,
    excludes: &[String],
) -> Result<Vec<LargeFile>, String> {
    let root_path = Path::new(root);
    if !root_path.exists() {
        return Err(format!("路径不存在: {}", root));
    }

    let exclude_paths: Vec<PathBuf> = excludes.iter().map(PathBuf::from).collect();
    let mut results: Vec<LargeFile> = Vec::new();

    fn walk_dir(
        dir: &Path,
        min_bytes: u64,
        skip_optional: bool,
        exclude_prefixes: &[PathBuf],
        results: &mut Vec<LargeFile>,
        limit: usize,
    ) -> io::Result<()> {
        if results.len() >= limit {
            return Ok(());
        }

        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return Ok(()), // 跳过无权限目录
        };

        for entry in entries {
            if results.len() >= limit {
                return Ok(());
            }
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();

            if is_excluded_large_file_path(&path, exclude_prefixes) {
                continue;
            }

            if is_ntfs_metadata_or_protected_path(&path, &PathBuf::from("C:\\")) {
                continue;
            }

            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            if metadata.is_file() {
                let size = metadata.len();
                if size >= min_bytes && !should_skip_large_file_path(&path, skip_optional) {
                    results.push(LargeFile {
                        path: normalize_path(&path),
                        size,
                        name: path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default(),
                    });
                }
            } else if metadata.is_dir() {
                if metadata.file_type().is_symlink() {
                    continue;
                }
                let _ = walk_dir(&path, min_bytes, skip_optional, exclude_prefixes, results, limit);
            }
        }
        Ok(())
    }

    walk_dir(
        root_path,
        min_bytes,
        skip_optional,
        &exclude_paths,
        &mut results,
        limit,
    )
    .map_err(|e| format!("扫描失败: {}", e))?;

    results.sort_by(|a, b| b.size.cmp(&a.size));
    results.truncate(limit);

    Ok(results)
}
