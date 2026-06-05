//! Tests for the VTP error handling subsystem.
//!
//! This module validates the behavior of [`VtpError`] variants and the
//! [`ErrorHandler`] utility, covering:
//! - Display formatting for each error variant
//! - Equality and inequality semantics
//! - Error handler creation (default and custom)
//! - Error accumulation, reset, and retry logic
//! - Clone and Debug derive correctness

#[cfg(test)]
mod tests {
    use vtp_core::error::{ErrorHandler, VtpError};

    /// Tests that every [`VtpError`] variant produces the correct human-readable
    /// string when formatted with `Display`.
    ///
    /// - `InvalidInput` displays as `"Invalid input provided"`
    /// - `InvalidState` displays as `"Invalid state encountered"`
    /// - `ComputationFailed` displays as `"Computation failed"`
    /// - `CheckpointFailed` displays as `"Checkpoint save/load failed"`
    /// - `SessionFinished` displays as `"Session has already finished"`
    /// - `SessionNotStarted` displays as `"Session has not started yet"`
    #[test]
    fn test_vtp_error_display() {
        assert_eq!(format!("{}", VtpError::InvalidInput), "Invalid input provided");
        assert_eq!(format!("{}", VtpError::InvalidState), "Invalid state encountered");
        assert_eq!(format!("{}", VtpError::ComputationFailed), "Computation failed");
        assert_eq!(format!("{}", VtpError::CheckpointFailed), "Checkpoint save/load failed");
        assert_eq!(format!("{}", VtpError::SessionFinished), "Session has already finished");
        assert_eq!(format!("{}", VtpError::SessionNotStarted), "Session has not started yet");
    }

    /// Tests that [`VtpError`] variants implement `PartialEq` correctly.
    ///
    /// - Two instances of the same variant are always equal
    /// - Different variants are never equal, including across logically unrelated pairs
    /// - Ensures the derived `PartialEq` implementation is exhaustive over all variants
    #[test]
    fn test_vtp_error_equality() {
        assert_eq!(VtpError::InvalidInput, VtpError::InvalidInput);
        assert_eq!(VtpError::InvalidState, VtpError::InvalidState);
        assert_eq!(VtpError::ComputationFailed, VtpError::ComputationFailed);
        assert_eq!(VtpError::CheckpointFailed, VtpError::CheckpointFailed);
        assert_eq!(VtpError::SessionFinished, VtpError::SessionFinished);
        assert_eq!(VtpError::SessionNotStarted, VtpError::SessionNotStarted);

        assert_ne!(VtpError::InvalidInput, VtpError::InvalidState);
        assert_ne!(VtpError::ComputationFailed, VtpError::CheckpointFailed);
        assert_ne!(VtpError::SessionFinished, VtpError::SessionNotStarted);
        assert_ne!(VtpError::InvalidInput, VtpError::SessionFinished);
        assert_ne!(VtpError::InvalidState, VtpError::ComputationFailed);
        assert_ne!(VtpError::CheckpointFailed, VtpError::SessionNotStarted);
    }

    /// Tests that [`ErrorHandler::default`] initializes with sensible defaults.
    ///
    /// - No previous error recorded (`last_error` is `None`)
    /// - Error count starts at zero
    /// - Maximum retries defaults to 3
    #[test]
    fn test_error_handler_default() {
        let handler = ErrorHandler::default();
        assert!(handler.last_error.is_none());
        assert_eq!(handler.error_count, 0);
        assert_eq!(handler.max_retries, 3);
    }

    /// Tests that [`ErrorHandler::new`] creates a handler with a custom retry limit.
    ///
    /// - `last_error` is initially `None`
    /// - Error count starts at zero
    /// - `max_retries` is set to the provided value (5 in this case)
    #[test]
    fn test_error_handler_new() {
        let handler = ErrorHandler::new(5);
        assert!(handler.last_error.is_none());
        assert_eq!(handler.error_count, 0);
        assert_eq!(handler.max_retries, 5);
    }

    /// Tests the core error-handling flow of [`ErrorHandler::handle_error`].
    ///
    /// - Returns `true` (retryable) when the error count is below `max_retries`
    /// - Increments `error_count` on each call
    /// - Tracks the most recent error in `last_error`
    /// - Returns `false` when the error count reaches `max_retries` (retries exhausted)
    #[test]
    fn test_error_handler_handle_error() {
        let mut handler = ErrorHandler::new(3);

        let result = handler.handle_error(VtpError::ComputationFailed);
        assert!(result);
        assert_eq!(handler.error_count, 1);
        assert_eq!(handler.last_error, Some(VtpError::ComputationFailed));

        let result = handler.handle_error(VtpError::InvalidInput);
        assert!(result);
        assert_eq!(handler.error_count, 2);

        let result = handler.handle_error(VtpError::InvalidState);
        assert!(!result); // max_retries exhausted — no more retries allowed
        assert_eq!(handler.error_count, 3);
        assert_eq!(handler.last_error, Some(VtpError::InvalidState));
    }

