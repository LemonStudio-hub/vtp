//! Session State Machine Module
//!
//! Manages the complete lifecycle of a VDF challenge, including:
//! - VDF computation iteration
//! - VRF lottery checking
//! - Checkpoint management
//! - Error handling
//!
//! # Design
//! The `Session` struct encapsulates a complete VDF challenge. It coordinates the VDF iterator
//! and VRF proof generation, performing lottery checks at the appropriate intervals.
//!
//! # Workflow
//! 1. Create a `Session`, initializing the VDF iterator and VRF keypair
//! 2. Call `run_batch` to perform batch VDF computation
//! 3. Automatically perform VRF lottery at checkpoint steps
//! 4. Handle different outcomes based on the returned `BatchResult`

use crate::error::{ErrorHandler, VtpError};
use crate::vdf::VdfIterator;
use crate::vrf;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Winner result structure.
///
/// Contains the winning step number and the VRF proof.
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinnerResult {
    /// The winning step number
    pub step: u64,

    /// The VRF proof
    proof: Vec<u8>,
}

#[wasm_bindgen]
impl WinnerResult {
    /// Get the VRF proof
    #[wasm_bindgen(getter)]
    pub fn proof(&self) -> Vec<u8> {
        self.proof.clone()
    }
}

/// Batch computation result enumeration.
///
/// Represents the result state of a batch VDF computation.
/// Each call to `run_batch` returns a `BatchResult`.
///
/// # Variants
/// - `Progress`: Computation in progress, returns the current step number
/// - `Winner`: A winning ticket is found, returns a `WinnerResult`
/// - `Finished`: VDF computation has completed
/// - `Error`: An error occurred
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchResult {
    /// Computation in progress, contains the current step number
    Progress(u64),

    /// A winning ticket is found, contains a `WinnerResult`
    Winner(WinnerResult),

    /// VDF computation has completed
    Finished = "finished",

    /// An error occurred
    Error(VtpError),
}

/// Session state structure.
///
/// Contains the current state information of a session, used for frontend display and monitoring.
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// The number of VDF steps completed so far
    pub step: u64,

    /// The total number of steps target
    pub total: u64,

    /// Whether the session is active (not yet finished)
    pub is_active: bool,

    /// Whether the session is paused
    pub is_paused: bool,

    /// The number of errors that have occurred
    pub error_count: u32,
}

/// VDF challenge session.
///
/// Manages a complete VDF challenge, including VDF computation, VRF lottery drawing, and checkpoints.
///
/// # Fields
/// - `vdf`: VDF iterator, responsible for sequential hash computation
/// - `keypair`: VRF keypair, used for generating and verifying lottery proofs
/// - `k`: Lottery interval, a VRF lottery draw is performed every k steps
/// - `tau`: Threshold, used to determine if a winning ticket is drawn
/// - `checkpoint_interval`: Checkpoint interval
/// - `error_handler`: Error handler
/// - `is_paused`: Pause flag
///
/// # Examples
/// ```rust
/// use vtp_core::session::{Session, BatchResult};
///
/// let seed = [0u8; 32];
/// let tau = [0u8; 32];
/// let mut session = Session::new(&seed, 100, 0, &tau, 50);
///
/// loop {
///     match session.run_batch(50) {
///         BatchResult::Progress(step) => { /* computation in progress */ },
///         BatchResult::Winner(result) => { /* winning ticket found */ },
///         BatchResult::Finished => break,
///         BatchResult::Error(err) => { /* error occurred */ },
///     }
/// }
/// ```
#[wasm_bindgen]
pub struct Session {
    /// VDF iterator
    vdf: VdfIterator,

    /// VRF keypair
    keypair: vrf::VrfKeypair,

    /// Lottery interval
    k: u64,

    /// Threshold (currently unused, reserved for future extension)
    #[allow(dead_code)]
    tau: Vec<u8>,

    /// Checkpoint interval
    checkpoint_interval: u64,

    /// Error handler
    error_handler: ErrorHandler,

    /// Pause flag
    is_paused: bool,
}

#[wasm_bindgen]
impl Session {
    /// Create a new session.
    ///
    /// # Arguments
    /// - `seed`: Initial seed for VDF computation, at least 32 bytes
    /// - `total`: Total number of VDF steps target
    /// - `k`: Lottery interval
    /// - `tau`: Threshold, 32 bytes
    /// - `checkpoint_interval`: Checkpoint interval
    ///
    /// # Returns
    /// Returns an initialized `Session` instance.
    ///
    /// # Panics
    /// Panics if `seed` or `tau` is less than 32 bytes.
    #[wasm_bindgen(constructor)]
    pub fn new(seed: &[u8], total: u64, k: u64, tau: &[u8], checkpoint_interval: u64) -> Self {
        let vdf = VdfIterator::new(seed, total);
        let keypair = vrf::generate_keypair();
        let error_handler = ErrorHandler::default();

        Self {
            vdf,
            keypair,
            k,
            tau: tau.to_vec(),
            checkpoint_interval,
            error_handler,
            is_paused: false,
        }
    }

