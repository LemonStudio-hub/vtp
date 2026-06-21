//! Integration tests for the VTP consensus module.
//!
//! Validates the behavior of the consensus engine across its full lifecycle,
//! covering:
//! - Engine creation with various validator set sizes
//! - Threshold computation for BFT quorum
//! - Block header serialization and deserialization
//! - Block chain integrity and tamper detection
//! - Leader election verification
//! - Full round lifecycle (propose → prevote → precommit → commit)

use vtp_core::consensus::{
    compute_threshold, verify_leader_election, BlockHeader, ConsensusEngine, ConsensusPhase,
    VotePhase,
};
use vtp_core::vrf;

/// Fixed tau (target) value used across all tests.
const TAU: [u8; 32] = [0xFFu8; 32];

/// Create a set of `n` VRF keypairs.
fn create_validators(n: usize) -> Vec<vrf::VrfKeypair> {
    (0..n).map(|_| vrf::generate_keypair()).collect()
}

/// Extract public keys from keypairs.
fn pubkeys(validators: &[vrf::VrfKeypair]) -> Vec<Vec<u8>> {
    validators.iter().map(|kp| kp.public_key()).collect()
}

/// Create a ConsensusEngine for testing.
fn create_engine(validators: &[vrf::VrfKeypair], index: usize) -> ConsensusEngine {
    let pks = pubkeys(validators);
    ConsensusEngine::new_native(
        &validators[index].secret_key(),
        &validators[index].public_key(),
        pks,
        &TAU,
    )
    .unwrap()
}

/// Serialize a vote to bytes (matching the engine's wire format).
fn serialize_vote(
    round: u64,
    phase: VotePhase,
    block_hash: Option<&[u8]>,
    voter: &[u8],
) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&round.to_be_bytes());
    data.push(phase as u8);

    match block_hash {
        Some(hash) => {
            data.push(1);
            data.extend_from_slice(&(hash.len() as u32).to_be_bytes());
            data.extend_from_slice(hash);
        }
        None => {
            data.push(0);
        }
    }

    data.extend_from_slice(&(voter.len() as u32).to_be_bytes());
    data.extend_from_slice(voter);
    data
}

// ─── Threshold Tests ─────────────────────────────────────────────

#[test]
fn test_threshold_single_validator() {
    assert_eq!(compute_threshold(1), 1);
}

#[test]
fn test_threshold_four_validators() {
    assert_eq!(compute_threshold(4), 3);
}

#[test]
fn test_threshold_seven_validators() {
    assert_eq!(compute_threshold(7), 5);
}

#[test]
fn test_threshold_ten_validators() {
    assert_eq!(compute_threshold(10), 7);
}

#[test]
fn test_threshold_zero_validators() {
    assert_eq!(compute_threshold(0), 0);
}

#[test]
fn test_threshold_two_validators() {
    assert_eq!(compute_threshold(2), 1);
}

// ─── Block Header Tests ──────────────────────────────────────────

#[test]
fn test_block_header_roundtrip() {
    let block = BlockHeader::new(
        42,
        vec![0xAA; 32],
        vec![0xBB; 32],
        vec![0xCC; 80],
        vec![0xDD; 32],
        1234567890,
    );

    let bytes = block.to_bytes();
    let restored = BlockHeader::from_bytes(&bytes).unwrap();

    assert_eq!(restored.round, 42);
    assert_eq!(restored.prev_hash(), vec![0xAA; 32]);
    assert_eq!(restored.vdf_state(), vec![0xBB; 32]);
    assert_eq!(restored.vrf_proof(), vec![0xCC; 80]);
    assert_eq!(restored.proposer(), vec![0xDD; 32]);
    assert_eq!(restored.timestamp, 1234567890);
}

#[test]
fn test_block_header_hash_computed_on_deserialize() {
    let original =
        BlockHeader::new(1, vec![0u8; 32], vec![1u8; 32], vec![2u8; 80], vec![3u8; 32], 1000);
    let bytes = original.to_bytes();
    let restored = BlockHeader::from_bytes(&bytes).unwrap();

    assert_eq!(restored.hash().len(), 32);
}

// ─── Engine Creation Tests ───────────────────────────────────────

#[test]
fn test_engine_creation() {
    let validators = create_validators(4);
    let engine = create_engine(&validators, 0);

    assert_eq!(engine.round(), 0);
    assert_eq!(engine.chain_height(), 0);
}

#[test]
fn test_engine_threshold() {
    let validators = create_validators(7);
    let engine = create_engine(&validators, 0);

    assert_eq!(engine.threshold(), 5);
}

#[test]
fn test_engine_state_initial() {
    let validators = create_validators(4);
    let engine = create_engine(&validators, 0);

    let state = engine.state();
    assert_eq!(state.round, 0);
    assert_eq!(state.phase, ConsensusPhase::Propose);
    assert_eq!(state.prevote_count, 0);
    assert_eq!(state.precommit_count, 0);
    assert_eq!(state.chain_height, 0);
}

// ─── Leader Election Tests ───────────────────────────────────────

