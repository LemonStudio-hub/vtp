use vtp_core::error::VtpError;
use vtp_core::session::{BatchResult, Session};

const SEED: [u8; 32] = [42u8; 32];
const TAU: [u8; 32] = [0u8; 32];

fn create_session(total: u64, k: u64, checkpoint_interval: u64) -> Session {
    Session::new(&SEED, total, k, &TAU, checkpoint_interval)
}

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

#[test]
fn test_session_run_batch_progress() {
    let mut session = create_session(1000, 20, 20);
    let result = session.run_batch(10);

    match result {
        BatchResult::Progress(step) => assert_eq!(step, 10),
        other => panic!("Expected Progress, got {:?}", other),
    }
}

#[test]
fn test_session_run_batch_finished() {
    let mut session = create_session(100, 0, 50);

    let result1 = session.run_batch(50);
    assert!(matches!(result1, BatchResult::Progress(50)));

    let result2 = session.run_batch(50);
    assert!(matches!(result2, BatchResult::Finished));
}

#[test]
fn test_session_pause_resume() {
    let mut session = create_session(1000, 20, 20);

    session.pause();

    let result = session.run_batch(100);
    match result {
        BatchResult::Progress(step) => assert_eq!(step, 0),
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

#[test]
fn test_session_checkpoint_data() {
    let mut session = create_session(1000, 20, 20);
    session.run_batch(10);

    let data = session.get_checkpoint_data();
    assert_eq!(data.len(), 40);

    let step_bytes: [u8; 8] = data[0..8].try_into().unwrap();
    let step = u64::from_be_bytes(step_bytes);
    assert_eq!(step, 10);

    let state_bytes = &data[8..40];
    assert_eq!(state_bytes.len(), 32);
}

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

#[test]
fn test_session_public_key() {
    let session = create_session(1000, 10, 10);
    let pk = session.public_key();
    assert_eq!(pk.len(), 32);
}

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

#[test]
fn test_session_zero_k_no_winner() {
    let mut session = create_session(1000, 0, 10);
    let mut saw_winner = false;

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
