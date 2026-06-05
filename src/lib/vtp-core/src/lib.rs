//! VTP Core Library - Verifiable Time Proof Core Implementation
//!
//! This library provides the core functionality of the VTP protocol, including:
//! - VDF (Verifiable Delay Function)
//! - VRF (Verifiable Random Function)
//! - Session state machine management
//!
//! The library is designed to compile to WebAssembly and run in browser environments.

#![allow(clippy::manual_is_multiple_of)]

pub mod error;
pub mod session;
pub mod utils;
pub mod vdf;
pub mod vrf;

use wasm_bindgen::prelude::*;

/// Initialize the panic hook for capturing and displaying panic information in WebAssembly.
///
/// This function is automatically called when the module is loaded. It sets up
/// `console_error_panic_hook` so that panic messages are written to the browser console.
///
/// # Note
/// - Only takes effect when the `console_error_panic_hook` feature is enabled
/// - Uses `set_once()` to ensure initialization occurs only once
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Convenience function for executing a single VDF computation.
///
/// Takes a seed and a step count, performs the specified number of VDF iterations,
/// and returns the final state. Primarily used for performance benchmarking and quick verification.
///
/// # Arguments
/// - `seed`: A 32-byte seed used as the initial state for VDF computation
/// - `steps`: The number of VDF iterations to execute
///
/// # Returns
/// Returns the 32-byte final state
///
/// # Panics
/// Panics if the seed is less than 32 bytes.
///
/// # Note
/// - This function is synchronous and will block the calling thread for long-running computations
///
/// # Examples
/// ```rust
/// use vtp_core::run_single_vdf;
/// let seed = [0u8; 32];
/// let result = run_single_vdf(&seed, 1000);
/// assert_eq!(result.len(), 32);
/// ```
#[wasm_bindgen]
pub fn run_single_vdf(seed: &[u8], steps: u32) -> Vec<u8> {
    let mut state = [0u8; 32];
    state.copy_from_slice(&seed[..32]);

    for _ in 0..steps {
        state = vdf::vdf_step(&state);
    }

    state.to_vec()
}

/// Get the complexity value of a single VDF computation step.
///
/// Returns the relative computational cost of a single VDF iteration.
/// The current implementation returns 1, representing a single SHA256 iteration.
///
/// # Returns
/// Returns the complexity value of a single VDF computation step
#[wasm_bindgen]
pub fn vdf_step_count() -> u32 {
    1
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    /// Test the basic functionality of `run_single_vdf`.
    ///
    /// Verifies:
    /// 1. The function executes without errors
    /// 2. The returned result is 32 bytes long
    #[wasm_bindgen_test]
    fn test_run_single_vdf() {
        let seed = [0u8; 32];
        let result = run_single_vdf(&seed, 100);
        assert_eq!(result.len(), 32);
    }
}
