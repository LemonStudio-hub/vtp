//! VRF (Verifiable Random Function) module.
//!
//! Implements ECVRF-EDWARDS25519-SHA512-TAI according to RFC 9381.
//!
//! A VRF allows a party holding the private key to generate a verifiable random value,
//! and anyone can verify the correctness of that random value using the public key.
//!
//! # Algorithm (RFC 9381 Section 5)
//! 1. Keypair generation: Generate an ED25519 public/private keypair
//! 2. Proof generation (ECVRF_prove):
//!    - Encode input to curve point H (try-and-increment)
//!    - Compute gamma = H * sk
//!    - Generate nonce k
//!    - Compute U = G * k, V = H * k
//!    - Compute challenge c = Hash(G, H, PK, gamma, U, V)
//!    - Compute s = k + c * sk (mod l)
//!    - Return proof pi = (gamma, c, s)
//! 3. Proof verification (ECVRF_verify):
//!    - Decode proof to get gamma, c, s
//!    - Compute U = G*s - PK*c, V = H*s - gamma*c
//!    - Verify c = Hash(G, H, PK, gamma, U, V)
//! 4. VRF output (ECVRF_proof_to_hash):
//!    - Compute beta = Hash(0x04, gamma)

use curve25519_dalek::constants::ED25519_BASEPOINT_TABLE;
use curve25519_dalek::edwards::{CompressedEdwardsY, EdwardsPoint};
use curve25519_dalek::scalar::Scalar;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use sha2::{Digest, Sha512};
use wasm_bindgen::prelude::*;

/// Suite string for ECVRF-EDWARDS25519-SHA512-TAI
const SUITE_STRING: u8 = 0x04;

/// Domain separation for encode to curve
const ZERO_PREFIX: u8 = 0x00;
const TWO_PREFIX: u8 = 0x02;
const THREE_PREFIX: u8 = 0x03;

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

/// Encode a message to a curve point using Try-and-Increment (TAI) method.
///
/// Implements ECVRF_encode_to_curve_try_and_increment as per RFC 9381 Section 5.4.1.
///
/// # Arguments
/// - `public_key`: The public key (PK) used for domain separation
/// - `alpha`: The message to encode
///
/// # Returns
/// Returns a curve point H = hash_to_curve(PK, alpha)
fn encode_to_curve_tai(public_key: &[u8], alpha: &[u8]) -> EdwardsPoint {
    let mut hasher = Sha512::new();
    hasher.update([SUITE_STRING]);
    hasher.update([ZERO_PREFIX]);
    hasher.update(public_key);
    hasher.update(alpha);

    for ctr in 0u8..=255 {
        let mut h = hasher.clone();
        h.update([ctr]);
        let hash = h.finalize();

        // Try to decode as a curve point
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&hash[..32]);

        // Attempt decompression using CompressedEdwardsY
        let compressed = CompressedEdwardsY(bytes);
        if let Some(point) = compressed.decompress() {
            // Multiply by cofactor (8 for Ed25519) to ensure point is in prime-order subgroup
            return point.mul_by_cofactor();
        }
    }

    // This should never happen with a good hash function
    panic!("Failed to encode to curve point");
}

/// Generate a pseudorandom nonce for ECVRF.
///
/// Implements ECVRF_generate_nonce as per RFC 9381 Section 5.4.2.
/// Uses a deterministic nonce generation method (similar to EdDSA).
///
/// # Arguments
/// - `secret_key`: The secret key bytes
/// - `h_point`: The encoded curve point H
///
/// # Returns
/// Returns a scalar nonce k
fn generate_nonce(secret_key: &[u8], h_point: &EdwardsPoint) -> Scalar {
    // Deterministic nonce generation: k = Hash(secret_key || H_bytes) mod l
    let mut hasher = Sha512::new();
    hasher.update(secret_key);
    hasher.update(h_point.compress().to_bytes());
    let hash = hasher.finalize();

    // Convert hash to scalar (mod l)
    let mut scalar_bytes = [0u8; 64];
    scalar_bytes.copy_from_slice(&hash);
    Scalar::from_bytes_mod_order_wide(&scalar_bytes)
}

