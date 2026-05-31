#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};
    use vtp_core::vdf::{vdf_step, VdfIterator};

    #[test]
    fn test_vdf_step_native() {
        let state = [0u8; 32];
        let next_state = vdf_step(&state);

        assert_ne!(state, next_state);
        assert_eq!(next_state.len(), 32);
    }

    #[test]
    fn test_vdf_iterator_native() {
        let seed = [0u8; 32];
        let mut iter = VdfIterator::new(&seed, 1000);

        assert_eq!(iter.step(), 0);
        assert_eq!(iter.total(), 1000);

        let steps = iter.run_batch(500);
        assert_eq!(steps, 500);
        assert_eq!(iter.step(), 500);

        let steps = iter.run_batch(600);
        assert_eq!(steps, 500);
        assert_eq!(iter.step(), 1000);
        assert!(iter.is_finished());
    }

    #[test]
    fn test_deterministic_native() {
        let seed = [0u8; 32];

        let mut iter1 = VdfIterator::new(&seed, 100);
        iter1.run_batch(100);

        let mut iter2 = VdfIterator::new(&seed, 100);
        iter2.run_batch(100);

        assert_eq!(iter1.get_state(), iter2.get_state());
    }

    #[test]
    fn test_vdf_step_known_hash() {
        let state = [0u8; 32];
        let result = vdf_step(&state);

        let mut hasher = Sha256::new();
        hasher.update([0u8; 32]);
        let expected = hasher.finalize();

        assert_eq!(result.as_slice(), expected.as_slice());
    }

    #[test]
    fn test_vdf_step_consistency() {
        let state = [42u8; 32];
        let result1 = vdf_step(&state);
        let result2 = vdf_step(&state);

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_vdf_iterator_new() {
        let seed = [1u8; 32];
        let iter = VdfIterator::new(&seed, 500);

        assert_eq!(iter.step(), 0);
        assert_eq!(iter.total(), 500);
        assert!(!iter.is_finished());
    }

    #[test]
    fn test_vdf_iterator_next_until_done() {
        let seed = [0u8; 32];
        let total = 100;
        let mut iter = VdfIterator::new(&seed, total);

        while iter.next() {}

        assert_eq!(iter.step(), total);
        assert!(iter.is_finished());
        assert!(!iter.next());
    }

    #[test]
    fn test_vdf_iterator_run_batch_zero() {
        let seed = [0u8; 32];
        let mut iter = VdfIterator::new(&seed, 100);

        let steps = iter.run_batch(0);

        assert_eq!(steps, 0);
        assert_eq!(iter.step(), 0);
        assert!(!iter.is_finished());
    }

    #[test]
    fn test_vdf_iterator_run_batch_exact_total() {
        let seed = [0u8; 32];
        let total = 200;
        let mut iter = VdfIterator::new(&seed, total);

        let steps = iter.run_batch(total);

        assert_eq!(steps, total);
        assert_eq!(iter.step(), total);
        assert!(iter.is_finished());
    }

    #[test]
    fn test_vdf_iterator_run_batch_over_total() {
        let seed = [0u8; 32];
        let total = 150;
        let mut iter = VdfIterator::new(&seed, total);

        let steps = iter.run_batch(500);

        assert_eq!(steps, total);
        assert_eq!(iter.step(), total);
        assert!(iter.is_finished());
    }

    #[test]
    fn test_vdf_iterator_state_changes() {
        let seed = [0u8; 32];
        let mut iter = VdfIterator::new(&seed, 10);

        let mut prev_state = iter.get_state();
        for _ in 0..10 {
            iter.next();
            let current_state = iter.get_state();
            assert_ne!(prev_state, current_state);
            prev_state = current_state;
        }
    }

    #[test]
    fn test_vdf_iterator_large_batch() {
        let seed = [0u8; 32];
        let total = 10000;
        let mut iter = VdfIterator::new(&seed, total);

        let steps = iter.run_batch(10000);

        assert_eq!(steps, total);
        assert_eq!(iter.step(), total);
        assert!(iter.is_finished());
    }

    #[test]
    fn test_vdf_chain_determinism() {
        let seed = [0u8; 32];
        let n: u64 = 500;

        let mut state = [0u8; 32];
        state.copy_from_slice(&seed[..32]);
        for _ in 0..n {
            state = vdf_step(&state);
        }

        let mut iter = VdfIterator::new(&seed, n);
        iter.run_batch(n);

        assert_eq!(state.to_vec(), iter.get_state());
    }
}
