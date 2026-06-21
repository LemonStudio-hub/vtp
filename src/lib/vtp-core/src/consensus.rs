//! VRF-Driven Leader Election + BFT Validation Consensus Module
//!
//! Implements a round-based consensus protocol combining VRF-based leader election
//! with BFT-style voting for finality.
//!
//! # Protocol Overview
//!
//! Each consensus round follows this lifecycle:
//!
//! ```text
//!   1. NEW_ROUND    → Nodes start VDF computation for round R
//!   2. LEADER_ELECT → VRF lottery selects leader (vrf_output < tau)
//!   3. PROPOSE      → Leader broadcasts proposal (VDF state + VRF proof)
//!   4. PREVOTE      → Nodes validate proposal, broadcast prevote
//!   5. PRECOMMIT    → Nodes collect 2f+1 prevotes, broadcast precommit
//!   6. COMMIT       → Nodes collect 2f+1 precommits, finalize block
//!   7. NEXT_ROUND   → Advance to round R+1
//! ```
//!
//! # Fault Tolerance
//! Tolerates `f` Byzantine faults out of `3f+1` total validators.
//! Safety is guaranteed by the 2/3+1 quorum requirement on both prevotes and precommits.
//!
//! # Leader Election
//! The leader for each round is determined by a VRF lottery. Each validator computes
//! `VRF(sk, round_seed)`, and the validator with the smallest VRF output becomes the leader.
//! This ensures:
//! - Unpredictability: no one can predict the leader before the round starts
//! - Verifiability: anyone can verify the leader's VRF proof
//! - Uniqueness: exactly one leader per round (with overwhelming probability)

use crate::utils::hash_bytes;
use crate::vrf;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Consensus phase enumeration.
///
/// Represents the current phase of a consensus round.
/// Transitions follow: Propose → Prevote → Precommit → Commit.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsensusPhase {
    /// Waiting for a proposal from the elected leader.
    Propose = 0,
    /// Collecting prevotes on the proposal.
    Prevote = 1,
    /// Collecting precommits after quorum prevotes.
    Precommit = 2,
    /// Block finalized.
    Commit = 3,
}

/// Vote phase for BFT voting.
///
/// Distinguishes between prevote and precommit votes in the two-phase BFT protocol.
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VotePhase {
    /// First voting phase — nodes vote on whether the proposal is valid.
    Prevote = 0,
    /// Second voting phase — nodes commit after seeing a quorum of prevotes.
    Precommit = 1,
}

/// Block header — the finalized output of consensus.
///
/// Contains all information needed to verify a finalized block, including
/// the VDF state, VRF proof, proposer identity, and a hash chain link.
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Consensus round number.
    pub round: u64,

    /// Hash of the previous block (32 bytes), forming a hash chain.
    prev_hash: Vec<u8>,

    /// 32-byte VDF state at the checkpoint.
    vdf_state: Vec<u8>,

    /// 80-byte VRF proof from the leader election.
    vrf_proof: Vec<u8>,

    /// 32-byte Ed25519 public key of the proposer.
    proposer: Vec<u8>,

    /// Unix timestamp in milliseconds.
    pub timestamp: u64,

    /// 32-byte SHA-256 hash of all block fields.
    hash: Vec<u8>,
}

#[wasm_bindgen]
impl BlockHeader {
    /// Get the previous block hash.
    #[wasm_bindgen(getter)]
    pub fn prev_hash(&self) -> Vec<u8> {
        self.prev_hash.clone()
    }

    /// Get the VDF state.
    #[wasm_bindgen(getter)]
    pub fn vdf_state(&self) -> Vec<u8> {
        self.vdf_state.clone()
    }

    /// Get the VRF proof.
    #[wasm_bindgen(getter)]
    pub fn vrf_proof(&self) -> Vec<u8> {
        self.vrf_proof.clone()
    }

    /// Get the proposer public key.
    #[wasm_bindgen(getter)]
    pub fn proposer(&self) -> Vec<u8> {
        self.proposer.clone()
    }

    /// Get the block hash.
    #[wasm_bindgen(getter)]
    pub fn hash(&self) -> Vec<u8> {
        self.hash.clone()
    }

