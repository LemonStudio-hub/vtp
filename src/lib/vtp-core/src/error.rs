//! Error handling module.
//!
//! Defines the error types and error handler for the VTP library.
//! Provides a unified error handling mechanism with support for error retry and recovery.
//!
//! # Error Types
//! - `InvalidInput`: Invalid input parameters
//! - `InvalidState`: Invalid internal state
//! - `ComputationFailed`: Computation failure
//! - `CheckpointFailed`: Checkpoint save/load failure
//! - `SessionFinished`: Session has already finished
//! - `SessionNotStarted`: Session has not started
//!
//! # Error Handling Strategy
//! Uses the `ErrorHandler` struct to manage error state, supporting:
//! - Error counting
//! - Maximum retry count
//! - Error recovery determination

use serde::{Deserialize, Serialize};
use std::fmt;
use wasm_bindgen::prelude::*;

/// VTP error enumeration.
///
/// Defines all possible error types that can occur in the VTP library.
/// Each variant represents a specific error condition.
///
/// # Usage Scenarios
/// - Returned as error state in `Session::run_batch`
/// - Recorded in `ErrorHandler`
/// - Frontend displays different error messages based on error type
#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VtpError {
    /// Invalid input parameters.
    ///
    /// Returned when the provided parameters do not meet requirements, for example:
    /// - Seed length is less than 32 bytes
    /// - Step count is 0
    InvalidInput,

    /// Invalid internal state.
    ///
    /// Returned when the internal state is inconsistent, for example:
    /// - VDF iterator state is corrupted
    /// - Session state is abnormal
    InvalidState,

    /// Computation failure.
    ///
    /// Returned when an error occurs during VDF or VRF computation.
    ComputationFailed,

    /// Checkpoint save/load failure.
    ///
    /// Returned when an IndexedDB operation fails.
    CheckpointFailed,

    /// Session has already finished.
    ///
    /// Returned when attempting to perform an operation on a finished session.
    SessionFinished,

    /// Session has not started.
    ///
    /// Returned when attempting to perform an operation on a session that has not started.
    SessionNotStarted,
}

/// Implements the `Display` trait for formatted error message output.
///
/// Provides human-readable error descriptions for logging and frontend display.
impl fmt::Display for VtpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VtpError::InvalidInput => write!(f, "Invalid input provided"),
            VtpError::InvalidState => write!(f, "Invalid state encountered"),
            VtpError::ComputationFailed => write!(f, "Computation failed"),
            VtpError::CheckpointFailed => write!(f, "Checkpoint save/load failed"),
            VtpError::SessionFinished => write!(f, "Session has already finished"),
            VtpError::SessionNotStarted => write!(f, "Session has not started yet"),
        }
    }
}

/// Implements the `Error` trait, allowing `VtpError` to be used as a standard error type.
impl std::error::Error for VtpError {}

/// Error handler.
///
/// Manages error state with support for error counting and retry mechanisms.
///
/// # Fields
/// - `last_error`: The most recently occurred error
/// - `error_count`: Cumulative error count
/// - `max_retries`: Maximum number of retries
///
/// # Examples
/// ```rust
/// use vtp_core::error::{ErrorHandler, VtpError};
///
/// let mut handler = ErrorHandler::new(3);
///
/// // Handle an error
/// let can_continue = handler.handle_error(VtpError::ComputationFailed);
/// if !can_continue {
///     println!("Too many errors, stopping...");
/// }
///
/// // Reset error count
/// handler.reset();
/// ```
#[derive(Debug, Clone)]
pub struct ErrorHandler {
    /// The most recently occurred error
    pub last_error: Option<VtpError>,

    /// Cumulative error count
    pub error_count: u32,

    /// Maximum number of retries
    pub max_retries: u32,
}

/// Default implementation.
///
/// Creates an `ErrorHandler` with a maximum retry count of 3.
impl Default for ErrorHandler {
    fn default() -> Self {
        Self { last_error: None, error_count: 0, max_retries: 3 }
    }
}

impl ErrorHandler {
    /// Create a new error handler.
    ///
    /// # Arguments
    /// - `max_retries`: Maximum number of retries
    ///
    /// # Returns
    /// Returns an initialized `ErrorHandler` instance.
    pub fn new(max_retries: u32) -> Self {
        Self { last_error: None, error_count: 0, max_retries }
    }

    /// Handle an error.
    ///
    /// Records the error and updates the error count.
    ///
    /// # Arguments
    /// - `error`: The error that occurred
    ///
    /// # Returns
    /// - `true`: Execution can continue (has not exceeded max retries)
    /// - `false`: Execution should stop (has exceeded max retries)
    ///
    /// # Note
    /// Even when returning `true`, you should consider adding an appropriate delay before retrying.
    pub fn handle_error(&mut self, error: VtpError) -> bool {
        self.last_error = Some(error);
        self.error_count += 1;

        if self.error_count >= self.max_retries {
            return false;
        }

        true
    }

    /// Reset the error state.
    ///
    /// Clears error records and count, typically called after error recovery.
    pub fn reset(&mut self) {
        self.last_error = None;
        self.error_count = 0;
    }

    /// Check whether retrying is allowed.
    ///
    /// # Returns
    /// - `true`: Retry is allowed (error count has not exceeded the limit)
    /// - `false`: Should not retry (has reached the maximum retry count)
    pub fn can_retry(&self) -> bool {
        self.error_count < self.max_retries
    }
}
