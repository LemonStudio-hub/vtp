/**
 * Consensus Protocol Type Definitions
 *
 * Shared types for the VRF-driven leader election + BFT validation
 * consensus protocol. Covers round lifecycle, voting, and block finalization.
 */

// ─── Consensus Phases ──────────────────────────────────────────────

/** Phases of a consensus round. */
export type ConsensusPhase = 'propose' | 'prevote' | 'precommit' | 'commit';

/** Vote phase in the two-phase BFT protocol. */
export type VotePhase = 'prevote' | 'precommit';

// ─── Block ─────────────────────────────────────────────────────────

/** Finalized block header. */
export interface BlockHeader {
  /** Consensus round number. */
  round: number;
  /** Hash of the previous block (32 bytes). */
  prevHash: Uint8Array;
  /** 32-byte VDF state at the checkpoint. */
  vdfState: Uint8Array;
  /** 80-byte VRF proof from leader election. */
  vrfProof: Uint8Array;
  /** 32-byte Ed25519 public key of the proposer. */
  proposer: Uint8Array;
  /** Unix timestamp in milliseconds. */
  timestamp: number;
  /** 32-byte SHA-256 hash of the block. */
  hash: Uint8Array;
}

// ─── Consensus State ───────────────────────────────────────────────

/** Snapshot of the consensus engine state (for UI display). */
export interface ConsensusState {
  /** Current round number. */
  round: number;
  /** Current phase. */
  phase: ConsensusPhase;
  /** Whether this node is the leader of the current round. */
  isLeader: boolean;
  /** Number of prevotes collected. */
  prevoteCount: number;
  /** Number of precommits collected. */
  precommitCount: number;
  /** Number of finalized blocks. */
  chainHeight: number;
}

// ─── Consensus Configuration ───────────────────────────────────────

/** Configuration for the consensus engine. */
export interface ConsensusConfig {
  /** Round timeout in ms — if no progress, advance to next round (default: 30000). */
  roundTimeoutMs: number;
  /** VRF output threshold for leader election (32 bytes). */
  tau: Uint8Array;
  /** Checkpoint interval in VDF steps. */
  checkpointInterval: number;
}

// ─── Consensus Events ──────────────────────────────────────────────

/** Events emitted by the ConsensusEngine. */
export interface ConsensusEvents {
  /** Phase transition occurred. */
  onPhaseChange: (round: number, phase: ConsensusPhase) => void;
  /** This node was elected as leader. */
  onLeaderElected: (round: number) => void;
  /** Received a valid proposal. */
  onProposal: (round: number, blockHash: Uint8Array) => void;
  /** A block was finalized. */
  onCommit: (round: number, block: BlockHeader) => void;
  /** Round timed out — no progress. */
  onRoundTimeout: (round: number) => void;
  /** Consensus state changed. */
  onStateChange: (state: ConsensusState) => void;
  /** Error occurred. */
  onError: (error: string) => void;
}

// ─── Network Messages ──────────────────────────────────────────────

/** Consensus proposal payload (for message codec). */
export interface ConsensusProposalPayload {
  type: 'consensus-proposal';
  round: number;
  vdfState: Uint8Array;
  vrfProof: Uint8Array;
  blockHash: Uint8Array;
  prevBlockHash: Uint8Array;
  timestamp: number;
}

/** Consensus vote payload (for message codec). */
export interface ConsensusVotePayload {
  type: 'consensus-vote';
  round: number;
  phase: VotePhase;
  blockHash: Uint8Array | null;
  voterPubkey: Uint8Array;
}

/** New round announcement payload (for message codec). */
export interface NewRoundPayload {
  type: 'new-round';
  round: number;
  seed: Uint8Array;
  validators: Uint8Array[];
}