    /// Get the current session state.
    ///
    /// # Returns
    /// Returns a `SessionState` struct containing the current step count, total steps, and other information.
    #[wasm_bindgen(getter)]
    pub fn state(&self) -> SessionState {
        SessionState {
            step: self.vdf.step(),
            total: self.vdf.total(),
            is_active: !self.vdf.is_finished(),
            is_paused: self.is_paused,
            error_count: self.error_handler.error_count,
        }
    }

    /// Get the VRF public key.
    ///
    /// # Returns
    /// Returns a 32-byte public key vector.
    #[wasm_bindgen(getter)]
    pub fn public_key(&self) -> Vec<u8> {
        self.keypair.public_key()
    }

    /// Pause the session.
    ///
    /// Once paused, `run_batch` will return the current progress without executing new computations.
    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    /// Resume the session.
    ///
    /// Once resumed, `run_batch` will continue executing VDF computations.
    pub fn resume(&mut self) {
        self.is_paused = false;
    }

    /// Check whether the session is paused.
    ///
    /// # Returns
    /// - `true`: The session is paused
    /// - `false`: The session is running
    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    /// Execute batch VDF computation.
    ///
    /// Executes up to `max_steps` steps of VDF computation, and performs VRF lottery drawing at checkpoints.
    ///
    /// # Arguments
    /// - `max_steps`: Maximum number of steps to execute
    ///
    /// # Returns
    /// Returns a `BatchResult` enum indicating the computation result.
    ///
    /// # Workflow
    /// 1. Check if the session has finished or is paused
    /// 2. Execute VDF batch computation
    /// 3. Generate VRF proof at checkpoint steps
    /// 4. Determine if a winning ticket is drawn
    /// 5. Return the corresponding result
    pub fn run_batch(&mut self, max_steps: u64) -> BatchResult {
        if self.vdf.is_finished() {
            return BatchResult::Error(VtpError::SessionFinished);
        }

        if self.is_paused {
            return BatchResult::Progress(self.vdf.step());
        }

        let _steps = self.vdf.run_batch(max_steps);
        let current_step = self.vdf.step();

        if self.is_checkpoint_step(current_step) {
            let message = current_step.to_be_bytes();
            let proof = vrf::prove(&self.keypair.secret_key(), &message);

            if self.should_trigger_vrf(current_step) {
                let winner = WinnerResult { step: current_step, proof };
                return BatchResult::Winner(winner);
            }
        }

        if self.vdf.is_finished() {
            BatchResult::Finished
        } else {
            BatchResult::Progress(current_step)
        }
    }

    /// Check if the given step is a checkpoint step.
    ///
    /// # Arguments
    /// - `step`: The current step number
    ///
    /// # Returns
    /// - `true`: It is a checkpoint step
    /// - `false`: It is not a checkpoint step
    #[allow(clippy::manual_is_multiple_of)]
    fn is_checkpoint_step(&self, step: u64) -> bool {
        step % self.checkpoint_interval == 0
    }

    /// Check whether VRF lottery drawing should be triggered.
    ///
    /// # Arguments
    /// - `step`: The current step number
    ///
    /// # Returns
    /// - `true`: VRF should be triggered
    /// - `false`: VRF should not be triggered
    #[allow(clippy::manual_is_multiple_of)]
    fn should_trigger_vrf(&self, step: u64) -> bool {
        if self.k == 0 {
            return false;
        }

        step % self.k == 0
    }

    /// Get checkpoint data.
    ///
    /// Generates checkpoint data for persistence, containing the current step number and VDF state.
    ///
    /// # Returns
    /// Returns the serialized checkpoint data.
    ///
    /// # Data Format
    /// [8-byte step count (big-endian)] [32-byte VDF state]
    pub fn get_checkpoint_data(&self) -> Vec<u8> {
        let state = self.vdf.get_state();
        let step = self.vdf.step();

        let mut data = Vec::new();
        data.extend_from_slice(&step.to_be_bytes());
        data.extend_from_slice(&state);
        data
    }

    /// Verify a winning proof.
    ///
    /// Verifies whether the given VRF proof is valid.
    ///
    /// # Arguments
    /// - `step`: The winning step number
    /// - `proof`: The VRF proof
    ///
    /// # Returns
    /// - `true`: The proof is valid
    /// - `false`: The proof is invalid
    pub fn verify_winner(&self, step: u64, proof: &[u8]) -> bool {
        let message = step.to_be_bytes();
        vrf::verify(&self.keypair.public_key(), &message, proof)
    }
}
