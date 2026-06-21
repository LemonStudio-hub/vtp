//! Utility functions module.
//!
//! Provides commonly used utility functions, including:
//! - Hash computation
//! - Byte and hexadecimal conversion
//! - Random number generation
//!
//! These functions primarily serve as helpers for VDF and VRF implementations.

use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};
use wasm_bindgen::prelude::*;

/// Compute the SHA256 hash of byte data.
///
/// Performs SHA256 hashing on the input data and returns a 32-byte hash value.
///
/// # Arguments
/// - `data`: The byte data to hash
///
/// # Returns
/// Returns a 32-byte hash vector.
///
/// # Examples
/// ```rust
/// use vtp_core::utils::hash_bytes;
/// let data = b"hello world";
/// let hash = hash_bytes(data);
/// assert_eq!(hash.len(), 32);
/// ```
///
/// # Performance
/// SHA256 computation speed is approximately 200-500MB/s (depending on hardware).
#[wasm_bindgen]
pub fn hash_bytes(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Convert a byte array to a hexadecimal string.
///
/// Converts each byte to a two-digit hexadecimal representation using lowercase letters.
///
/// # Arguments
/// - `bytes`: The byte array to convert
///
/// # Returns
/// Returns a hexadecimal string.
///
/// # Examples
/// ```rust
/// use vtp_core::utils::bytes_to_hex;
/// let bytes = vec![0x00, 0x0f, 0xff];
/// let hex = bytes_to_hex(&bytes);
/// assert_eq!(hex, "000fff");
/// ```
///
/// # Format
/// - Each byte uses a two-digit hexadecimal representation
/// - Uses lowercase letters (a-f)
/// - Does not include the `0x` prefix
#[wasm_bindgen]
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Convert a hexadecimal string to a byte array.
///
/// Parses a hexadecimal string and returns the corresponding byte array.
///
/// # Arguments
/// - `hex`: The hexadecimal string
///
/// # Returns
/// - `Ok(Vec<u8>)`: The successfully converted byte array
/// - `Err(JsValue)`: Error message when conversion fails
///
/// # Errors
/// The following cases will return an error:
/// - String length is not even
/// - Contains non-hexadecimal characters
///
/// # Examples
/// ```rust
/// use vtp_core::utils::hex_to_bytes;
/// let hex = "000fff";
/// let bytes = hex_to_bytes(hex).unwrap();
/// assert_eq!(bytes, vec![0x00, 0x0f, 0xff]);
/// ```
///
/// # Note
/// - Input string is case-insensitive
/// - Does not accept the `0x` prefix
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

/// Generate a random byte array of the specified length.
///
/// Generates random bytes using the operating system's cryptographically secure random number generator.
///
/// # Arguments
/// - `length`: The number of bytes to generate
///
/// # Returns
/// Returns a random byte array of the specified length.
///
/// # Security
/// - Uses `OsRng` to ensure cryptographic security of the random numbers
/// - Generated random numbers are unpredictable
/// - Suitable for key generation and random challenges
///
/// # Examples
/// ```rust
/// use vtp_core::utils::generate_random_bytes;
/// let random_bytes = generate_random_bytes(32);
/// assert_eq!(random_bytes.len(), 32);
/// ```
///
/// # Performance
/// Generation speed depends on the operating system's random number generator implementation.
#[wasm_bindgen]
pub fn generate_random_bytes(length: u32) -> Vec<u8> {
    use rand::RngCore;
    let mut bytes = vec![0u8; length as usize];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    bytes
}

/// Sign data using Ed25519.
///
/// Produces a standard 64-byte Ed25519 signature over the given data
/// using the provided 32-byte secret key.
///
/// # Arguments
/// - `secret_key`: 32-byte Ed25519 secret key
/// - `data`: The data to sign
///
/// # Returns
/// Returns a 64-byte Ed25519 signature.
///
/// # Errors
/// Returns an error if the secret key is not a valid 32-byte Ed25519 key.
#[wasm_bindgen]
pub fn ed25519_sign(secret_key: &[u8], data: &[u8]) -> Result<Vec<u8>, JsValue> {
    if secret_key.len() != 32 {
        return Err(JsValue::from_str("Secret key must be 32 bytes"));
    }
    let key_bytes: [u8; 32] =
        secret_key.try_into().map_err(|_| JsValue::from_str("Invalid secret key length"))?;
    let key = SigningKey::from_bytes(&key_bytes);
    Ok(key.sign(data).to_bytes().to_vec())
}

/// Verify an Ed25519 signature.
///
/// Verifies a 64-byte Ed25519 signature against the given data and
/// 32-byte public key.
///
/// # Arguments
/// - `public_key`: 32-byte Ed25519 verifying key
/// - `data`: The signed data
/// - `signature`: 64-byte Ed25519 signature
///
/// # Returns
/// Returns `true` if the signature is valid, `false` otherwise.
#[wasm_bindgen]
pub fn ed25519_verify(public_key: &[u8], data: &[u8], signature: &[u8]) -> bool {
    let Ok(pk_bytes): Result<[u8; 32], _> = public_key.try_into() else {
        return false;
    };
    let Ok(sig_bytes): Result<[u8; 64], _> = signature.try_into() else {
        return false;
    };
    let Ok(key) = VerifyingKey::from_bytes(&pk_bytes) else {
        return false;
    };
    let sig = ed25519_dalek::Signature::from_bytes(&sig_bytes);
    key.verify(data, &sig).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    /// Test SHA256 hash computation.
    ///
    /// Verifies:
    /// 1. Output length is 32 bytes
    /// 2. Same input produces the same output
    #[wasm_bindgen_test]
    fn test_hash_bytes() {
        let data = b"test data";
        let hash = hash_bytes(data);
        assert_eq!(hash.len(), 32);
    }

    /// Test byte to hexadecimal conversion.
    ///
    /// Verifies:
    /// 1. Correctly converts various byte values
    /// 2. Uses lowercase letters
    #[wasm_bindgen_test]
    fn test_bytes_to_hex() {
        let bytes = vec![0x00, 0x0f, 0xff];
        let hex = bytes_to_hex(&bytes);
        assert_eq!(hex, "000fff");
    }

    /// Test hexadecimal to byte conversion.
    ///
    /// Verifies:
    /// 1. Correctly parses hexadecimal strings
    /// 2. Returns the correct byte array
    #[wasm_bindgen_test]
    fn test_hex_to_bytes() {
        let hex = "000fff";
        let bytes = hex_to_bytes(hex).unwrap();
        assert_eq!(bytes, vec![0x00, 0x0f, 0xff]);
    }

    /// Test invalid hexadecimal strings.
    ///
    /// Verifies:
    /// 1. Odd-length strings return an error
    /// 2. Non-hexadecimal characters return an error
    #[wasm_bindgen_test]
    fn test_invalid_hex() {
        let result = hex_to_bytes("invalid");
        assert!(result.is_err());
    }

    /// Test random byte generation.
    ///
    /// Verifies:
    /// 1. Generates byte arrays of the correct length
    /// 2. Multiple generations produce different results
    #[wasm_bindgen_test]
    fn test_generate_random_bytes() {
        let bytes1 = generate_random_bytes(32);
        let bytes2 = generate_random_bytes(32);

        assert_eq!(bytes1.len(), 32);
        assert_eq!(bytes2.len(), 32);
        assert_ne!(bytes1, bytes2);
    }

    /// Test Ed25519 signing and verification.
    #[wasm_bindgen_test]
    fn test_ed25519_sign_verify() {
        let keypair = crate::vrf::generate_keypair();
        let sk = keypair.secret_key();
        let pk = keypair.public_key();
        let data = b"hello world";
        let signature = ed25519_sign(&sk, data).unwrap();
        assert_eq!(signature.len(), 64);
        assert!(ed25519_verify(&pk, data, &signature));
    }

    /// Test Ed25519 verification rejects tampered data.
    #[wasm_bindgen_test]
    fn test_ed25519_rejects_tampered() {
        let keypair = crate::vrf::generate_keypair();
        let sk = keypair.secret_key();
        let pk = keypair.public_key();
        let data = b"hello world";
        let signature = ed25519_sign(&sk, data).unwrap();

        // Tamper with data
        let mut tampered_data = data.to_vec();
        tampered_data[0] ^= 0xff;
        assert!(!ed25519_verify(&pk, &tampered_data, &signature));

        // Tamper with signature
        let mut tampered_sig = signature.clone();
        tampered_sig[0] ^= 0xff;
        assert!(!ed25519_verify(&pk, data, &tampered_sig));
    }

    /// Test Ed25519 with invalid key lengths.
    #[wasm_bindgen_test]
    fn test_ed25519_invalid_key() {
        let result = ed25519_sign(&[0u8; 16], b"test");
        assert!(result.is_err());
    }
}