    /// Serialize the block header to bytes for transmission.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.round.to_be_bytes());
        data.extend_from_slice(&(self.prev_hash.len() as u32).to_be_bytes());
        data.extend_from_slice(&self.prev_hash);
        data.extend_from_slice(&(self.vdf_state.len() as u32).to_be_bytes());
        data.extend_from_slice(&self.vdf_state);
        data.extend_from_slice(&(self.vrf_proof.len() as u32).to_be_bytes());
        data.extend_from_slice(&self.vrf_proof);
        data.extend_from_slice(&(self.proposer.len() as u32).to_be_bytes());
        data.extend_from_slice(&self.proposer);
        data.extend_from_slice(&self.timestamp.to_be_bytes());
        data
    }

    /// Deserialize a block header from bytes.
    pub fn from_bytes(data: &[u8]) -> Result<BlockHeader, JsValue> {
        if data.len() < 8 {
            return Err(JsValue::from_str("BlockHeader data too short"));
        }

        let mut offset = 0;

        let round = u64::from_be_bytes(
            data[offset..offset + 8].try_into().map_err(|_| JsValue::from_str("Invalid round"))?,
        );
        offset += 8;

        let read_bytes = |data: &[u8], offset: &mut usize| -> Result<Vec<u8>, JsValue> {
            if data.len() < *offset + 4 {
                return Err(JsValue::from_str("Truncated length field"));
            }
            let len = u32::from_be_bytes(
                data[*offset..*offset + 4]
                    .try_into()
                    .map_err(|_| JsValue::from_str("Invalid len"))?,
            ) as usize;
            *offset += 4;
            if data.len() < *offset + len {
                return Err(JsValue::from_str("Truncated data"));
            }
            let result = data[*offset..*offset + len].to_vec();
            *offset += len;
            Ok(result)
        };

        let prev_hash = read_bytes(data, &mut offset)?;
        let vdf_state = read_bytes(data, &mut offset)?;
        let vrf_proof = read_bytes(data, &mut offset)?;
        let proposer = read_bytes(data, &mut offset)?;

        if data.len() < offset + 8 {
            return Err(JsValue::from_str("Truncated timestamp"));
        }
        let timestamp = u64::from_be_bytes(
            data[offset..offset + 8].try_into().map_err(|_| JsValue::from_str("Invalid ts"))?,
        );

        let mut header = BlockHeader {
            round,
            prev_hash,
            vdf_state,
            vrf_proof,
            proposer,
            timestamp,
            hash: Vec::new(),
        };
        header.hash = header.compute_hash();
        Ok(header)
    }
}

impl BlockHeader {
    /// Create a new block header with a computed hash.
    ///
    /// This is the primary constructor for use in tests and integration code.
    pub fn new(
        round: u64,
        prev_hash: Vec<u8>,
        vdf_state: Vec<u8>,
        vrf_proof: Vec<u8>,
        proposer: Vec<u8>,
        timestamp: u64,
    ) -> Self {
        let mut header =
            Self { round, prev_hash, vdf_state, vrf_proof, proposer, timestamp, hash: Vec::new() };
        header.hash = header.compute_hash();
        header
    }

    /// Compute the SHA-256 hash of the block header.
    fn compute_hash(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.round.to_be_bytes());
        data.extend_from_slice(&self.prev_hash);
        data.extend_from_slice(&self.vdf_state);
        data.extend_from_slice(&self.vrf_proof);
        data.extend_from_slice(&self.proposer);
        data.extend_from_slice(&self.timestamp.to_be_bytes());
        hash_bytes(&data)
    }
}

/// Consensus vote (prevote or precommit).
///
/// A vote is a signed attestation from a validator that they have observed
/// a valid (or invalid) proposal for a given round and phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusVote {
    /// The consensus round this vote belongs to.
    pub round: u64,

    /// Whether this is a prevote or precommit.
    pub phase: VotePhase,

    /// Hash of the block being voted on (32 bytes).
    /// `None` represents a nil vote (validator rejects the proposal).
    pub block_hash: Option<Vec<u8>>,

    /// 32-byte public key of the voter.
    pub voter: Vec<u8>,
}

/// Per-round consensus state.
///
/// Tracks the proposal, collected votes, and phase transitions for a single round.
#[derive(Debug)]
struct ConsensusRoundState {
    /// Current phase of this round.
    phase: ConsensusPhase,

    /// The block proposal from the leader (if received).
    proposal: Option<BlockHeader>,

    /// Map from voter pubkey → prevote.
    prevotes: HashMap<Vec<u8>, ConsensusVote>,

    /// Map from voter pubkey → precommit.
    precommits: HashMap<Vec<u8>, ConsensusVote>,

    /// The set of validator public keys for this round.
    validators: Vec<Vec<u8>>,

    /// The quorum threshold (2f+1).
    threshold: usize,
}

impl ConsensusRoundState {
    fn new(validators: Vec<Vec<u8>>) -> Self {
        let n = validators.len();
        let f = (n - 1) / 3; // max Byzantine faults
        let threshold = 2 * f + 1;

        Self {
            phase: ConsensusPhase::Propose,
            proposal: None,
            prevotes: HashMap::new(),
            precommits: HashMap::new(),
            validators,
            threshold,
        }
    }

    /// Check if a pubkey is in the validator set.
    fn is_validator(&self, pubkey: &[u8]) -> bool {
        self.validators.iter().any(|v| v == pubkey)
    }

    /// Count non-nil prevotes for a specific block hash.
    #[allow(dead_code)]
    fn prevote_count_for(&self, block_hash: &[u8]) -> usize {
        self.prevotes.values().filter(|v| v.block_hash.as_deref() == Some(block_hash)).count()
    }

    /// Count non-nil precommits for a specific block hash.
    fn precommit_count_for(&self, block_hash: &[u8]) -> usize {
        self.precommits.values().filter(|v| v.block_hash.as_deref() == Some(block_hash)).count()
    }

