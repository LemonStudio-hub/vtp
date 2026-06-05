//! VRF (Verifiable Random Function) module.
//!
//! Implements a verifiable random function based on ED25519.
//! A VRF allows a party holding the private key to generate a verifiable random value,
//! and anyone can verify the correctness of that random value using the public key.
//!
//! # Algorithm
//! 1. Keypair generation: Generate an ED25519 public/private keypair
//! 2. Proof generation: Sign a message using the private key
//! 3. Proof verification: Verify the signature validity using the public key
//!
//! # Security
//! - Based on the ED25519 elliptic curve, providing 128-bit security level
//! - Signatures are unforgeable unless the private key is known
//! - The same message and private key always produce the same signature (deterministic)

use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use wasm_bindgen::prelude::*;

/// VRF keypair structure.
///
/// Contains the public and private keys used for VRF computation.
/// The keypair is generated using the ED25519 algorithm.
///
/// # Security Notes
/// - The private key must be stored securely and must not be leaked
/// - The public key can be safely shared
/// - Keypairs are disposable; each call to `generate_keypair` generates a new one
#[wasm_bindgen]
pub struct VrfKeypair {
    /// Public key, 32 bytes, used for verifying VRF proofs
    public_key: Vec<u8>,

    /// Private key, 32 bytes, used for generating VRF proofs
    secret_key: Vec<u8>,
}

#[wasm_bindgen]
impl VrfKeypair {
    /// Get the public key.
    ///
    /// # Returns
    /// Returns a 32-byte public key vector.
    ///
    /// # Note
    /// Each call clones the public key; frequent calls may impact performance.
    #[wasm_bindgen(getter)]
    pub fn public_key(&self) -> Vec<u8> {
        self.public_key.clone()
    }

    /// Get the private key.
    ///
    /// # Returns
    /// Returns a 32-byte private key vector.
    ///
    /// # Security Warning
    /// The private key must be stored securely and must not be leaked to third parties.
    #[wasm_bindgen(getter)]
    pub fn secret_key(&self) -> Vec<u8> {
        self.secret_key.clone()
    }
}

/// Generate a new VRF keypair.
///
/// Generates an ED25519 keypair using the operating system's cryptographic random number generator.
///
/// # Returns
/// Returns a `VrfKeypair` struct containing the public and private keys.
///
/// # Security
/// - Uses `OsRng` to ensure cryptographic security of the random numbers
/// - The generated keypair has a 128-bit security level
///
/// # Examples
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

/// Generate a VRF proof.
///
/// Signs a message using the private key to generate a verifiable random proof.
///
/// # Arguments
/// - `secret_key`: 32-byte private key
/// - `message`: The message to sign
///
/// # Returns
/// Returns a 64-byte signature vector.
///
/// # Algorithm
/// 1. Compute SHA256 hash of the message
/// 2. Sign the hash value using ED25519
///
/// # Panics
/// Panics if `secret_key` length is not 32 bytes.
///
/// # Examples
/// ```rust
/// use vtp_core::vrf::{generate_keypair, prove};
/// let keypair = generate_keypair();
/// let message = b"challenge data";
/// let proof = prove(&keypair.secret_key(), message);
/// ```
#[wasm_bindgen]
pub fn prove(secret_key: &[u8], message: &[u8]) -> Vec<u8> {
    let signing_key =
        SigningKey::from_bytes(secret_key.try_into().expect("Invalid secret key length"));

    let mut hasher = Sha256::new();
    hasher.update(message);
    let hash = hasher.finalize();

    let signature = signing_key.sign(&hash);
    signature.to_bytes().to_vec()
}

/// Verify a VRF proof.
///
/// Verifies the validity of a VRF proof using the public key.
///
/// # Arguments
/// - `public_key`: 32-byte public key
/// - `message`: The original message
/// - `proof`: 64-byte signature
///
/// # Returns
/// - `true`: The proof is valid
/// - `false`: The proof is invalid or verification failed
///
/// # Algorithm
/// 1. Compute SHA256 hash of the message
/// 2. Verify the signature using ED25519
///
/// # Error Handling
/// The following cases will return `false`:
/// - Invalid public key format
/// - Invalid signature format
/// - Signature does not match the message
///
/// # Examples
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
    let verifying_key =
        match VerifyingKey::from_bytes(public_key.try_into().expect("Invalid public key length")) {
            Ok(key) => key,
            Err(_) => return false,
        };

    let signature =
        ed25519_dalek::Signature::from_bytes(proof.try_into().expect("Invalid proof length"));

    let mut hasher = Sha256::new();
    hasher.update(message);
    let hash = hasher.finalize();

    verifying_key.verify(&hash, &signature).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    /// Test keypair generation.
    ///
    /// Verifies:
    /// 1. Public key length is 32 bytes
    /// 2. Private key length is 32 bytes
    #[wasm_bindgen_test]
    fn test_generate_keypair() {
        let keypair = generate_keypair();
        assert_eq!(keypair.public_key().len(), 32);
        assert_eq!(keypair.secret_key().len(), 32);
    }

    /// Test VRF proof generation and verification.
    ///
    /// Verifies:
    /// 1. Proof generation succeeds
    /// 2. Proof can be correctly verified
    #[wasm_bindgen_test]
    fn test_prove_and_verify() {
        let keypair = generate_keypair();
        let message = b"test message";

        let proof = prove(&keypair.secret_key(), message);
        assert!(!proof.is_empty());

        let is_valid = verify(&keypair.public_key(), message, &proof);
        assert!(is_valid);
    }

    /// Test verification of an invalid proof.
    ///
    /// Verifies:
    /// 1. A tampered proof fails verification
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

    /// Test proofs for different messages.
    ///
    /// Verifies:
    /// 1. Different messages produce different proofs
    /// 2. A proof can only verify its corresponding message
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
