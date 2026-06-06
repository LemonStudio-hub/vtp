//! Tests for the VTP session lifecycle and batch processing logic.
//!
//! This module validates the behavior of [`Session`] across its full lifecycle,
//! covering:
//! - Session creation and initial state
//! - Batch processing progress and completion
//! - Pause and resume semantics
//! - Checkpoint data encoding and retrieval
//! - Winner detection and proof verification
//! - Public key access
//! - State transitions throughout execution
//! - Zero-k (no winner) scenarios
//! - Deterministic execution guarantees
//!
//! A helper function [`create_session`] is provided to reduce boilerplate.

use vtp_core::error::VtpError;
use vtp_core::session::{BatchResult, Session};

/// Fixed seed used across all tests for reproducibility.
const SEED: [u8; 32] = [42u8; 32];

/// Fixed tau (target) value used across all tests.
///
/// Set to all-0xFF so that any VRF output (a 32-byte hash, almost certainly
/// less than 0xFFFF...FF) will satisfy the `vrf_output < tau` winning
/// condition.  This makes the winner-detection test deterministic.
const TAU: [u8; 32] = [0xFFu8; 32];

/// Creates a [`Session`] with the shared test constants (`SEED` and `TAU`).
///
/// # Arguments
///
/// * `total` - Total number of steps in the session
/// * `k` - Winner threshold interval (0 disables winner detection)
/// * `checkpoint_interval` - Steps between automatic checkpoints
fn create_session(total: u64, k: u64, checkpoint_interval: u64) -> Session {
    Session::new(&SEED, total, k, &TAU, checkpoint_interval)
}

/// Tests that a newly created [`Session`] starts in a clean initial state.
///
/// - Step counter is 0
/// - Total steps matches the configured value
/// - Session is active but not paused
/// - No errors have been recorded
#[test]
fn test_session_new() {
    let session = create_session(1000, 10, 10);
    let state = session.state();

    assert_eq!(state.step, 0);
    assert_eq!(state.total, 1000);
    assert!(state.is_active);
    assert!(!state.is_paused);
    assert_eq!(state.error_count, 0);
}

/// Tests that `run_batch` advances the session by the requested number of steps.
///
/// - A batch of 10 steps returns `BatchResult::Progress(10)`
/// - The returned step count matches the requested batch size
/// - The batch size (10) does not exceed the winner interval (20), so no winner is triggered
#[test]
fn test_session_run_batch_progress() {
    let mut session = create_session(1000, 20, 20);
    let result = session.run_batch(10);

    match result {
        BatchResult::Progress(step) => assert_eq!(step, 10),
        other => panic!("Expected Progress, got {:?}", other),
    }
}

/// Tests that the session transitions to `BatchResult::Finished` when all steps
/// are consumed.
///
/// - First batch of 50 returns `Progress(50)`
/// - Second batch of 50 returns `Finished` (total of 100 steps reached)
#[test]
fn test_session_run_batch_finished() {
    let mut session = create_session(100, 0, 50);

    let result1 = session.run_batch(50);
    assert!(matches!(result1, BatchResult::Progress(50)));

    let result2 = session.run_batch(50);
    assert!(matches!(result2, BatchResult::Finished));
}

/// Tests the pause and resume lifecycle of a [`Session`].
///
/// - While paused, `run_batch` returns `Progress(0)` without advancing
/// - The step counter remains at 0 while paused
/// - After resuming, `run_batch` advances normally
/// - Validates that pause/resume correctly gates batch execution
#[test]
fn test_session_pause_resume() {
    let mut session = create_session(1000, 20, 20);

    session.pause();

    let result = session.run_batch(100);
    match result {
        BatchResult::Progress(step) => assert_eq!(step, 0), // no progress while paused
        other => panic!("Expected Progress while paused, got {:?}", other),
    }

    let state = session.state();
    assert_eq!(state.step, 0);

    session.resume();

    let result = session.run_batch(10);
    match result {
        BatchResult::Progress(step) => assert_eq!(step, 10),
        other => panic!("Expected Progress after resume, got {:?}", other),
    }
}

/// Tests that calling `run_batch` on a finished session returns an error.
///
/// - After the session completes (total steps consumed), the first call returns `Finished`
/// - Subsequent calls return `BatchResult::Error(VtpError::SessionFinished)`
/// - Prevents further computation on an already-completed session
#[test]
fn test_session_run_batch_after_finished() {
    let mut session = create_session(100, 0, 50);

    session.run_batch(50);
    let result = session.run_batch(50);
    assert!(matches!(result, BatchResult::Finished));

    let result = session.run_batch(10);
    match result {
        BatchResult::Error(err) => assert_eq!(err, VtpError::SessionFinished),
        other => panic!("Expected Error(SessionFinished), got {:?}", other),
    }
}