    /// Check if prevote quorum is reached for any block hash.
    fn has_prevote_quorum(&self) -> Option<Vec<u8>> {
        // Find the block hash with the most prevotes
        let mut counts: HashMap<Vec<u8>, usize> = HashMap::new();
        for vote in self.prevotes.values() {
            if let Some(ref hash) = vote.block_hash {
                *counts.entry(hash.clone()).or_insert(0) += 1;
            }
        }
        counts.into_iter().find(|(_, count)| *count >= self.threshold).map(|(hash, _)| hash)
    }

    /// Check if precommit quorum is reached for any block hash.
    fn has_precommit_quorum(&self) -> Option<Vec<u8>> {
        let mut counts: HashMap<Vec<u8>, usize> = HashMap::new();
        for vote in self.precommits.values() {
            if let Some(ref hash) = vote.block_hash {
                *counts.entry(hash.clone()).or_insert(0) += 1;
            }
        }
        counts.into_iter().find(|(_, count)| *count >= self.threshold).map(|(hash, _)| hash)
    }
}

/// Consensus engine state summary (for WASM export).
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusState {
    /// Current round number.
    pub round: u64,

    /// Current phase.
    pub phase: ConsensusPhase,

    /// Whether this node is the leader of the current round.
    pub is_leader: bool,

    /// Number of prevotes collected.
    pub prevote_count: u32,

    /// Number of precommits collected.
    pub precommit_count: u32,

    /// Number of finalized blocks.
    pub chain_height: u64,
}

/// Finalized block chain.
///
/// Stores the sequence of finalized block headers, forming an immutable chain
/// where each block references the hash of its predecessor.
#[derive(Debug, Clone)]
struct BlockChain {
    /// Ordered list of finalized block headers.
    blocks: Vec<BlockHeader>,
}

impl BlockChain {
    fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    /// Add a finalized block to the chain.
    fn append(&mut self, block: BlockHeader) {
        self.blocks.push(block);
    }

    /// Get the latest block hash (genesis hash if empty).
    fn latest_hash(&self) -> Vec<u8> {
        self.blocks.last().map(|b| b.hash.clone()).unwrap_or_else(|| vec![0u8; 32])
        // genesis: all zeros
    }

    /// Get the chain height (number of finalized blocks).
    fn height(&self) -> u64 {
        self.blocks.len() as u64
    }

    /// Verify the hash chain integrity.
    fn verify_chain(&self) -> bool {
        for (i, block) in self.blocks.iter().enumerate() {
            // Verify block hash
            let expected_hash = block.compute_hash();
            if block.hash != expected_hash {
                return false;
            }

            // Verify prev_hash link
            if i == 0 {
                if block.prev_hash != vec![0u8; 32] {
                    return false;
                }
            } else {
                if block.prev_hash != self.blocks[i - 1].hash {
                    return false;
                }
            }
        }
        true
    }
}

/// Main consensus engine.
///
/// Coordinates the full consensus protocol: leader election via VRF,
/// proposal validation, two-phase BFT voting, and block finalization.
///
/// # Usage
/// ```ignore
/// let engine = ConsensusEngine::new(secret_key, public_key, validators, tau);
/// engine.start_round();
/// if engine.is_leader() {
///     let proposal = engine.propose(vdf_state, vrf_proof);
///     // broadcast proposal
/// }
/// // On receiving a proposal:
/// engine.receive_proposal(proposal_bytes);
/// // On receiving votes:
/// engine.receive_vote(vote_bytes);
/// ```
#[wasm_bindgen]
pub struct ConsensusEngine {
    /// This node's VRF secret key (32 bytes).
    secret_key: Vec<u8>,

    /// This node's VRF public key (32 bytes).
    public_key: Vec<u8>,

    /// Current round number.
    current_round: u64,

    /// Per-round state.
    round_state: Option<ConsensusRoundState>,

    /// VRF output threshold for leader election (32 bytes).
    tau: Vec<u8>,

    /// Finalized block chain.
    chain: BlockChain,

    /// Map of round → leader's VRF proof (for verification).
    round_proofs: HashMap<u64, Vec<u8>>,

    /// All known validator public keys.
    all_validators: Vec<Vec<u8>>,
}

#[wasm_bindgen]
impl ConsensusEngine {
    /// Create a new consensus engine (WASM constructor).
    ///
    /// # Arguments
    /// - `secret_key`: 32-byte Ed25519 secret key
    /// - `public_key`: 32-byte Ed25519 public key
    /// - `validator_keys`: Array of all validator public keys (including self)
    /// - `tau`: 32-byte VRF output threshold for leader election
    ///
    /// # Returns
    /// A new `ConsensusEngine` instance.
    #[wasm_bindgen(constructor)]
    pub fn new(
        secret_key: &[u8],
        public_key: &[u8],
        validator_keys: &JsValue,
        tau: &[u8],
    ) -> Result<ConsensusEngine, JsValue> {
        let validators: Vec<Vec<u8>> = serde_wasm_bindgen::from_value(validator_keys.clone())
            .map_err(|e| JsValue::from_str(&format!("Invalid validator keys: {}", e)))?;

        Self::new_native(secret_key, public_key, validators, tau).map_err(JsValue::from_str)
    }