#[test]
fn test_leader_election_with_high_tau() {
    let validators = create_validators(4);
    let mut engine = create_engine(&validators, 0);
    engine.start_round(1);
    assert!(engine.is_leader());
}

#[test]
fn test_leader_election_with_low_tau() {
    let validators = create_validators(4);
    let pks = pubkeys(&validators);
    let low_tau = [0x00u8; 32];

    let mut engine = ConsensusEngine::new_native(
        &validators[0].secret_key(),
        &validators[0].public_key(),
        pks,
        &low_tau,
    )
    .unwrap();

    engine.start_round(1);
    assert!(!engine.is_leader());
}

#[test]
fn test_leader_election_vrf_proof_nonempty() {
    let validators = create_validators(4);
    let mut engine = create_engine(&validators, 0);
    let proof = engine.start_round(1);
    assert_eq!(proof.len(), 80);
}

// ─── Round Lifecycle Tests ───────────────────────────────────────

#[test]
fn test_start_round_resets_state() {
    let validators = create_validators(4);
    let mut engine = create_engine(&validators, 0);

    engine.start_round(1);
    assert_eq!(engine.round(), 1);
    assert_eq!(engine.prevote_count(), 0);
    assert_eq!(engine.precommit_count(), 0);

    engine.start_round(2);
    assert_eq!(engine.round(), 2);
    assert_eq!(engine.prevote_count(), 0);
    assert_eq!(engine.precommit_count(), 0);
}

#[test]
fn test_propose_creates_block() {
    let validators = create_validators(4);
    let mut engine = create_engine(&validators, 0);
    engine.start_round(1);

    let vdf_state = vec![0xAA; 32];
    let block_bytes = engine.propose_native(&vdf_state, 1000).unwrap();

    assert!(!block_bytes.is_empty());

    let block = BlockHeader::from_bytes(&block_bytes).unwrap();
    assert_eq!(block.round, 1);
    assert_eq!(block.vdf_state(), vdf_state);
    assert_eq!(block.timestamp, 1000);
}

#[test]
fn test_propose_fails_for_non_leader() {
    let validators = create_validators(4);
    let pks = pubkeys(&validators);
    let low_tau = [0x00u8; 32];

    let mut engine = ConsensusEngine::new_native(
        &validators[0].secret_key(),
        &validators[0].public_key(),
        pks,
        &low_tau,
    )
    .unwrap();

    engine.start_round(1);
    let result = engine.propose_native(&[0xAA; 32], 1000);
    assert!(result.is_err());
}

#[test]
fn test_propose_round_and_timestamp() {
    let validators = create_validators(4);
    let mut engine = create_engine(&validators, 0);
    engine.start_round(5);

    let block_bytes = engine.propose_native(&[0xBB; 32], 99999).unwrap();
    let block = BlockHeader::from_bytes(&block_bytes).unwrap();

    assert_eq!(block.round, 5);
    assert_eq!(block.timestamp, 99999);
}

// ─── Vote Tests ──────────────────────────────────────────────────

#[test]
fn test_prevote_accepts_valid_proposal() {
    let validators = create_validators(4);
    let mut engine = create_engine(&validators, 0);
    engine.start_round(1);

    engine.propose_native(&[0xAA; 32], 1000).unwrap();
    let vote_bytes = engine.prevote_native(true).unwrap();

    assert!(!vote_bytes.is_empty());
    let round = u64::from_be_bytes(vote_bytes[0..8].try_into().unwrap());
    assert_eq!(round, 1);
    assert_eq!(vote_bytes[8], VotePhase::Prevote as u8);
    assert_eq!(vote_bytes[9], 1); // has hash
}

#[test]
fn test_prevote_nil_vote() {
    let validators = create_validators(4);
    let mut engine = create_engine(&validators, 0);
    engine.start_round(1);

    engine.propose_native(&[0xAA; 32], 1000).unwrap();
    let vote_bytes = engine.prevote_native(false).unwrap();

    assert!(!vote_bytes.is_empty());
    assert_eq!(vote_bytes[9], 0); // nil vote
}

#[test]
fn test_receive_vote_rejects_wrong_round() {
    let validators = create_validators(4);
    let mut engine = create_engine(&validators, 0);
    engine.start_round(1);

    let vote_bytes =
        serialize_vote(999, VotePhase::Prevote, Some(&[0xAA; 32]), &validators[1].public_key());

    let status = engine.receive_vote(&vote_bytes);
    assert_eq!(status, -1);
}

#[test]
fn test_receive_vote_rejects_non_validator() {
    let validators = create_validators(4);
    let fake_keypair = vrf::generate_keypair();
    let mut engine = create_engine(&validators, 0);
    engine.start_round(1);

    let vote_bytes =
        serialize_vote(1, VotePhase::Prevote, Some(&[0xAA; 32]), &fake_keypair.public_key());

    let status = engine.receive_vote(&vote_bytes);
    assert_eq!(status, -1);
}

