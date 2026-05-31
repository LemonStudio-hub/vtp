#[cfg(test)]
mod tests {
    use vtp_core::error::{ErrorHandler, VtpError};

    #[test]
    fn test_vtp_error_display() {
        assert_eq!(format!("{}", VtpError::InvalidInput), "Invalid input provided");
        assert_eq!(format!("{}", VtpError::InvalidState), "Invalid state encountered");
        assert_eq!(format!("{}", VtpError::ComputationFailed), "Computation failed");
        assert_eq!(format!("{}", VtpError::CheckpointFailed), "Checkpoint save/load failed");
        assert_eq!(format!("{}", VtpError::SessionFinished), "Session has already finished");
        assert_eq!(format!("{}", VtpError::SessionNotStarted), "Session has not started yet");
    }

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

    #[test]
    fn test_error_handler_default() {
        let handler = ErrorHandler::default();
        assert!(handler.last_error.is_none());
        assert_eq!(handler.error_count, 0);
        assert_eq!(handler.max_retries, 3);
    }

    #[test]
    fn test_error_handler_new() {
        let handler = ErrorHandler::new(5);
        assert!(handler.last_error.is_none());
        assert_eq!(handler.error_count, 0);
        assert_eq!(handler.max_retries, 5);
    }

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
        assert!(!result);
        assert_eq!(handler.error_count, 3);
        assert_eq!(handler.last_error, Some(VtpError::InvalidState));
    }

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

    #[test]
    fn test_error_handler_exact_boundary() {
        let mut handler = ErrorHandler::new(3);

        assert!(handler.handle_error(VtpError::InvalidInput));
        assert!(handler.handle_error(VtpError::InvalidState));
        assert!(!handler.handle_error(VtpError::ComputationFailed));

        assert_eq!(handler.error_count, 3);
        assert_eq!(handler.last_error, Some(VtpError::ComputationFailed));
    }

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