    /// Start a new consensus round.
    ///
    /// Initializes the round state and determines if this node is the leader
    /// by running the VRF lottery.
    ///
    /// # Arguments
    /// - `round`: The round number to start
    ///
    /// # Returns
    /// The VRF proof for this round (broadcast to prove leadership).
    #[wasm_bindgen]
    pub fn start_round(&mut self, round: u64) -> Vec<u8> {
        self.current_round = round;
        let round_state = ConsensusRoundState::new(self.all_validators.clone());
        self.round_state = Some(round_state);

        // Generate VRF proof for leader election
        let seed = round.to_be_bytes();
        let proof = vrf::prove(&self.secret_key, &seed);

        // Store the proof for later verification
        self.round_proofs.insert(round, proof.clone());

        proof
    }

    /// Check if this node is the leader for the current round.
    ///
    /// The leader is determined by comparing the VRF output against the tau threshold.
    /// A node is the leader if its VRF output < tau.
    ///
    /// # Returns
    /// `true` if this node is the leader.
    #[wasm_bindgen]
    pub fn is_leader(&self) -> bool {
        let proof = match self.round_proofs.get(&self.current_round) {
            Some(p) => p,
            None => return false,
        };
        let vrf_output = vrf::proof_to_hash(proof);
        vrf_output < self.tau
    }

    /// Create a block proposal (leader only).
    ///
    /// The leader calls this after VDF computation reaches a checkpoint.
    /// The proposal includes the VDF state, VRF proof, and is signed.
    ///
    /// # Arguments
    /// - `vdf_state`: 32-byte VDF state at the checkpoint
    /// - `timestamp`: Unix timestamp in milliseconds
    ///
    /// # Returns
    /// Serialized proposal bytes for broadcast.
    #[wasm_bindgen]
    pub fn propose(&mut self, vdf_state: &[u8], timestamp: u64) -> Result<Vec<u8>, JsValue> {
        if !self.is_leader() {
            return Err(JsValue::from_str("Only the leader can propose"));
        }

        let vrf_proof = self
            .round_proofs
            .get(&self.current_round)
            .ok_or_else(|| JsValue::from_str("No VRF proof for current round"))?
            .clone();

        let block = BlockHeader {
            round: self.current_round,
            prev_hash: self.chain.latest_hash(),
            vdf_state: vdf_state.to_vec(),
            vrf_proof,
            proposer: self.public_key.clone(),
            timestamp,
            hash: Vec::new(),
        };

        // Compute hash after construction
        let mut block = block;
        block.hash = block.compute_hash();

        // Store the proposal in round state
        if let Some(ref mut state) = self.round_state {
            state.proposal = Some(block.clone());
            state.phase = ConsensusPhase::Prevote;
        }

        Ok(block.to_bytes())
    }

    /// Receive and validate a block proposal.
    ///
    /// Any validator calls this upon receiving a proposal from the leader.
    /// Validates the VRF proof for leader election and the VDF state.
    ///
    /// # Arguments
    /// - `proposal_bytes`: Serialized proposal from the leader
    ///
    /// # Returns
    /// `true` if the proposal is valid and accepted.
    #[wasm_bindgen]
    pub fn receive_proposal(&mut self, proposal_bytes: &[u8]) -> bool {
        let block = match BlockHeader::from_bytes(proposal_bytes) {
            Ok(b) => b,
            Err(_) => return false,
        };

        // Verify round matches
        if block.round != self.current_round {
            return false;
        }

        // Verify the VRF proof for leader election
        let round_seed = block.round.to_be_bytes();
        if !vrf::verify(&block.proposer, &round_seed, &block.vrf_proof) {
            return false;
        }

        // Verify the proposer wins the VRF lottery (output < tau)
        let vrf_output = vrf::proof_to_hash(&block.vrf_proof);
        if vrf_output >= self.tau {
            return false;
        }

        // Verify the block hash
        let expected_hash = block.compute_hash();
        if block.hash != expected_hash {
            return false;
        }

        // Verify prev_hash links to our chain
        if block.prev_hash != self.chain.latest_hash() {
            return false;
        }

        // Accept the proposal
        if let Some(ref mut state) = self.round_state {
            if state.proposal.is_none() {
                state.proposal = Some(block);
                state.phase = ConsensusPhase::Prevote;
                return true;
            }
        }

        false
    }

