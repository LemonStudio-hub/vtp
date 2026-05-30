//! VRF (Verifiable Random Function) 模块
//!
//! 实现了基于 ED25519 的可验证随机函数。
//! VRF 允许持有私钥的一方生成一个可验证的随机值，
//! 任何人都可以使用公钥验证该随机值的正确性。
//!
//! # 算法说明
//! 1. 生成密钥对：使用 ED25519 生成公私钥对
//! 2. 证明生成：使用私钥对消息进行签名
//! 3. 证明验证：使用公钥验证签名的有效性
//!
//! # 安全性
//! - 基于 ED25519 椭圆曲线，提供 128 位安全级别
//! - 签名不可伪造，除非知道私钥
//! - 相同消息和私钥总是产生相同的签名（确定性）

use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use wasm_bindgen::prelude::*;

/// VRF 密钥对结构体
///
/// 包含用于 VRF 计算的公钥和私钥。
/// 密钥对通过 ED25519 算法生成。
///
/// # 安全注意事项
/// - 私钥应该安全存储，不应泄露
/// - 公钥可以安全分享
/// - 密钥对是一次性的，每次调用 `generate_keypair` 都会生成新的
#[wasm_bindgen]
pub struct VrfKeypair {
    /// 公钥，32 字节，用于验证 VRF 证明
    public_key: Vec<u8>,

    /// 私钥，32 字节，用于生成 VRF 证明
    secret_key: Vec<u8>,
}

#[wasm_bindgen]
impl VrfKeypair {
    /// 获取公钥
    ///
    /// # 返回值
    /// 返回 32 字节的公钥向量
    ///
    /// # 注意
    /// 每次调用都会克隆公钥，频繁调用可能影响性能
    #[wasm_bindgen(getter)]
    pub fn public_key(&self) -> Vec<u8> {
        self.public_key.clone()
    }

    /// 获取私钥
    ///
    /// # 返回值
    /// 返回 32 字节的私钥向量
    ///
    /// # 安全警告
    /// 私钥应该安全存储，不应泄露给第三方
    #[wasm_bindgen(getter)]
    pub fn secret_key(&self) -> Vec<u8> {
        self.secret_key.clone()
    }
}

/// 生成新的 VRF 密钥对
///
/// 使用操作系统提供的随机数生成器生成 ED25519 密钥对。
///
/// # 返回值
/// 返回包含公钥和私钥的 VrfKeypair 结构体
///
/// # 安全性
/// - 使用 `OsRng` 确保随机数的密码学安全性
/// - 生成的密钥对具有 128 位安全级别
///
/// # 示例
/// ```rust
/// use vtp_core::vrf::generate_keypair;
/// let keypair = generate_keypair();
/// println!("Public key: {:?}", keypair.public_key());
/// ```
#[wasm_bindgen]
pub fn generate_keypair() -> VrfKeypair {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();

    VrfKeypair {
        public_key: verifying_key.to_bytes().to_vec(),
        secret_key: signing_key.to_bytes().to_vec(),
    }
}

/// 生成 VRF 证明
///
/// 使用私钥对消息进行签名，生成可验证的随机证明。
///
/// # 参数
/// - `secret_key`: 32 字节的私钥
/// - `message`: 要签名的消息
///
/// # 返回值
/// 返回 64 字节的签名向量
///
/// # 算法
/// 1. 对消息进行 SHA256 哈希
/// 2. 使用 ED25519 对哈希值进行签名
///
/// # Panics
/// 如果 secret_key 长度不是 32 字节，会 panic
///
/// # 示例
/// ```rust
/// use vtp_core::vrf::{generate_keypair, prove};
/// let keypair = generate_keypair();
/// let message = b"challenge data";
/// let proof = prove(&keypair.secret_key(), message);
/// ```
#[wasm_bindgen]
pub fn prove(secret_key: &[u8], message: &[u8]) -> Vec<u8> {
    let signing_key = SigningKey::from_bytes(secret_key.try_into().expect("Invalid secret key length"));

    let mut hasher = Sha256::new();
    hasher.update(message);
    let hash = hasher.finalize();

    let signature = signing_key.sign(&hash);
    signature.to_bytes().to_vec()
}