    /// Tests that [`ErrorHandler::reset`] clears all accumulated error state.
    ///
    /// - After recording multiple errors, calling `reset()` clears `last_error` to `None`
    /// - The `error_count` is reset to 0
    /// - The handler is effectively returned to its initial state
    #[test]
    fn test_error_handler_reset() {
        let mut handler = ErrorHandler::new(3);

        handler.handle_error(VtpError::ComputationFailed);
        handler.handle_error(VtpError::CheckpointFailed);
        assert_eq!(handler.error_count, 2);
        assert!(handler.last_error.is_some());

        handler.reset();
        assert!(handler.last_error.is_none());
        assert_eq!(handler.error_count, 0);
    }

    /// Tests that [`ErrorHandler::can_retry`] returns the correct boolean based on
    /// remaining retry budget.
    ///
    /// - Returns `true` when no errors have been recorded
    /// - Continues returning `true` as long as `error_count < max_retries`
    /// - Returns `false` once `error_count` equals `max_retries`
    #[test]
    fn test_error_handler_can_retry() {
        let mut handler = ErrorHandler::new(3);

        assert!(handler.can_retry());

        handler.handle_error(VtpError::InvalidInput);
        assert!(handler.can_retry());

        handler.handle_error(VtpError::InvalidState);
        assert!(handler.can_retry());

        handler.handle_error(VtpError::ComputationFailed);
        assert!(!handler.can_retry());
    }

    /// Tests that the error handler correctly tracks multiple sequential errors
    /// when given a large retry budget.
    ///
    /// - Each call to `handle_error` updates `last_error` to the latest error variant
    /// - The `error_count` accumulates across all six distinct error variants
    /// - Uses `max_retries = 10` to ensure all errors are accepted without exhaustion
    #[test]
    fn test_error_handler_multiple_errors() {
        let mut handler = ErrorHandler::new(10);

        handler.handle_error(VtpError::InvalidInput);
        assert_eq!(handler.last_error, Some(VtpError::InvalidInput));

        handler.handle_error(VtpError::InvalidState);
        assert_eq!(handler.last_error, Some(VtpError::InvalidState));

        handler.handle_error(VtpError::ComputationFailed);
        assert_eq!(handler.last_error, Some(VtpError::ComputationFailed));

        handler.handle_error(VtpError::CheckpointFailed);
        assert_eq!(handler.last_error, Some(VtpError::CheckpointFailed));

        handler.handle_error(VtpError::SessionFinished);
        assert_eq!(handler.last_error, Some(VtpError::SessionFinished));

        handler.handle_error(VtpError::SessionNotStarted);
        assert_eq!(handler.last_error, Some(VtpError::SessionNotStarted));
        assert_eq!(handler.error_count, 6);
    }

    /// Tests the exact boundary condition where `error_count` reaches `max_retries`.
    ///
    /// - With `max_retries = 3`, the first two errors return `true` (retryable)
    /// - The third error returns `false` (retries exhausted)
    /// - The final error count equals `max_retries` exactly
    /// - The last error is still recorded even when retries are exhausted
    #[test]
    fn test_error_handler_exact_boundary() {
        let mut handler = ErrorHandler::new(3);

        assert!(handler.handle_error(VtpError::InvalidInput));
        assert!(handler.handle_error(VtpError::InvalidState));
        assert!(!handler.handle_error(VtpError::ComputationFailed)); // 3rd error hits the max_retries limit

        assert_eq!(handler.error_count, 3);
        assert_eq!(handler.last_error, Some(VtpError::ComputationFailed));
    }

    /// Tests that [`VtpError`] implements `Clone` correctly for all variants.
    ///
    /// - Each variant, when cloned, produces a value equal to the original
    /// - Validates the derived `Clone` implementation is exhaustive over all variants
    #[test]
    fn test_vtp_error_clone() {
        let errors = vec![
            VtpError::InvalidInput,
            VtpError::InvalidState,
            VtpError::ComputationFailed,
            VtpError::CheckpointFailed,
            VtpError::SessionFinished,
            VtpError::SessionNotStarted,
        ];

        for error in errors {
            let cloned = error.clone();
            assert_eq!(error, cloned);
        }
    }

    /// Tests that [`VtpError`] produces the expected debug representation
    /// when formatted with `Debug`.
    ///
    /// - Each variant outputs its variant name (e.g., `"InvalidInput"`)
    /// - Validates the derived `Debug` implementation is exhaustive over all variants
    #[test]
    fn test_vtp_error_debug() {
        assert_eq!(format!("{:?}", VtpError::InvalidInput), "InvalidInput");
        assert_eq!(format!("{:?}", VtpError::InvalidState), "InvalidState");
        assert_eq!(format!("{:?}", VtpError::ComputationFailed), "ComputationFailed");
        assert_eq!(format!("{:?}", VtpError::CheckpointFailed), "CheckpointFailed");
        assert_eq!(format!("{:?}", VtpError::SessionFinished), "SessionFinished");
        assert_eq!(format!("{:?}", VtpError::SessionNotStarted), "SessionNotStarted");
    }
}