/// Compute the ECVRF challenge.
///
/// Implements ECVRF_challenge_generation as per RFC 9381 Section 5.4.3.
///
/// # Arguments
/// - `h_point`: The encoded curve point H
/// - `public_key_point`: The public key point PK
/// - `gamma`: The gamma point (H * sk)
/// - `u_point`: The U point (G * k)
/// - `v_point`: The V point (H * k)
///
/// # Returns
/// Returns a scalar challenge c
fn compute_challenge(
    h_point: &EdwardsPoint,
    public_key_point: &EdwardsPoint,
    gamma: &EdwardsPoint,
    u_point: &EdwardsPoint,
    v_point: &EdwardsPoint,
) -> Scalar {
    let mut hasher = Sha512::new();
    hasher.update([SUITE_STRING]);
    hasher.update([TWO_PREFIX]);
    hasher.update(public_key_point.compress().to_bytes());
    hasher.update(h_point.compress().to_bytes());
    hasher.update(gamma.compress().to_bytes());
    hasher.update(u_point.compress().to_bytes());
    hasher.update(v_point.compress().to_bytes());
    hasher.update([THREE_PREFIX]);

    let hash = hasher.finalize();

    // Truncate to 16 bytes (c_len = 16 for Ed25519)
    let mut challenge_bytes = [0u8; 16];
    challenge_bytes.copy_from_slice(&hash[..16]);

    // Convert to scalar - pad to 32 bytes and use from_bytes_mod_order
    let mut scalar_bytes = [0u8; 32];
    scalar_bytes[..16].copy_from_slice(&challenge_bytes);
    Scalar::from_bytes_mod_order(scalar_bytes)
}

/// Generate a VRF proof.
///
/// Implements ECVRF_prove as per RFC 9381 Section 5.1.
///
/// # Arguments
/// - `secret_key`: 32-byte private key
/// - `alpha`: The message to hash
///
/// # Returns
/// Returns the VRF proof pi = (gamma || c || s) as bytes
///
/// # Algorithm
/// 1. Derive public key from secret key
/// 2. Encode alpha to curve point H
/// 3. Compute gamma = H * sk
/// 4. Generate nonce k
/// 5. Compute U = G * k, V = H * k
/// 6. Compute challenge c = Hash(G, H, PK, gamma, U, V)
/// 7. Compute s = k + c * sk (mod l)
/// 8. Return proof pi = (gamma, c, s)
///
/// # Panics
/// Panics if `secret_key` length is not 32 bytes.
#[wasm_bindgen]
pub fn prove(secret_key: &[u8], alpha: &[u8]) -> Vec<u8> {
    let signing_key =
        SigningKey::from_bytes(secret_key.try_into().expect("Invalid secret key length"));
    let verifying_key = signing_key.verifying_key();
    let public_key_bytes = verifying_key.to_bytes();

    // Derive scalar from secret key via SHA-512 + clamping (Ed25519 standard)
    // This matches how ed25519-dalek derives the public key from the signing key
    let hash = Sha512::digest(secret_key);
    let mut sk_bytes = [0u8; 32];
    sk_bytes.copy_from_slice(&hash[..32]);
    sk_bytes[0] &= 248; // Clear lowest 3 bits
    sk_bytes[31] &= 127; // Clear highest bit
    sk_bytes[31] |= 64; // Set second-to-last bit
    let sk_scalar = Scalar::from_bytes_mod_order(sk_bytes);

    // Get public key point
    let pk_point = CompressedEdwardsY(public_key_bytes).decompress().expect("Invalid public key");

    // Encode alpha to curve point H
    let h_point = encode_to_curve_tai(&public_key_bytes, alpha);

    // Compute gamma = H * sk
    let gamma = h_point * sk_scalar;

    // Generate nonce k
    let k = generate_nonce(secret_key, &h_point);

    // Compute U = G * k, V = H * k
    let u_point = ED25519_BASEPOINT_TABLE * &k;
    let v_point = h_point * k;

    // Compute challenge c
    let c = compute_challenge(&h_point, &pk_point, &gamma, &u_point, &v_point);

    // Compute s = k + c * sk (mod l)
    let s = k + (c * sk_scalar);

    // Encode proof: gamma (32 bytes) || c (16 bytes) || s (32 bytes)
    let mut proof = Vec::with_capacity(80);
    proof.extend_from_slice(&gamma.compress().to_bytes());
    proof.extend_from_slice(&c.to_bytes()[..16]);
    proof.extend_from_slice(&s.to_bytes());

    proof
}

