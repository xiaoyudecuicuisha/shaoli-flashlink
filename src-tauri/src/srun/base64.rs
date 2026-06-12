//! 深澜自定义 Base64 编码（非标准字母表，禁止替换为标准库实现）

/// 深澜自定义 Base64 编码
/// 使用特定字母表: LVoJPiCN2R8G90yg+hmFHuacZ1OWMnrsSTXkYpUq/3dlbfKwv6xztjI7DeBE45QA
const SRUN_ALPHABET: &[u8; 64] = b"LVoJPiCN2R8G90yg+hmFHuacZ1OWMnrsSTXkYpUq/3dlbfKwv6xztjI7DeBE45QA";
const PADCHAR: u8 = b'=';

/// 获取字节值，越界返回 0（与 Python 版 _getbyte 行为一致）
fn get_byte(s: &[u8], i: usize) -> u8 {
    if i >= s.len() {
        return 0;
    }
    s[i]
}

/// 深澜自定义 Base64 编码
pub fn srun_base64_encode(data: &[u8]) -> String {
    if data.is_empty() {
        return String::new();
    }

    let mut result = Vec::new();
    let imax = data.len() - data.len() % 3;

    let mut i = 0;
    while i < imax {
        let b10 = ((get_byte(data, i) as u32) << 16)
            | ((get_byte(data, i + 1) as u32) << 8)
            | (get_byte(data, i + 2) as u32);
        result.push(SRUN_ALPHABET[(b10 >> 18) as usize]);
        result.push(SRUN_ALPHABET[((b10 >> 12) & 63) as usize]);
        result.push(SRUN_ALPHABET[((b10 >> 6) & 63) as usize]);
        result.push(SRUN_ALPHABET[(b10 & 63) as usize]);
        i += 3;
    }

    let remainder = data.len() - imax;
    if remainder == 1 {
        let b10 = (get_byte(data, imax) as u32) << 16;
        result.push(SRUN_ALPHABET[(b10 >> 18) as usize]);
        result.push(SRUN_ALPHABET[((b10 >> 12) & 63) as usize]);
        result.push(PADCHAR);
        result.push(PADCHAR);
    } else if remainder == 2 {
        let b10 = ((get_byte(data, imax) as u32) << 16)
            | ((get_byte(data, imax + 1) as u32) << 8);
        result.push(SRUN_ALPHABET[(b10 >> 18) as usize]);
        result.push(SRUN_ALPHABET[((b10 >> 12) & 63) as usize]);
        result.push(SRUN_ALPHABET[((b10 >> 6) & 63) as usize]);
        result.push(PADCHAR);
    }

    String::from_utf8(result).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        assert_eq!(srun_base64_encode(b""), "");
    }

    #[test]
    fn test_known_values() {
        // 简单验证编码不为空且长度正确
        let encoded = srun_base64_encode(b"hello");
        assert!(!encoded.is_empty());
        // Base64 编码后长度应为 4 的倍数
        assert_eq!(encoded.len() % 4, 0);
    }
}