#[test]
fn test_receive_vote_rejects_duplicate() {
    let validators = create_validators(4);
    let mut engine = create_engine(&validators, 0);
    engine.start_round(1);

    let vote_bytes =
        serialize_vote(1, VotePhase::Prevote, Some(&[0xAA; 32]), &validators[1].public_key());

    assert_eq!(engine.receive_vote(&vote_bytes), 0);
    assert_eq!(engine.receive_vote(&vote_bytes), -1);
}

#[test]
fn test_prevote_quorum_detection() {
    let validators = create_validators(4);
    let mut engine = create_engine(&validators, 0);
    engine.start_round(1);

    // n=4, threshold=3
    let v1 = serialize_vote(1, VotePhase::Prevote, Some(&[0xAA; 32]), &validators[1].public_key());
    let v2 = serialize_vote(1, VotePhase::Prevote, Some(&[0xAA; 32]), &validators[2].public_key());
    let v3 = serialize_vote(1, VotePhase::Prevote, Some(&[0xAA; 32]), &validators[3].public_key());

    assert_eq!(engine.receive_vote(&v1), 0);
    assert_eq!(engine.receive_vote(&v2), 0);
    assert_eq!(engine.receive_vote(&v3), 1); // prevote quorum
}

#[test]
fn test_precommit_quorum_detection() {
    let validators = create_validators(4);
    let mut engine = create_engine(&validators, 0);
    engine.start_round(1);

    let block_hash = [0xBB; 32];
    let v1 =
        serialize_vote(1, VotePhase::Precommit, Some(&block_hash), &validators[1].public_key());
    let v2 =
        serialize_vote(1, VotePhase::Precommit, Some(&block_hash), &validators[2].public_key());
    let v3 =
        serialize_vote(1, VotePhase::Precommit, Some(&block_hash), &validators[3].public_key());

    assert_eq!(engine.receive_vote(&v1), 0);
    assert_eq!(engine.receive_vote(&v2), 0);
    assert_eq!(engine.receive_vote(&v3), 2); // precommit quorum
}

#[test]
fn test_vote_counts() {
    let validators = create_validators(4);
    let mut engine = create_engine(&validators, 0);
    engine.start_round(1);

    assert_eq!(engine.prevote_count(), 0);
    assert_eq!(engine.precommit_count(), 0);

    let v1 = serialize_vote(1, VotePhase::Prevote, Some(&[0xAA; 32]), &validators[1].public_key());
    engine.receive_vote(&v1);
    assert_eq!(engine.prevote_count(), 1);
    assert_eq!(engine.precommit_count(), 0);

    let v2 =
        serialize_vote(1, VotePhase::Precommit, Some(&[0xAA; 32]), &validators[2].public_key());
    engine.receive_vote(&v2);
    assert_eq!(engine.prevote_count(), 1);
    assert_eq!(engine.precommit_count(), 1);
}

// ─── Chain Tests ─────────────────────────────────────────────────

#[test]
fn test_chain_starts_empty() {
    let validators = create_validators(4);
    let engine = create_engine(&validators, 0);

    assert_eq!(engine.chain_height(), 0);
    assert_eq!(engine.latest_block_hash(), vec![0u8; 32]);
}

#[test]
fn test_verify_chain_empty() {
    let validators = create_validators(4);
    let engine = create_engine(&validators, 0);
    assert!(engine.verify_chain());
}

#[test]
fn test_finalization_increases_chain_height() {
    let validators = create_validators(4);
    let mut engine = create_engine(&validators, 0);
    engine.start_round(1);

    let block_bytes = engine.propose_native(&[0xAA; 32], 1000).unwrap();
    let block = BlockHeader::from_bytes(&block_bytes).unwrap();

    // Collect prevote quorum (need 3 out of 4)
    for i in 1..4 {
        let v =
            serialize_vote(1, VotePhase::Prevote, Some(&block.hash()), &validators[i].public_key());
        engine.receive_vote(&v);
    }

    // Collect precommit quorum (need 3 out of 4)
    for i in 1..4 {
        let v = serialize_vote(
            1,
            VotePhase::Precommit,
            Some(&block.hash()),
            &validators[i].public_key(),
        );
        engine.receive_vote(&v);
    }

    engine.finalize_native().unwrap();
    assert_eq!(engine.chain_height(), 1);
}

// ─── Leader Election Verification ────────────────────────────────

#[test]
fn test_verify_leader_election_function() {
    let validators = create_validators(4);
    let mut engine = create_engine(&validators, 0);
    engine.start_round(1);

    let block_bytes = engine.propose_native(&[0xAA; 32], 1000).unwrap();
    assert!(verify_leader_election(&block_bytes, &TAU));
}

#[test]
fn test_verify_leader_election_with_low_tau() {
    let validators = create_validators(4);
    let mut engine = create_engine(&validators, 0);
    engine.start_round(1);

    let block_bytes = engine.propose_native(&[0xAA; 32], 1000).unwrap();
    let low_tau = [0x00u8; 32];
    assert!(!verify_leader_election(&block_bytes, &low_tau));
}