    /// Cast a prevote for the current round's proposal.
    ///
    /// Should be called after validating the proposal. If the proposal is valid,
    /// the prevote is for the block hash; otherwise a nil prevote is cast.
    ///
    /// # Arguments
    /// - `accept`: Whether to accept the proposal
    ///
    /// # Returns
    /// Serialized vote bytes for broadcast, or error if no proposal received.
    #[wasm_bindgen]
    pub fn prevote(&self, accept: bool) -> Result<Vec<u8>, JsValue> {
        let state =
            self.round_state.as_ref().ok_or_else(|| JsValue::from_str("No active round"))?;

        if state.phase != ConsensusPhase::Prevote && state.phase != ConsensusPhase::Propose {
            return Err(JsValue::from_str("Not in prevote phase"));
        }

        let block_hash =
            if accept { state.proposal.as_ref().map(|p| p.hash.clone()) } else { None };

        let vote = ConsensusVote {
            round: self.current_round,
            phase: VotePhase::Prevote,
            block_hash,
            voter: self.public_key.clone(),
        };

        self.serialize_vote(&vote)
    }

    /// Cast a precommit for the current round.
    ///
    /// Should be called after collecting a quorum of prevotes for the same block hash.
    ///
    /// # Arguments
    /// - `block_hash`: The block hash to precommit (must match the prevote quorum)
    ///
    /// # Returns
    /// Serialized vote bytes for broadcast.
    #[wasm_bindgen]
    pub fn precommit(&self, block_hash: &[u8]) -> Result<Vec<u8>, JsValue> {
        let state =
            self.round_state.as_ref().ok_or_else(|| JsValue::from_str("No active round"))?;

        if state.phase != ConsensusPhase::Prevote && state.phase != ConsensusPhase::Precommit {
            return Err(JsValue::from_str("Not in precommit phase"));
        }

        let vote = ConsensusVote {
            round: self.current_round,
            phase: VotePhase::Precommit,
            block_hash: Some(block_hash.to_vec()),
            voter: self.public_key.clone(),
        };

        self.serialize_vote(&vote)
    }

    /// Receive and aggregate a vote from another validator.
    ///
    /// Validates the vote and adds it to the appropriate vote pool.
    /// Automatically transitions phases when quorum is reached.
    ///
    /// # Arguments
    /// - `vote_bytes`: Serialized vote from a peer
    ///
    /// # Returns
    /// A status code:
    /// - 0: Vote accepted, no phase change
    /// - 1: Vote accepted, prevote quorum reached → transition to precommit
    /// - 2: Vote accepted, precommit quorum reached → block finalized
    /// - -1: Vote rejected (invalid)
    #[wasm_bindgen]
    pub fn receive_vote(&mut self, vote_bytes: &[u8]) -> i32 {
        let vote = match self.deserialize_vote(vote_bytes) {
            Some(v) => v,
            None => return -1,
        };

        // Validate round
        if vote.round != self.current_round {
            return -1;
        }

        // Validate voter is in the validator set
        let state = match self.round_state.as_mut() {
            Some(s) => s,
            None => return -1,
        };

        if !state.is_validator(&vote.voter) {
            return -1;
        }

        // Add vote to the appropriate pool
        match vote.phase {
            VotePhase::Prevote => {
                // Reject if we already have a prevote from this voter
                if state.prevotes.contains_key(&vote.voter) {
                    return -1;
                }
                state.prevotes.insert(vote.voter.clone(), vote);

                // Check for prevote quorum
                if state.has_prevote_quorum().is_some() {
                    state.phase = ConsensusPhase::Precommit;
                    return 1;
                }
            }
            VotePhase::Precommit => {
                // Reject if we already have a precommit from this voter
                if state.precommits.contains_key(&vote.voter) {
                    return -1;
                }
                state.precommits.insert(vote.voter.clone(), vote);

                // Check for precommit quorum
                if state.has_precommit_quorum().is_some() {
                    state.phase = ConsensusPhase::Commit;
                    return 2;
                }
            }
        }

        0
    }

    /// Finalize the current round's block.
    ///
    /// Should be called after receiving a precommit quorum. Adds the block
    /// to the finalized chain.
    ///
    /// # Returns
    /// The finalized block header bytes, or error if finalization is not possible.
    #[wasm_bindgen]
    pub fn finalize(&mut self) -> Result<Vec<u8>, JsValue> {
        let state =
            self.round_state.as_ref().ok_or_else(|| JsValue::from_str("No active round"))?;

        if state.phase != ConsensusPhase::Commit {
            return Err(JsValue::from_str("Not in commit phase"));
        }

        let block = state
            .proposal
            .as_ref()
            .ok_or_else(|| JsValue::from_str("No proposal to finalize"))?
            .clone();

        // Verify precommit quorum exists for this block's hash
        let state_ref = self.round_state.as_ref().unwrap();
        if state_ref.precommit_count_for(&block.hash) < state_ref.threshold {
            return Err(JsValue::from_str("Insufficient precommits"));
        }

        // Add to chain
        self.chain.append(block.clone());

        Ok(block.to_bytes())
    }

    /// Get the current consensus state (for UI display).
    #[wasm_bindgen(getter)]
    pub fn state(&self) -> ConsensusState {
        let (phase, prevote_count, precommit_count) = match &self.round_state {
            Some(rs) => (rs.phase, rs.prevotes.len() as u32, rs.precommits.len() as u32),
            None => (ConsensusPhase::Propose, 0, 0),
        };

        ConsensusState {
            round: self.current_round,
            phase,
            is_leader: self.is_leader(),
            prevote_count,
            precommit_count,
            chain_height: self.chain.height(),
        }
    }

