//! VDF (Verifiable Delay Function) module.
//!
//! Implements a verifiable delay function based on consecutive SHA256 iterations.
//! A VDF is a function that requires sequential computation and cannot be accelerated through parallelization.
//!
//! # Algorithm
//! Each VDF step computes a SHA256 hash of the previous step's result:
//! state[i+1] = SHA256(state[i])
//!
//! # Performance Targets
//! - In a WebAssembly environment, single-step latency ≤ 500ns
//! - Corresponding throughput ≥ 2M steps/second

use sha2::{Digest, Sha256};
use wasm_bindgen::prelude::*;

/// VDF iterator, used to manage the state and progress of VDF computation.
///
/// This struct encapsulates the complete state of VDF computation, including:
/// - Current computation state (32-byte hash value)
/// - Number of completed steps
/// - Total steps target
///
/// # Use Cases
/// - Long-running VDF computation tasks
/// - Computations that require pause/resume support
/// - Batch processing optimization
///
/// # Examples
/// ```rust
/// use vtp_core::vdf::VdfIterator;
/// let seed = [0u8; 32];
/// let mut iterator = VdfIterator::new(&seed, 1000000);
///
/// // Execute 1000 steps
/// let steps = iterator.run_batch(1000);
///
/// // Check progress
/// println!("Progress: {}/{}", iterator.step(), iterator.total());
/// ```
#[wasm_bindgen]
pub struct VdfIterator {
    /// Current VDF state, a 32-byte hash value
    state: [u8; 32],

    /// Number of completed VDF steps
    step: u64,

    /// Total steps target
    total: u64,
}

#[wasm_bindgen]
impl VdfIterator {
    /// Create a new VDF iterator.
    ///
    /// # Arguments
    /// - `seed`: Initial seed, at least 32 bytes
    /// - `total`: Total steps target
    ///
    /// # Returns
    /// Returns an initialized `VdfIterator` instance.
    ///
    /// # Panics
    /// Panics if `seed` is less than 32 bytes.
    #[wasm_bindgen(constructor)]
    pub fn new(seed: &[u8], total: u64) -> Self {
        let mut state = [0u8; 32];
        state.copy_from_slice(&seed[..32]);

        Self { state, step: 0, total }
    }

    /// Get the number of completed steps.
    ///
    /// # Returns
    /// Returns the number of VDF iterations completed so far.
    pub fn step(&self) -> u64 {
        self.step
    }

    /// Get the total steps target.
    ///
    /// # Returns
    /// Returns the total steps target for the VDF computation.
    pub fn total(&self) -> u64 {
        self.total
    }

    /// Check whether the VDF computation has completed.
    ///
    /// # Returns
    /// - `true`: All steps have been completed
    /// - `false`: There are remaining steps
    pub fn is_finished(&self) -> bool {
        self.step >= self.total
    }

    /// Get the current VDF state.
    ///
    /// # Returns
    /// Returns a 32-byte vector of the current state.
    ///
    /// # Note
    /// Each call creates a new `Vec`; frequent calls may impact performance.
    pub fn get_state(&self) -> Vec<u8> {
        self.state.to_vec()
    }

    /// Execute a single VDF computation step.
    ///
    /// # Returns
    /// - `true`: Successfully executed one step
    /// - `false`: All steps have been completed, cannot continue
    ///
    /// # Performance Considerations
    /// For batch processing, it is recommended to use the `run_batch` method for better performance.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> bool {
        if self.is_finished() {
            return false;
        }

        self.state = vdf_step(&self.state);
        self.step += 1;
        true
    }

    /// Execute VDF computation in batch.
    ///
    /// Executes up to `max_steps` steps of VDF computation, or until all steps are completed.
    /// This method optimizes loop overhead and is suitable for long-running computation tasks.
    ///
    /// # Arguments
    /// - `max_steps`: Maximum number of steps to execute
    ///
    /// # Returns
    /// Returns the actual number of steps executed.
    ///
    /// # Performance Notes
    /// - Uses `saturating_sub` to prevent overflow
    /// - Uses `min` to ensure it does not exceed the remaining steps
    /// - Operates directly on the array inside the loop to avoid extra function call overhead
    pub fn run_batch(&mut self, max_steps: u64) -> u64 {
        let remaining = self.total.saturating_sub(self.step);
        let steps = max_steps.min(remaining);

        for _ in 0..steps {
            self.state = vdf_step(&self.state);
            self.step += 1;
        }

        steps
    }
}

/// Execute a single VDF computation step.
///
/// Computes a SHA256 hash on the input state and returns the new state.
/// This is the core primitive of VDF computation.
///
/// # Arguments
/// - `state`: 32-byte input state
///
/// # Returns
/// Returns the 32-byte output state.
///
/// # Algorithm
/// output = SHA256(input)
///
/// # Performance
/// - A single SHA256 computation takes approximately 200-500ns (depending on hardware)
/// - May be slightly slower in WebAssembly
pub fn vdf_step(state: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(state);
    let result = hasher.finalize();

    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    /// Test single-step VDF computation.
    ///
    /// Verifies:
    /// 1. Output differs from input
    /// 2. Output length is 32 bytes
    #[wasm_bindgen_test]
    fn test_vdf_step() {
        let state = [0u8; 32];
        let next_state = vdf_step(&state);

        assert_ne!(state, next_state);
        assert_eq!(next_state.len(), 32);
    }

    /// Test basic functionality of the VDF iterator.
    ///
    /// Verifies:
    /// 1. Initial state is correct
    /// 2. Single-step iteration works correctly
    /// 3. Returns `false` correctly after completion
    #[wasm_bindgen_test]
    fn test_vdf_iterator() {
        let seed = [0u8; 32];
        let mut iter = VdfIterator::new(&seed, 100);

        assert_eq!(iter.step(), 0);
        assert_eq!(iter.total(), 100);
        assert!(!iter.is_finished());

        for i in 1..=100 {
            assert!(iter.next());
            assert_eq!(iter.step(), i);
        }

        assert!(iter.is_finished());
        assert!(!iter.next());
    }

    /// Test batch VDF computation.
    ///
    /// Verifies:
    /// 1. Batch execution is correct
    /// 2. Does not exceed total steps
    /// 3. Progress is updated correctly
    #[wasm_bindgen_test]
    fn test_vdf_batch() {
        let seed = [0u8; 32];
        let mut iter = VdfIterator::new(&seed, 1000);

        let steps = iter.run_batch(100);
        assert_eq!(steps, 100);
        assert_eq!(iter.step(), 100);

        let steps = iter.run_batch(1000);
        assert_eq!(steps, 900);
        assert_eq!(iter.step(), 1000);
        assert!(iter.is_finished());
    }

    /// Test VDF determinism.
    ///
    /// Verifies that the same seed produces the same result, ensuring computational determinism.
    #[wasm_bindgen_test]
    fn test_deterministic() {
        let seed = [0u8; 32];

        let mut iter1 = VdfIterator::new(&seed, 100);
        iter1.run_batch(100);

        let mut iter2 = VdfIterator::new(&seed, 100);
        iter2.run_batch(100);

        assert_eq!(iter1.get_state(), iter2.get_state());
    }
}