/// Tests that checkpoint data encodes the current session state correctly.
///
/// - After 10 steps, checkpoint data contains at least 9 bytes
/// - First 8 bytes encode the current step as a big-endian `u64`
/// - Remaining bytes represent the serialised class group element (VDF state)
#[test]
fn test_session_checkpoint_data() {
    let mut session = create_session(1000, 20, 20);
    session.run_batch(10);

    let data = session.get_checkpoint_data();
    assert!(data.len() > 8, "checkpoint data must contain step + state");

    let step_bytes: [u8; 8] = data[0..8].try_into().unwrap();
    let step = u64::from_be_bytes(step_bytes); // decode step from big-endian
    assert_eq!(step, 10);

    // The remaining bytes are the serialised class group element
    let state_bytes = &data[8..];
    assert!(!state_bytes.is_empty(), "VDF state must not be empty");
}

/// Tests that a winner is correctly detected at the expected step interval
/// and that the winner's proof can be verified.
///
/// - With `k = 20`, a `Winner` result appears at step 20
/// - The winner's proof is non-empty
/// - `verify_winner` confirms the proof is valid for the winner's step
#[test]
fn test_session_verify_winner() {
    let mut session = create_session(1000, 20, 20);

    session.run_batch(10);

    let result = session.run_batch(10);
    match result {
        BatchResult::Winner(winner) => {
            assert_eq!(winner.step, 20);
            assert!(!winner.proof().is_empty());
            assert!(session.verify_winner(winner.step, &winner.proof()));
        }
        other => panic!("Expected Winner at step 20, got {:?}", other),
    }
}

/// Tests that the session exposes a valid public key of the expected length.
///
/// - The public key is exactly 32 bytes
/// - The key is accessible immediately after session creation (no computation required)
#[test]
fn test_session_public_key() {
    let session = create_session(1000, 10, 10);
    let pk = session.public_key();
    assert_eq!(pk.len(), 32);
}

/// Tests that the session state is updated correctly after each batch execution.
///
/// - Initial state: step = 0, active, not paused, no errors
/// - After first batch (50 steps): step = 50, still active
/// - After second batch (50 steps): step = 100, no longer active (finished)
/// - Validates the full state transition lifecycle from start to completion
#[test]
fn test_session_state_updates() {
    let mut session = create_session(100, 1, 50);

    let state = session.state();
    assert_eq!(state.step, 0);
    assert_eq!(state.total, 100);
    assert!(state.is_active);
    assert!(!state.is_paused);
    assert_eq!(state.error_count, 0);

    session.run_batch(50);

    let state = session.state();
    assert_eq!(state.step, 50);
    assert_eq!(state.total, 100);
    assert!(state.is_active);

    session.run_batch(50);

    let state = session.state();
    assert_eq!(state.step, 100);
    assert_eq!(state.total, 100);
    assert!(!state.is_active);
}

/// Tests the `is_paused` query method across multiple pause/resume cycles.
///
/// - A freshly created session is not paused
/// - `pause()` makes `is_paused()` return `true`
/// - `resume()` makes `is_paused()` return `false`
/// - Multiple pause/resume toggles work correctly
/// - Running a batch while paused keeps the session in the paused state
#[test]
fn test_session_is_paused() {
    let mut session = create_session(1000, 10, 10);

    assert!(!session.is_paused());

    session.pause();
    assert!(session.is_paused());

    session.resume();
    assert!(!session.is_paused());

    session.pause();
    assert!(session.is_paused());

    session.run_batch(10);
    let state = session.state();
    assert_eq!(state.step, 0);
    assert!(state.is_paused);
}

/// Tests that a session with `k = 0` never produces a winner.
///
/// - With the winner threshold interval set to 0, winner detection is disabled
/// - The session runs to completion (`Finished`) without emitting any `Winner` result
/// - Ensures the zero-k edge case is handled gracefully
#[test]
fn test_session_zero_k_no_winner() {
    let mut session = create_session(1000, 0, 10);
    let mut saw_winner = false;

    // Run batches until the session finishes, tracking whether any Winner appears
    loop {
        match session.run_batch(100) {
            BatchResult::Progress(_) => {}
            BatchResult::Winner(_) => {
                saw_winner = true;
                break;
            }
            BatchResult::Finished => break,
            BatchResult::Error(err) => panic!("Unexpected error: {:?}", err),
        }
    }

    assert!(!saw_winner);
}

/// Tests that two sessions created with identical parameters produce identical
/// results, proving deterministic execution.
///
/// - Both sessions advance by the same step count at each batch
/// - `BatchResult` variants match at every iteration
/// - Final checkpoint data is byte-for-byte identical
#[test]
fn test_session_deterministic() {
    let mut session1 = Session::new(&SEED, 100, 0, &TAU, 10);
    let mut session2 = Session::new(&SEED, 100, 0, &TAU, 10);

    for _ in 0..10 {
        let r1 = session1.run_batch(10);
        let r2 = session2.run_batch(10);

        match (&r1, &r2) {
            (BatchResult::Progress(s1), BatchResult::Progress(s2)) => assert_eq!(s1, s2),
            (BatchResult::Finished, BatchResult::Finished) => {}
            _ => panic!("Mismatched results: {:?} vs {:?}", r1, r2),
        }
    }

    let cp1 = session1.get_checkpoint_data();
    let cp2 = session2.get_checkpoint_data();
    assert_eq!(cp1, cp2);
}