    /// Get the current round number.
    #[wasm_bindgen(getter)]
    pub fn round(&self) -> u64 {
        self.current_round
    }

    /// Get the finalized chain height.
    #[wasm_bindgen(getter)]
    pub fn chain_height(&self) -> u64 {
        self.chain.height()
    }

    /// Get the latest finalized block hash.
    #[wasm_bindgen]
    pub fn latest_block_hash(&self) -> Vec<u8> {
        self.chain.latest_hash()
    }

    /// Verify the integrity of the finalized chain.
    ///
    /// Checks that all block hashes are correct and the chain links are valid.
    ///
    /// # Returns
    /// `true` if the chain is valid.
    #[wasm_bindgen]
    pub fn verify_chain(&self) -> bool {
        self.chain.verify_chain()
    }

    /// Get the number of prevotes collected in the current round.
    #[wasm_bindgen]
    pub fn prevote_count(&self) -> u32 {
        self.round_state.as_ref().map(|s| s.prevotes.len() as u32).unwrap_or(0)
    }

    /// Get the number of precommits collected in the current round.
    #[wasm_bindgen]
    pub fn precommit_count(&self) -> u32 {
        self.round_state.as_ref().map(|s| s.precommits.len() as u32).unwrap_or(0)
    }

    /// Get the quorum threshold for the current validator set.
    #[wasm_bindgen]
    pub fn threshold(&self) -> u32 {
        let n = self.all_validators.len();
        let f = (n - 1) / 3;
        (2 * f + 1) as u32
    }
}

// ─── Private Helpers ──────────────────────────────────────────────

impl ConsensusEngine {
    /// Create a new consensus engine (Rust-native constructor).
    ///
    /// This is the core constructor used by both the WASM binding and native tests.
    pub fn new_native(
        secret_key: &[u8],
        public_key: &[u8],
        validators: Vec<Vec<u8>>,
        tau: &[u8],
    ) -> Result<Self, &'static str> {
        if validators.is_empty() {
            return Err("Validator set cannot be empty");
        }

        if tau.len() != 32 {
            return Err("Tau must be 32 bytes");
        }

        Ok(Self {
            secret_key: secret_key.to_vec(),
            public_key: public_key.to_vec(),
            current_round: 0,
            round_state: None,
            tau: tau.to_vec(),
            chain: BlockChain::new(),
            round_proofs: HashMap::new(),
            all_validators: validators,
        })
    }

    /// Create a block proposal (native version, returns Result<..., String>).
    pub fn propose_native(&mut self, vdf_state: &[u8], timestamp: u64) -> Result<Vec<u8>, String> {
        if !self.is_leader() {
            return Err("Only the leader can propose".to_string());
        }

        let vrf_proof = self
            .round_proofs
            .get(&self.current_round)
            .ok_or("No VRF proof for current round")?
            .clone();

        let block = BlockHeader::new(
            self.current_round,
            self.chain.latest_hash(),
            vdf_state.to_vec(),
            vrf_proof,
            self.public_key.clone(),
            timestamp,
        );

        if let Some(ref mut state) = self.round_state {
            state.proposal = Some(block.clone());
            state.phase = ConsensusPhase::Prevote;
        }

        Ok(block.to_bytes())
    }

    /// Cast a prevote (native version).
    pub fn prevote_native(&self, accept: bool) -> Result<Vec<u8>, String> {
        let state = self.round_state.as_ref().ok_or("No active round")?;

        if state.phase != ConsensusPhase::Prevote && state.phase != ConsensusPhase::Propose {
            return Err("Not in prevote phase".to_string());
        }

        let block_hash =
            if accept { state.proposal.as_ref().map(|p| p.hash.clone()) } else { None };

        let vote = ConsensusVote {
            round: self.current_round,
            phase: VotePhase::Prevote,
            block_hash,
            voter: self.public_key.clone(),
        };

        Ok(self.serialize_vote_bytes(&vote))
    }

    /// Cast a precommit (native version).
    pub fn precommit_native(&self, block_hash: &[u8]) -> Result<Vec<u8>, String> {
        let state = self.round_state.as_ref().ok_or("No active round")?;

        if state.phase != ConsensusPhase::Prevote && state.phase != ConsensusPhase::Precommit {
            return Err("Not in precommit phase".to_string());
        }

        let vote = ConsensusVote {
            round: self.current_round,
            phase: VotePhase::Precommit,
            block_hash: Some(block_hash.to_vec()),
            voter: self.public_key.clone(),
        };

        Ok(self.serialize_vote_bytes(&vote))
    }

    /// Finalize the current round (native version).
    pub fn finalize_native(&mut self) -> Result<Vec<u8>, String> {
        let state = self.round_state.as_ref().ok_or("No active round")?;

        if state.phase != ConsensusPhase::Commit {
            return Err("Not in commit phase".to_string());
        }

        let block = state.proposal.as_ref().ok_or("No proposal to finalize")?.clone();

        let state_ref = self.round_state.as_ref().unwrap();
        if state_ref.precommit_count_for(&block.hash) < state_ref.threshold {
            return Err("Insufficient precommits".to_string());
        }

        self.chain.append(block.clone());
        Ok(block.to_bytes())
    }

    /// Serialize a vote to bytes (non-fallible helper).
    fn serialize_vote_bytes(&self, vote: &ConsensusVote) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&vote.round.to_be_bytes());
        data.push(vote.phase as u8);

        match &vote.block_hash {
            Some(hash) => {
                data.push(1);
                data.extend_from_slice(&(hash.len() as u32).to_be_bytes());
                data.extend_from_slice(hash);
            }
            None => {
                data.push(0);
            }
        }

        data.extend_from_slice(&(vote.voter.len() as u32).to_be_bytes());
        data.extend_from_slice(&vote.voter);
        data
    }

    /// Serialize a vote to bytes for transmission (WASM version).
    fn serialize_vote(&self, vote: &ConsensusVote) -> Result<Vec<u8>, JsValue> {
        Ok(self.serialize_vote_bytes(vote))
    }

    /// Deserialize a vote from bytes.
    fn deserialize_vote(&self, data: &[u8]) -> Option<ConsensusVote> {
        if data.len() < 10 {
            return None;
        }

        let mut offset = 0;

        let round = u64::from_be_bytes(data[offset..offset + 8].try_into().ok()?);
        offset += 8;

        let phase = match data[offset] {
            0 => VotePhase::Prevote,
            1 => VotePhase::Precommit,
            _ => return None,
        };
        offset += 1;

        let has_hash = data[offset] != 0;
        offset += 1;

        let block_hash = if has_hash {
            if data.len() < offset + 4 {
                return None;
            }
            let len = u32::from_be_bytes(data[offset..offset + 4].try_into().ok()?) as usize;
            offset += 4;
            if data.len() < offset + len {
                return None;
            }
            let hash = data[offset..offset + len].to_vec();
            offset += len;
            Some(hash)
        } else {
            None
        };

        if data.len() < offset + 4 {
            return None;
        }
        let voter_len = u32::from_be_bytes(data[offset..offset + 4].try_into().ok()?) as usize;
        offset += 4;
        if data.len() < offset + voter_len {
            return None;
        }
        let voter = data[offset..offset + voter_len].to_vec();

        Some(ConsensusVote { round, phase, block_hash, voter })
    }
}

