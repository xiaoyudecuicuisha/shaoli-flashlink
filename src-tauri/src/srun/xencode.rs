//! XEncode 加密：深澜 XXTEA 变种（包装算术以匹配 Python 实现）

/// XEncode — 深澜 XXTEA 变种加密
///
/// ⚠️ 核心陷阱：Python 整数无限精度，Rust u32 会溢出。
/// 所有算术运算必须使用 wrapping_* 系列方法。
///
/// 将字节数组打包为 u32 数组（小端序）
/// 对应 Python 的 _sencode
fn sencode(msg: &[u8], key: bool) -> Vec<u32> {
    let l = msg.len();
    let mut pwd = Vec::new();
    let mut i = 0;
    while i < l {
        let b0 = if i < l { msg[i] as u32 } else { 0 };
        let b1 = if i + 1 < l { msg[i + 1] as u32 } else { 0 };
        let b2 = if i + 2 < l { msg[i + 2] as u32 } else { 0 };
        let b3 = if i + 3 < l { msg[i + 3] as u32 } else { 0 };
        pwd.push(b0 | (b1 << 8) | (b2 << 16) | (b3 << 24));
        i += 4;
    }
    if key {
        pwd.push(l as u32);
    }
    pwd
}

/// 将 u32 数组解包为字节数组（小端序）
/// 对应 Python 的 _lencode
fn lencode(msg: &[u32], key: bool) -> Vec<u8> {
    let l = msg.len();
    let mut ll = (l - 1) << 2;

    if key {
        let m = msg[l - 1] as usize;
        // Python: if m < ll - 3 or m > ll: return None
        // 注意 Python 的 ll = (l-1) << 2，这里已计算
        if m < ll.wrapping_sub(3) || m > ll {
            return Vec::new();
        }
        ll = m;
    }

    let mut res = Vec::with_capacity(l * 4);
    for item in msg.iter().take(l) {
        res.push((item & 0xff) as u8);
        res.push(((item >> 8) & 0xff) as u8);
        res.push(((item >> 16) & 0xff) as u8);
        res.push(((item >> 24) & 0xff) as u8);
    }

    if key {
        res.truncate(ll);
    }
    res
}

/// XEncode 核心加密函数
///
/// Python 原版关键表达式与 Rust wrapping 等价：
/// - `d = d + c & 0xFFFFFFFF`  → `d = d.wrapping_add(c)`（因为 c 已是 0x9E3779B9）
/// - `z >> 5`                  → `z.wrapping_shr(5)`
/// - `y << 2`                  → `y.wrapping_shl(2)`
/// - `pwd[p] + m & 0xFFFFFFFF` → `pwd[p].wrapping_add(m)`（Python 优先级：(add) & mask）
pub fn xencode(msg: &[u8], key: &[u8]) -> Vec<u8> {
    if msg.is_empty() {
        return Vec::new();
    }

    let mut pwd = sencode(msg, true);
    let mut pwdk = sencode(key, false);

    // 补齐到 4 个元素
    while pwdk.len() < 4 {
        pwdk.push(0);
    }

    let n = pwd.len() - 1;
    let mut z: u32 = pwd[n];
    let c: u32 = 0x9E3779B9; // 0x86014019 | 0x183639A0
    let mut q: usize = 6 + 52 / (n + 1);
    let mut d: u32 = 0;

    while q > 0 {
        // Python: d = d + c & 0xFFFFFFFF
        // 运算顺序: (d + c) & 0xFFFFFFFF = wrapping_add
        d = d.wrapping_add(c);
        let e = (d >> 2) & 3;

        let mut p: usize = 0;
        while p < n {
            let y = pwd[p + 1];
            // m = z >> 5 ^ y << 2
            let mut m = z.wrapping_shr(5) ^ y.wrapping_shl(2);
            // m = m + ((y >> 3 ^ z << 4) ^ (d ^ y))
            m = m.wrapping_add((y.wrapping_shr(3) ^ z.wrapping_shl(4)) ^ (d ^ y));
            // m = m + (pwdk[(p & 3) ^ e] ^ z)
            m = m.wrapping_add(pwdk[(p & 3) ^ e as usize] ^ z);
            // pwd[p] = pwd[p] + m & 0xFFFFFFFF
            pwd[p] = pwd[p].wrapping_add(m);
            z = pwd[p];
            p += 1;
        }

        // 最后一个元素（环绕到 pwd[0]）
        let y = pwd[0];
        let mut m = z.wrapping_shr(5) ^ y.wrapping_shl(2);
        m = m.wrapping_add((y.wrapping_shr(3) ^ z.wrapping_shl(4)) ^ (d ^ y));
        m = m.wrapping_add(pwdk[(p & 3) ^ e as usize] ^ z);
        pwd[n] = pwd[n].wrapping_add(m);
        z = pwd[n];

        q -= 1;
    }

    lencode(&pwd, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xencode_empty() {
        let result = xencode(b"", b"key");
        assert!(result.is_empty());
    }

    #[test]
    fn test_xencode_deterministic() {
        // 相同输入应产生相同输出
        let r1 = xencode(b"test_data", b"test_key");
        let r2 = xencode(b"test_data", b"test_key");
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_xencode_different_keys() {
        // 不同密钥应产生不同输出
        let r1 = xencode(b"test_data", b"key1");
        let r2 = xencode(b"test_data", b"key2");
        assert_ne!(r1, r2);
    }

    #[test]
    fn test_sencode_lencode_roundtrip() {
        let original = b"hello world 1234";
        let encoded = sencode(original, false);
        let decoded = lencode(&encoded, false);
        assert_eq!(&decoded[..original.len()], original);
    }
}
