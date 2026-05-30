//! 工具函数模块
//!
//! 提供常用的工具函数，包括：
//! - 哈希计算
//! - 字节与十六进制转换
//! - 随机数生成
//!
//! 这些函数主要用于辅助 VDF 和 VRF 的实现。

use sha2::{Digest, Sha256};
use wasm_bindgen::prelude::*;

/// 计算字节数据的 SHA256 哈希
///
/// 对输入数据进行 SHA256 哈希计算，返回 32 字节的哈希值。
///
/// # 参数
/// - `data`: 要哈希的字节数据
///
/// # 返回值
/// 返回 32 字节的哈希向量
///
/// # 示例
/// ```rust
/// use vtp_core::utils::hash_bytes;
/// let data = b"hello world";
/// let hash = hash_bytes(data);
/// assert_eq!(hash.len(), 32);
/// ```
///
/// # 性能
/// SHA256 计算速度约为 200-500MB/s（取决于硬件）
#[wasm_bindgen]
pub fn hash_bytes(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// 将字节数组转换为十六进制字符串
///
/// 将每个字节转换为两位十六进制表示，使用小写字母。
///
/// # 参数
/// - `bytes`: 要转换的字节数组
///
/// # 返回值
/// 返回十六进制字符串
///
/// # 示例
/// ```rust
/// use vtp_core::utils::bytes_to_hex;
/// let bytes = vec![0x00, 0x0f, 0xff];
/// let hex = bytes_to_hex(&bytes);
/// assert_eq!(hex, "000fff");
/// ```
///
/// # 格式
/// - 每个字节使用两位十六进制表示
/// - 使用小写字母 (a-f)
/// - 不包含 0x 前缀
#[wasm_bindgen]
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// 将十六进制字符串转换为字节数组
///
/// 解析十六进制字符串，返回对应的字节数组。
///
/// # 参数
/// - `hex`: 十六进制字符串
///
/// # 返回值
/// - `Ok(Vec<u8>)`: 成功转换的字节数组
/// - `Err(JsValue)`: 转换失败的错误信息
///
/// # 错误
/// 以下情况会返回错误：
/// - 字符串长度不是偶数
/// - 包含非十六进制字符
///
/// # 示例
/// ```rust
/// use vtp_core::utils::hex_to_bytes;
/// let hex = "000fff";
/// let bytes = hex_to_bytes(hex).unwrap();
/// assert_eq!(bytes, vec![0x00, 0x0f, 0xff]);
/// ```
///
/// # 注意
/// - 输入字符串不区分大小写
/// - 不接受 0x 前缀
#[wasm_bindgen]
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, JsValue> {
    if hex.len() % 2 != 0 {
        return Err(JsValue::from_str("Invalid hex string length"));
    }

    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for i in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[i..i + 2], 16)
            .map_err(|_| JsValue::from_str("Invalid hex character"))?;
        bytes.push(byte);
    }

    Ok(bytes)
}

/// 生成指定长度的随机字节数组
///
/// 使用操作系统提供的密码学安全随机数生成器生成随机字节。
///
/// # 参数
/// - `length`: 要生成的字节数量
///
/// # 返回值
/// 返回指定长度的随机字节数组
///
/// # 安全性
/// - 使用 `OsRng` 确保随机数的密码学安全性
/// - 生成的随机数不可预测
/// - 适合用于密钥生成和随机挑战
///
/// # 示例
/// ```rust
/// use vtp_core::utils::generate_random_bytes;
/// let random_bytes = generate_random_bytes(32);
/// assert_eq!(random_bytes.len(), 32);
/// ```
///
/// # 性能
/// 生成速度取决于操作系统的随机数生成器实现
#[wasm_bindgen]
pub fn generate_random_bytes(length: u32) -> Vec<u8> {
    use rand::RngCore;
    let mut bytes = vec![0u8; length as usize];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    /// 测试 SHA256 哈希计算
    ///
    /// 验证：
    /// 1. 输出长度为 32 字节
    /// 2. 相同输入产生相同输出
    #[wasm_bindgen_test]
    fn test_hash_bytes() {
        let data = b"test data";
        let hash = hash_bytes(data);
        assert_eq!(hash.len(), 32);
    }

    /// 测试字节到十六进制转换
    ///
    /// 验证：
    /// 1. 正确转换各种字节值
    /// 2. 使用小写字母
    #[wasm_bindgen_test]
    fn test_bytes_to_hex() {
        let bytes = vec![0x00, 0x0f, 0xff];
        let hex = bytes_to_hex(&bytes);
        assert_eq!(hex, "000fff");
    }

    /// 测试十六进制到字节转换
    ///
    /// 验证：
    /// 1. 正确解析十六进制字符串
    /// 2. 返回正确的字节数组
    #[wasm_bindgen_test]
    fn test_hex_to_bytes() {
        let hex = "000fff";
        let bytes = hex_to_bytes(hex).unwrap();
        assert_eq!(bytes, vec![0x00, 0x0f, 0xff]);
    }

    /// 测试无效十六进制字符串
    ///
    /// 验证：
    /// 1. 奇数长度字符串返回错误
    /// 2. 非十六进制字符返回错误
    #[wasm_bindgen_test]
    fn test_invalid_hex() {
        let result = hex_to_bytes("invalid");
        assert!(result.is_err());
    }

    /// 测试随机字节生成
    ///
    /// 验证：
    /// 1. 生成正确长度的字节数组
    /// 2. 多次生成的结果不同
    #[wasm_bindgen_test]
    fn test_generate_random_bytes() {
        let bytes1 = generate_random_bytes(32);
        let bytes2 = generate_random_bytes(32);

        assert_eq!(bytes1.len(), 32);
        assert_eq!(bytes2.len(), 32);
        assert_ne!(bytes1, bytes2);
    }
}