/// 验证 VRF 证明
///
/// 使用公钥验证 VRF 证明的有效性。
///
/// # 参数
/// - `public_key`: 32 字节的公钥
/// - `message`: 原始消息
/// - `proof`: 64 字节的签名
///
/// # 返回值
/// - `true`: 证明有效
/// - `false`: 证明无效或验证失败
///
/// # 算法
/// 1. 对消息进行 SHA256 哈希
/// 2. 使用 ED25519 验证签名
///
/// # 错误处理
/// 以下情况会返回 false：
/// - 公钥格式无效
/// - 签名格式无效
/// - 签名与消息不匹配
///
/// # 示例
/// ```rust
/// use vtp_core::vrf::{generate_keypair, prove, verify};
/// let keypair = generate_keypair();
/// let message = b"challenge data";
/// let proof = prove(&keypair.secret_key(), message);
///
/// assert!(verify(&keypair.public_key(), message, &proof));
/// assert!(!verify(&keypair.public_key(), b"wrong message", &proof));
/// ```
#[wasm_bindgen]
pub fn verify(public_key: &[u8], message: &[u8], proof: &[u8]) -> bool {
    let verifying_key = match VerifyingKey::from_bytes(public_key.try_into().expect("Invalid public key length")) {
        Ok(key) => key,
        Err(_) => return false,
    };

    let signature = match ed25519_dalek::Signature::from_bytes(proof.try_into().expect("Invalid proof length")) {
        sig => sig,
    };

    let mut hasher = Sha256::new();
    hasher.update(message);
    let hash = hasher.finalize();

    verifying_key.verify(&hash, &signature).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    /// 测试密钥对生成
    ///
    /// 验证：
    /// 1. 公钥长度为 32 字节
    /// 2. 私钥长度为 32 字节
    #[wasm_bindgen_test]
    fn test_generate_keypair() {
        let keypair = generate_keypair();
        assert_eq!(keypair.public_key().len(), 32);
        assert_eq!(keypair.secret_key().len(), 32);
    }

    /// 测试 VRF 证明生成和验证
    ///
    /// 验证：
    /// 1. 证明生成成功
    /// 2. 证明可以被正确验证
    #[wasm_bindgen_test]
    fn test_prove_and_verify() {
        let keypair = generate_keypair();
        let message = b"test message";

        let proof = prove(&keypair.secret_key(), message);
        assert!(!proof.is_empty());

        let is_valid = verify(&keypair.public_key(), message, &proof);
        assert!(is_valid);
    }

    /// 测试无效证明的验证
    ///
    /// 验证：
    /// 1. 修改后的证明无法通过验证
    #[wasm_bindgen_test]
    fn test_invalid_proof() {
        let keypair = generate_keypair();
        let message = b"test message";

        let proof = prove(&keypair.secret_key(), message);
        let mut invalid_proof = proof.clone();
        invalid_proof[0] ^= 0xff;

        let is_valid = verify(&keypair.public_key(), message, &invalid_proof);
        assert!(!is_valid);
    }

    /// 测试不同消息的证明
    ///
    /// 验证：
    /// 1. 不同消息产生不同的证明
    /// 2. 证明只能验证对应的消息
    #[wasm_bindgen_test]
    fn test_different_messages() {
        let keypair = generate_keypair();
        let message1 = b"message 1";
        let message2 = b"message 2";

        let proof1 = prove(&keypair.secret_key(), message1);
        let proof2 = prove(&keypair.secret_key(), message2);

        assert!(verify(&keypair.public_key(), message1, &proof1));
        assert!(verify(&keypair.public_key(), message2, &proof2));
        assert!(!verify(&keypair.public_key(), message1, &proof2));
    }
}
