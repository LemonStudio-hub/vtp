#[cfg(test)]
mod tests {
    use vtp_core::vrf::{generate_keypair, prove, verify};

    #[test]
    fn test_generate_keypair_native() {
        let keypair = generate_keypair();
        assert_eq!(keypair.public_key().len(), 32);
        assert_eq!(keypair.secret_key().len(), 32);
    }

    #[test]
    fn test_prove_and_verify_native() {
        let keypair = generate_keypair();
        let message = b"test message";

        let proof = prove(&keypair.secret_key(), message);
        assert!(!proof.is_empty());

        let is_valid = verify(&keypair.public_key(), message, &proof);
        assert!(is_valid);
    }

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

    #[test]
    fn test_keypair_uniqueness() {
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();

        assert_ne!(keypair1.public_key(), keypair2.public_key());
        assert_ne!(keypair1.secret_key(), keypair2.secret_key());
    }

    #[test]
    fn test_proof_length() {
        let keypair = generate_keypair();
        let message = b"test";

        let proof = prove(&keypair.secret_key(), message);

        assert_eq!(proof.len(), 64);
    }

    #[test]
    fn test_empty_message() {
        let keypair = generate_keypair();
        let message = b"";

        let proof = prove(&keypair.secret_key(), message);
        assert_eq!(proof.len(), 64);

        let is_valid = verify(&keypair.public_key(), message, &proof);
        assert!(is_valid);
    }

    #[test]
    fn test_large_message() {
        let keypair = generate_keypair();
        let message = vec![0xABu8; 10 * 1024];

        let proof = prove(&keypair.secret_key(), &message);
        assert_eq!(proof.len(), 64);

        let is_valid = verify(&keypair.public_key(), &message, &proof);
        assert!(is_valid);
    }

    #[test]
    fn test_wrong_keypair_verify() {
        let keypair1 = generate_keypair();
        let keypair2 = generate_keypair();
        let message = b"shared message";

        let proof = prove(&keypair1.secret_key(), message);

        let is_valid = verify(&keypair2.public_key(), message, &proof);
        assert!(!is_valid);
    }

    #[test]
    fn test_deterministic_proof() {
        let keypair = generate_keypair();
        let message = b"deterministic test";

        let proof1 = prove(&keypair.secret_key(), message);
        let proof2 = prove(&keypair.secret_key(), message);

        assert_eq!(proof1, proof2);
    }

    #[test]
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