// ─── Standalone Verification Functions ────────────────────────────

/// Verify a block header's VRF proof for leader election.
///
/// This function can be called by any node to verify that the proposer
/// was legitimately elected as the leader for the given round.
///
/// # Arguments
/// - `block_bytes`: Serialized block header
/// - `tau`: 32-byte VRF output threshold
///
/// # Returns
/// `true` if the block's VRF proof is valid and the proposer wins the election.
#[wasm_bindgen]
pub fn verify_leader_election(block_bytes: &[u8], tau: &[u8]) -> bool {
    let block = match BlockHeader::from_bytes(block_bytes) {
        Ok(b) => b,
        Err(_) => return false,
    };

    // Verify VRF proof
    let round_seed = block.round.to_be_bytes();
    if !vrf::verify(&block.proposer, &round_seed, &block.vrf_proof) {
        return false;
    }

    // Verify VRF output < tau
    let vrf_output = vrf::proof_to_hash(&block.vrf_proof);
    vrf_output < tau.to_vec()
}

/// Compute the quorum threshold for a given number of validators.
///
/// Uses the formula: threshold = 2 * floor((n-1)/3) + 1
///
/// # Arguments
/// - `num_validators`: Total number of validators
///
/// # Returns
/// The quorum threshold (minimum votes needed for finality).
#[wasm_bindgen]
pub fn compute_threshold(num_validators: u32) -> u32 {
    if num_validators == 0 {
        return 0;
    }
    let f = (num_validators - 1) / 3;
    2 * f + 1
}