/// Extract VRF output from proof.
///
/// Implements ECVRF_proof_to_hash as per RFC 9381 Section 5.2.
///
/// # Arguments
/// - `proof`: The VRF proof bytes (80 bytes)
///
/// # Returns
/// Returns the VRF hash output beta (32 bytes)
///
/// # Algorithm
/// 1. Decode proof to get gamma
/// 2. Compute beta = Hash(suite_string || 0x04 || gamma)
///
/// # Panics
/// Panics if proof length is not 80 bytes.
#[wasm_bindgen]
pub fn proof_to_hash(proof: &[u8]) -> Vec<u8> {
    assert_eq!(proof.len(), 80, "Invalid proof length");

    // Extract gamma (first 32 bytes)
    let mut gamma_bytes = [0u8; 32];
    gamma_bytes.copy_from_slice(&proof[..32]);

    // Decompress gamma
    let gamma = curve25519_dalek::edwards::CompressedEdwardsY(gamma_bytes)
        .decompress()
        .expect("Invalid gamma in proof");

    // Compute beta = Hash(suite_string || 0x04 || gamma)
    let mut hasher = Sha512::new();
    hasher.update([SUITE_STRING]);
    hasher.update([THREE_PREFIX]);
    hasher.update(gamma.compress().to_bytes());
    let hash = hasher.finalize();

    // Return first 32 bytes
    let mut beta = vec![0u8; 32];
    beta.copy_from_slice(&hash[..32]);
    beta
}

