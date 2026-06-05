//! Tests for the Verifiable Delay Function (VDF) implementation.
//!
//! This module validates the correctness and determinism of the VDF subsystem,
//! covering:
//! - Single-step hash computation and output properties
//! - VDF iterator creation, batch execution, and completion
//! - Deterministic state derivation from identical seeds
//! - Known-hash verification against a reference SHA-256 computation
//! - Idempotency of repeated step calls on the same input
//! - Iterator state transitions across `next()` and `run_batch()` calls
//! - Edge cases: zero-step batch, exact-total batch, over-total batch
//! - Large batch processing
//! - Equivalence between manual step chaining and iterator-based computation

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};
    use vtp_core::vdf::{vdf_step, VdfIterator};

    /// Tests that a single VDF step transforms the input state.
    ///
    /// - The output state is different from the input state
    /// - The output length is 32 bytes (unchanged from input)
    #[test]
    fn test_vdf_step_native() {
        let state = [0u8; 32];
        let next_state = vdf_step(&state);

        assert_ne!(state, next_state);
        assert_eq!(next_state.len(), 32);
    }

    /// Tests the [`VdfIterator`] batch execution and step tracking.
    ///
    /// - A newly created iterator starts at step 0 with the configured total
    /// - A batch of 500 steps advances the iterator to step 500
    /// - A subsequent batch of 600 steps is capped at the remaining 500 (total = 1000)
    /// - The iterator reports `is_finished()` once the total is reached
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
        assert_eq!(steps, 500); // capped at remaining steps (1000 - 500)
        assert_eq!(iter.step(), 1000);
        assert!(iter.is_finished());
    }

    /// Tests that two iterators initialized with the same seed produce identical
    /// final states, proving deterministic computation.
    ///
    /// - Both iterators are created with the same seed and total steps
    /// - After running the full batch, their internal states are byte-for-byte equal
    #[test]
    fn test_deterministic_native() {
        let seed = [0u8; 32];

        let mut iter1 = VdfIterator::new(&seed, 100);
        iter1.run_batch(100);

        let mut iter2 = VdfIterator::new(&seed, 100);
        iter2.run_batch(100);

        assert_eq!(iter1.get_state(), iter2.get_state());
    }

    /// Tests that `vdf_step` on an all-zero input produces the SHA-256 hash of
    /// that input, validating the step function against a known reference.
    ///
    /// - Computes `vdf_step(&[0u8; 32])` and independently computes `SHA-256([0u8; 32])`
    /// - Both results are byte-for-byte identical
    #[test]
    fn test_vdf_step_known_hash() {
        let state = [0u8; 32];
        let result = vdf_step(&state);

        let mut hasher = Sha256::new();
        hasher.update([0u8; 32]);
        let expected = hasher.finalize();

        assert_eq!(result.as_slice(), expected.as_slice());
    }

    /// Tests that calling `vdf_step` twice on the same input yields identical results,
    /// confirming idempotency (determinism of the step function itself).
    ///
    /// - Two calls to `vdf_step` with `[42u8; 32]` produce the same output
    #[test]
    fn test_vdf_step_consistency() {
        let state = [42u8; 32];
        let result1 = vdf_step(&state);
        let result2 = vdf_step(&state);

        assert_eq!(result1, result2);
    }

    /// Tests that a newly created [`VdfIterator`] starts in the expected initial state.
    ///
    /// - Step counter is 0
    /// - Total matches the configured value
    /// - Iterator is not yet finished
    #[test]
    fn test_vdf_iterator_new() {
        let seed = [1u8; 32];
        let iter = VdfIterator::new(&seed, 500);

        assert_eq!(iter.step(), 0);
        assert_eq!(iter.total(), 500);
        assert!(!iter.is_finished());
    }

    /// Tests that calling `next()` repeatedly drives the iterator to completion.
    ///
    /// - Repeated `next()` calls advance the step counter one at a time
    /// - After all steps are consumed, `next()` returns `false`
    /// - The iterator reports `is_finished()` and the step equals the total
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

    /// Tests that `run_batch(0)` is a no-op.
    ///
    /// - Requesting zero steps returns 0
    /// - The step counter remains at 0
    /// - The iterator is not finished
    #[test]
    fn test_vdf_iterator_run_batch_zero() {
        let seed = [0u8; 32];
        let mut iter = VdfIterator::new(&seed, 100);

        let steps = iter.run_batch(0);

        assert_eq!(steps, 0);
        assert_eq!(iter.step(), 0);
        assert!(!iter.is_finished());
    }

    /// Tests that a batch matching exactly the total steps completes the iterator.
    ///
    /// - `run_batch(total)` processes all remaining steps
    /// - The step counter equals the total
    /// - The iterator reports `is_finished()`
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

    /// Tests that requesting more steps than the total caps execution at the total.
    ///
    /// - `run_batch(500)` on a 150-step iterator processes only 150 steps
    /// - The returned step count equals the total (not the requested batch size)
    /// - The iterator reports `is_finished()`
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

    /// Tests that each `next()` call produces a distinct state, confirming that
    /// the VDF state evolves on every step.
    ///
    /// - Calls `next()` 10 times and captures state after each step
    /// - Each consecutive state is different from the previous one
    /// - Validates that the VDF is not stuck or cycling
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

    /// Tests that a large batch (10,000 steps) processes correctly via `run_batch`.
    ///
    /// - All 10,000 steps are processed in a single batch call
    /// - The step counter equals the total
    /// - The iterator reports `is_finished()`
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

    /// Tests that manually chaining `vdf_step` calls produces the same final state
    /// as using the [`VdfIterator`] with `run_batch`, proving equivalence.
    ///
    /// - Runs 500 manual `vdf_step` calls starting from the seed
    /// - Runs an equivalent `VdfIterator` for 500 steps
    /// - Both produce identical 32-byte final states
    #[test]
    fn test_vdf_chain_determinism() {
        let seed = [0u8; 32];
        let n: u64 = 500;

        let mut state = [0u8; 32];
        state.copy_from_slice(&seed[..32]);
        // Manually apply vdf_step 500 times to build the reference chain
        for _ in 0..n {
            state = vdf_step(&state);
        }

        let mut iter = VdfIterator::new(&seed, n);
        iter.run_batch(n); // run the same 500 steps via the iterator

        assert_eq!(state.to_vec(), iter.get_state());
    }
}