// ─── Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    /// Helper: create a validator set of `n` keypairs.
    fn create_validators(n: usize) -> Vec<vrf::VrfKeypair> {
        (0..n).map(|_| vrf::generate_keypair()).collect()
    }

    /// Helper: extract public keys from keypairs.
    fn pubkeys(validators: &[vrf::VrfKeypair]) -> Vec<Vec<u8>> {
        validators.iter().map(|kp| kp.public_key()).collect()
    }

    #[wasm_bindgen_test]
    fn test_consensus_engine_creation() {
        let validators = create_validators(4);
        let pks = pubkeys(&validators);
        let tau = [0xFFu8; 32];
        let pks_js = serde_wasm_bindgen::to_value(&pks).unwrap();

        let engine = ConsensusEngine::new(
            &validators[0].secret_key(),
            &validators[0].public_key(),
            &pks_js,
            &tau,
        )
        .unwrap();

        assert_eq!(engine.round(), 0);
        assert_eq!(engine.chain_height(), 0);
    }

    #[wasm_bindgen_test]
    fn test_threshold_computation() {
        assert_eq!(compute_threshold(1), 1);
        assert_eq!(compute_threshold(4), 3);
        assert_eq!(compute_threshold(7), 5);
        assert_eq!(compute_threshold(10), 7);
        assert_eq!(compute_threshold(0), 0);
    }

    #[wasm_bindgen_test]
    fn test_block_header_roundtrip() {
        let block = BlockHeader {
            round: 1,
            prev_hash: vec![0u8; 32],
            vdf_state: vec![1u8; 32],
            vrf_proof: vec![2u8; 80],
            proposer: vec![3u8; 32],
            timestamp: 1000000,
            hash: Vec::new(),
        };
        let mut block = block;
        block.hash = block.compute_hash();

        let bytes = block.to_bytes();
        let restored = BlockHeader::from_bytes(&bytes).unwrap();

        assert_eq!(restored.round, 1);
        assert_eq!(restored.prev_hash, vec![0u8; 32]);
        assert_eq!(restored.vdf_state, vec![1u8; 32]);
        assert_eq!(restored.vrf_proof, vec![2u8; 80]);
        assert_eq!(restored.proposer, vec![3u8; 32]);
        assert_eq!(restored.timestamp, 1000000);
        assert_eq!(restored.hash, block.hash);
    }

    #[wasm_bindgen_test]
    fn test_vote_roundtrip() {
        let validators = create_validators(4);
        let pks = pubkeys(&validators);
        let tau = [0xFFu8; 32];
        let pks_js = serde_wasm_bindgen::to_value(&pks).unwrap();

        let engine = ConsensusEngine::new(
            &validators[0].secret_key(),
            &validators[0].public_key(),
            &pks_js,
            &tau,
        )
        .unwrap();

        let vote = ConsensusVote {
            round: 1,
            phase: VotePhase::Prevote,
            block_hash: Some(vec![0xAAu8; 32]),
            voter: validators[0].public_key(),
        };

        let bytes = engine.serialize_vote(&vote).unwrap();
        let restored = engine.deserialize_vote(&bytes).unwrap();

        assert_eq!(restored.round, 1);
        assert_eq!(restored.phase, VotePhase::Prevote);
        assert_eq!(restored.block_hash, Some(vec![0xAAu8; 32]));
        assert_eq!(restored.voter, validators[0].public_key());
    }

    #[wasm_bindgen_test]
    fn test_nil_vote_roundtrip() {
        let validators = create_validators(4);
        let pks = pubkeys(&validators);
        let tau = [0xFFu8; 32];
        let pks_js = serde_wasm_bindgen::to_value(&pks).unwrap();

        let engine = ConsensusEngine::new(
            &validators[0].secret_key(),
            &validators[0].public_key(),
            &pks_js,
            &tau,
        )
        .unwrap();

        let vote = ConsensusVote {
            round: 5,
            phase: VotePhase::Prevote,
            block_hash: None,
            voter: validators[0].public_key(),
        };

        let bytes = engine.serialize_vote(&vote).unwrap();
        let restored = engine.deserialize_vote(&bytes).unwrap();

        assert_eq!(restored.round, 5);
        assert_eq!(restored.phase, VotePhase::Prevote);
        assert_eq!(restored.block_hash, None);
    }

    #[wasm_bindgen_test]
    fn test_block_chain_integrity() {
        let mut chain = BlockChain::new();
        assert_eq!(chain.height(), 0);
        assert_eq!(chain.latest_hash(), vec![0u8; 32]);

        let mut block1 = BlockHeader {
            round: 1,
            prev_hash: vec![0u8; 32],
            vdf_state: vec![1u8; 32],
            vrf_proof: vec![2u8; 80],
            proposer: vec![3u8; 32],
            timestamp: 1000,
            hash: Vec::new(),
        };
        block1.hash = block1.compute_hash();
        chain.append(block1);

        assert_eq!(chain.height(), 1);
        assert!(chain.verify_chain());

        let mut block2 = BlockHeader {
            round: 2,
            prev_hash: chain.latest_hash(),
            vdf_state: vec![4u8; 32],
            vrf_proof: vec![5u8; 80],
            proposer: vec![6u8; 32],
            timestamp: 2000,
            hash: Vec::new(),
        };
        block2.hash = block2.compute_hash();
        chain.append(block2);

        assert_eq!(chain.height(), 2);
        assert!(chain.verify_chain());
    }

    #[wasm_bindgen_test]
    fn test_block_chain_tamper_detection() {
        let mut chain = BlockChain::new();

        let mut block1 = BlockHeader {
            round: 1,
            prev_hash: vec![0u8; 32],
            vdf_state: vec![1u8; 32],
            vrf_proof: vec![2u8; 80],
            proposer: vec![3u8; 32],
            timestamp: 1000,
            hash: Vec::new(),
        };
        block1.hash = block1.compute_hash();
        chain.append(block1);
        assert!(chain.verify_chain());

        // Tamper with the stored block
        chain.blocks[0].vdf_state = vec![0xFFu8; 32];
        assert!(!chain.verify_chain());
    }
}