/// Verify a VRF proof.
///
/// Implements ECVRF_verify as per RFC 9381 Section 5.3.
///
/// # Arguments
/// - `public_key`: 32-byte public key
/// - `alpha`: The original message
/// - `proof`: The VRF proof (80 bytes)
///
/// # Returns
/// - `true`: The proof is valid
/// - `false`: The proof is invalid or verification failed
///
/// # Algorithm
/// 1. Decode proof to get gamma, c, s
/// 2. Encode alpha to curve point H
/// 3. Compute U = G*s - PK*c
/// 4. Compute V = H*s - gamma*c
/// 5. Verify c = Hash(G, H, PK, gamma, U, V)
///
/// # Error Handling
/// The following cases will return `false`:
/// - Invalid public key format
/// - Invalid proof format
/// - Proof does not verify
#[wasm_bindgen]
pub fn verify(public_key: &[u8], alpha: &[u8], proof: &[u8]) -> bool {
    // Check proof length
    if proof.len() != 80 {
        return false;
    }

    // Decode public key
    let mut pk_bytes = [0u8; 32];
    pk_bytes.copy_from_slice(public_key);

    let pk_point = match CompressedEdwardsY(pk_bytes).decompress() {
        Some(point) => point,
        None => return false,
    };

    // Decode proof: gamma (32 bytes) || c (16 bytes) || s (32 bytes)
    let mut gamma_bytes = [0u8; 32];
    gamma_bytes.copy_from_slice(&proof[..32]);

    let gamma = match CompressedEdwardsY(gamma_bytes).decompress() {
        Some(point) => point,
        None => return false,
    };

    // Extract c (16 bytes) and convert to scalar
    let mut c_bytes = [0u8; 32];
    c_bytes[..16].copy_from_slice(&proof[32..48]);
    let c = Scalar::from_bytes_mod_order(c_bytes);

    // Extract s (32 bytes) and convert to scalar
    let mut s_bytes = [0u8; 32];
    s_bytes.copy_from_slice(&proof[48..80]);
    let s = Scalar::from_bytes_mod_order(s_bytes);

    // Encode alpha to curve point H
    let h_point = encode_to_curve_tai(public_key, alpha);

    // Compute U = G*s - PK*c
    let u_point = ED25519_BASEPOINT_TABLE * &s - pk_point * c;

    // Compute V = H*s - gamma*c
    let v_point = h_point * s - gamma * c;

    // Compute expected challenge
    let expected_c = compute_challenge(&h_point, &pk_point, &gamma, &u_point, &v_point);

    // Verify c matches
    c == expected_c
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
    /// 2. Proof length is 80 bytes
    /// 3. Proof can be correctly verified
    /// 4. VRF output can be extracted
    #[wasm_bindgen_test]
    fn test_prove_and_verify() {
        let keypair = generate_keypair();
        let message = b"test message";

        let proof = prove(&keypair.secret_key(), message);
        assert_eq!(proof.len(), 80);

        let is_valid = verify(&keypair.public_key(), message, &proof);
        assert!(is_valid);

        let vrf_output = proof_to_hash(&proof);
        assert_eq!(vrf_output.len(), 32);
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
    /// 2. Different messages produce different VRF outputs
    /// 3. A proof can only verify its corresponding message
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

        let hash1 = proof_to_hash(&proof1);
        let hash2 = proof_to_hash(&proof2);
        assert_ne!(hash1, hash2);
    }

    /// Test deterministic proof generation.
    ///
    /// Verifies:
    /// 1. Same input produces same proof
    /// 2. Same input produces same VRF output
    #[wasm_bindgen_test]
    fn test_deterministic_proof() {
        let keypair = generate_keypair();
        let message = b"deterministic test";

        let proof1 = prove(&keypair.secret_key(), message);
        let proof2 = prove(&keypair.secret_key(), message);

        assert_eq!(proof1, proof2);

        let hash1 = proof_to_hash(&proof1);
        let hash2 = proof_to_hash(&proof2);
        assert_eq!(hash1, hash2);
    }

    /// Test wrong keypair verification.
    ///
    /// Verifies:
    /// 1. Proof generated with one key doesn't verify with another
    #[wasm_bindgen_test]
    fn test_wrong_keypair_verify() {
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();
        let message = b"shared message";

        let proof = prove(&keypair1.secret_key(), message);

        let is_valid = verify(&keypair2.public_key(), message, &proof);
        assert!(!is_valid);
    }

    /// Test empty message handling.
    ///
    /// Verifies:
    /// 1. Empty message produces valid proof
    /// 2. Empty message proof verifies correctly
    #[wasm_bindgen_test]
    fn test_empty_message() {
        let keypair = generate_keypair();
        let message = b"";

        let proof = prove(&keypair.secret_key(), message);
        assert_eq!(proof.len(), 80);

        let is_valid = verify(&keypair.public_key(), message, &proof);
        assert!(is_valid);
    }

    /// Test large message handling.
    ///
    /// Verifies:
    /// 1. Large message produces valid proof
    /// 2. Large message proof verifies correctly
    #[wasm_bindgen_test]
    fn test_large_message() {
        let keypair = generate_keypair();
        let message = vec![0xABu8; 10 * 1024];

        let proof = prove(&keypair.secret_key(), &message);
        assert_eq!(proof.len(), 80);

        let is_valid = verify(&keypair.public_key(), &message, &proof);
        assert!(is_valid);
    }

    /// Test VRF output is pseudorandom.
    ///
    /// Verifies:
    /// 1. VRF output length is 32 bytes
    /// 2. Different inputs produce different outputs
    #[wasm_bindgen_test]
    fn test_vrf_output_pseudorandom() {
        let keypair = generate_keypair();

        let proof1 = prove(&keypair.secret_key(), b"input1");
        let proof2 = prove(&keypair.secret_key(), b"input2");

        let hash1 = proof_to_hash(&proof1);
        let hash2 = proof_to_hash(&proof2);

        assert_eq!(hash1.len(), 32);
        assert_eq!(hash2.len(), 32);
        assert_ne!(hash1, hash2);
    }

    /// Test proof bit flip detection.
    ///
    /// Verifies:
    /// 1. Any single bit flip in proof causes verification failure
    #[wasm_bindgen_test]
    fn test_proof_bit_flip_detection() {
        let keypair = generate_keypair();
        let message = b"bit flip test";

        let proof = prove(&keypair.secret_key(), message);

        for byte_idx in 0..proof.len() {
            for bit_idx in 0..8 {
                let mut flipped_proof = proof.clone();
                flipped_proof[byte_idx] ^= 1 << bit_idx;

                let is_valid = verify(&keypair.public_key(), message, &flipped_proof);
                assert!(
                    !is_valid,
                    "Verification should fail when bit {} of byte {} is flipped",
                    bit_idx, byte_idx
                );
            }
        }
    }
}
