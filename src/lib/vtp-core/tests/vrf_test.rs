//! Tests for the Verifiable Random Function (VRF) implementation.
//!
//! This module validates the correctness and security properties of the VRF
//! subsystem, covering:
//! - Keypair generation and expected key lengths
//! - Proof generation and successful verification
//! - Rejection of tampered proofs
//! - Message-domain separation (different messages produce different proofs)
//! - Keypair uniqueness across independent generations
//! - Fixed proof length guarantees
//! - Edge cases: empty and large messages
//! - Cross-keypair verification rejection
//! - Deterministic proof generation
//! - Single-bit-flip detection sensitivity

#[cfg(test)]
mod tests {
    use vtp_core::vrf::{generate_keypair, prove, verify};

    /// Tests that [`generate_keypair`] produces keys of the expected size.
    ///
    /// - Public key is exactly 32 bytes
    /// - Secret key is exactly 32 bytes
    #[test]
    fn test_generate_keypair_native() {
        let keypair = generate_keypair();
        assert_eq!(keypair.public_key().len(), 32);
        assert_eq!(keypair.secret_key().len(), 32);
    }

    /// Tests the core prove-and-verify round trip.
    ///
    /// - A proof generated with the correct secret key is non-empty
    /// - Verification succeeds when using the matching public key and message
    #[test]
    fn test_prove_and_verify_native() {
        let keypair = generate_keypair();
        let message = b"test message";

        let proof = prove(&keypair.secret_key(), message);
        assert!(!proof.is_empty());

        let is_valid = verify(&keypair.public_key(), message, &proof);
        assert!(is_valid);
    }

    /// Tests that a tampered proof is correctly rejected during verification.
    ///
    /// - A valid proof is generated, then its first byte is XOR-flipped
    /// - Verification with the corrupted proof returns `false`
    /// - Ensures the VRF is resistant to simple byte-level tampering
    #[test]
    fn test_invalid_proof_native() {
        let keypair = generate_keypair();
        let message = b"test message";

        let proof = prove(&keypair.secret_key(), message);
        let mut invalid_proof = proof.clone();
        invalid_proof[0] ^= 0xff;

        let is_valid = verify(&keypair.public_key(), message, &invalid_proof);
        assert!(!is_valid);
    }

    /// Tests that proofs are bound to their specific messages (message-domain separation).
    ///
    /// - Proofs generated for different messages verify correctly with their own message
    /// - A proof generated for `message1` does NOT verify against `message2`
    /// - Ensures proofs cannot be replayed across different messages
    #[test]
    fn test_different_messages_native() {
        let keypair = generate_keypair();
        let message1 = b"message 1";
        let message2 = b"message 2";

        let proof1 = prove(&keypair.secret_key(), message1);
        let proof2 = prove(&keypair.secret_key(), message2);

        assert!(verify(&keypair.public_key(), message1, &proof1));
        assert!(verify(&keypair.public_key(), message2, &proof2));
        assert!(!verify(&keypair.public_key(), message1, &proof2));
    }

    /// Tests that independently generated keypairs are unique.
    ///
    /// - Two calls to `generate_keypair` produce different public keys
    /// - Two calls to `generate_keypair` produce different secret keys
    /// - Ensures the key generation uses sufficient randomness
    #[test]
    fn test_keypair_uniqueness() {
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();

        assert_ne!(keypair1.public_key(), keypair2.public_key());
        assert_ne!(keypair1.secret_key(), keypair2.secret_key());
    }

    /// Tests that VRF proofs always have a fixed length of 64 bytes.
    ///
    /// - A proof generated for a short message is exactly 64 bytes
    /// - Ensures downstream consumers can rely on a constant proof size
    #[test]
    fn test_proof_length() {
        let keypair = generate_keypair();
        let message = b"test";

        let proof = prove(&keypair.secret_key(), message);

        assert_eq!(proof.len(), 64);
    }

    /// Tests that VRF handles empty messages correctly.
    ///
    /// - A proof for an empty message is still 64 bytes
    /// - Verification of the empty-message proof succeeds
    /// - Ensures the VRF does not break on zero-length inputs
    #[test]
    fn test_empty_message() {
        let keypair = generate_keypair();
        let message = b"";

        let proof = prove(&keypair.secret_key(), message);
        assert_eq!(proof.len(), 64);

        let is_valid = verify(&keypair.public_key(), message, &proof);
        assert!(is_valid);
    }

    /// Tests that VRF handles large messages correctly.
    ///
    /// - A 10 KiB message produces a 64-byte proof (same as any other message size)
    /// - Verification of the large-message proof succeeds
    /// - Ensures the VRF is not affected by message size
    #[test]
    fn test_large_message() {
        let keypair = generate_keypair();
        let message = vec![0xABu8; 10 * 1024];

        let proof = prove(&keypair.secret_key(), &message);
        assert_eq!(proof.len(), 64);

        let is_valid = verify(&keypair.public_key(), &message, &proof);
        assert!(is_valid);
    }

    /// Tests that a proof generated with one keypair does not verify with a
    /// different keypair's public key.
    ///
    /// - A proof created using `keypair1`'s secret key
    /// - Verification against `keypair2`'s public key returns `false`
    /// - Ensures proofs are cryptographically bound to the proving key
    #[test]
    fn test_wrong_keypair_verify() {
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();
        let message = b"shared message";

        let proof = prove(&keypair1.secret_key(), message);

        let is_valid = verify(&keypair2.public_key(), message, &proof);
        assert!(!is_valid);
    }

    /// Tests that VRF proof generation is deterministic.
    ///
    /// - Two proofs generated for the same key and message are byte-for-byte identical
    /// - Ensures reproducibility, which is critical for verification consistency
    #[test]
    fn test_deterministic_proof() {
        let keypair = generate_keypair();
        let message = b"deterministic test";

        let proof1 = prove(&keypair.secret_key(), message);
        let proof2 = prove(&keypair.secret_key(), message);

        assert_eq!(proof1, proof2);
    }

    /// Tests that flipping any single bit in a valid proof causes verification
    /// to fail, demonstrating the avalanche effect.
    ///
    /// - Iterates over every byte and every bit position in the 64-byte proof
    /// - Flips one bit at a time and verifies the proof is rejected
    /// - Ensures even minimal tampering is detected
    /// - This is a comprehensive bit-sensitivity check (64 bytes × 8 bits = 512 assertions)
    #[test]
    fn test_proof_bit_flip_detection() {
        let keypair = generate_keypair();
        let message = b"bit flip test";

        let proof = prove(&keypair.secret_key(), message);

        for byte_idx in 0..proof.len() {
            for bit_idx in 0..8 {
                let mut flipped_proof = proof.clone();
                flipped_proof[byte_idx] ^= 1 << bit_idx; // flip exactly one bit

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
