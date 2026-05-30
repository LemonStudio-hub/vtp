#[cfg(test)]
mod tests {
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
}
