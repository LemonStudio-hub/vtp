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
}
