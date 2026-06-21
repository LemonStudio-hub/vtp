/**
 * Consensus Engine
 *
 * Main orchestrator for the VRF-driven leader election + BFT validation
 * consensus protocol. Manages the round lifecycle, drives phase transitions,
 * and integrates with the WASM core for cryptographic operations.
 *
 * # Protocol Flow
 *
 * ```text
 *   Round R:
 *   1. startRound()     → Generate VRF proof, determine if leader
 *   2. propose()        → [Leader] Create block proposal
 *   3. receiveProposal()→ [Follower] Validate leader's proposal
 *   4. prevote()        → Cast prevote for valid proposal
 *   5. receiveVote()    → Aggregate votes from peers
 *   6. precommit()      → Cast precommit after prevote quorum
 *   7. finalize()       → Finalize block after precommit quorum
 *   8. nextRound()      → Advance to round R+1
 * ```
 */

import { VotePool } from './vote-pool';
import { BlockChain } from './block-chain';
import { bytesToHex, bytesEqual, compareBytes } from '$lib/utils/bytes';
import type { CryptoProvider } from './crypto-provider';
import type {
  ConsensusPhase,
  ConsensusState,
  ConsensusConfig,
  ConsensusEvents,
  BlockHeader
} from './types';

/** Default consensus configuration. */
const DEFAULT_CONFIG: ConsensusConfig = {
  roundTimeoutMs: 30_000,
  tau: new Uint8Array(32).fill(0xff), // accept all VRF outputs by default
  checkpointInterval: 1000
};

/**
 * Consensus engine state machine.
 *
 * Coordinates VRF-based leader election with two-phase BFT voting.
 * Each instance represents a single validator node.
 */
export class ConsensusEngine {
  /** This node's VRF secret key (32 bytes). */
  private readonly secretKey: Uint8Array;
  /** This node's VRF public key (32 bytes). */
  private readonly publicKey: Uint8Array;
  /** All validator public keys (including self). */
  private readonly validators: Uint8Array[];
  /** VRF output threshold for leader election. */
  private readonly tau: Uint8Array;
  /** Checkpoint interval in VDF steps. */
  private readonly checkpointInterval: number;
  /** Round timeout in ms. */
  private readonly roundTimeoutMs: number;
  /** Cryptographic operations provider. */
  private readonly crypto: CryptoProvider;

  /** Current round number. */
  private currentRound = 0;
  /** Current phase within the round. */
  private currentPhase: ConsensusPhase = 'propose';
  /** VRF proof for the current round. */
  private currentVrfProof: Uint8Array | null = null;
  /** Whether this node is the leader of the current round. */
  private _isLeader = false;
  /** Whether a round is active. */
  private roundActive = false;
  /** Round timeout timer. */
  private roundTimer: ReturnType<typeof setTimeout> | null = null;

  /** Vote pool for the current round. */
  private votePool: VotePool;
  /** Finalized block chain. */
  private chain: BlockChain;
  /** Current proposal (if any). */
  private currentProposal: BlockHeader | null = null;

  /** Event listeners. */
  private listeners: Partial<ConsensusEvents> = {};

  /**
   * Create a new consensus engine.
   *
   * @param secretKey - 32-byte VRF secret key.
   * @param publicKey - 32-byte VRF public key.
   * @param validators - Array of all validator public keys (including self).
   * @param crypto - Cryptographic operations provider.
   * @param config - Optional configuration overrides.
   */
  constructor(
    secretKey: Uint8Array,
    publicKey: Uint8Array,
    validators: Uint8Array[],
    crypto: CryptoProvider,
    config?: Partial<ConsensusConfig>
  ) {
    if (validators.length === 0) {
      throw new Error('Validator set cannot be empty');
    }

    this.secretKey = secretKey;
    this.publicKey = publicKey;
    this.validators = validators;
    this.crypto = crypto;
    this.tau = config?.tau ?? DEFAULT_CONFIG.tau;
    this.checkpointInterval = config?.checkpointInterval ?? DEFAULT_CONFIG.checkpointInterval;
    this.roundTimeoutMs = config?.roundTimeoutMs ?? DEFAULT_CONFIG.roundTimeoutMs;

    this.votePool = new VotePool(validators.length);
    this.chain = new BlockChain(crypto);
  }

  // ─── Public API ─────────────────────────────────────────────────

  /**
   * Register event listeners.
   */
  on<K extends keyof ConsensusEvents>(event: K, listener: ConsensusEvents[K]): void {
    this.listeners[event] = listener;
  }

  /**
   * Remove event listener.
   */
  off<K extends keyof ConsensusEvents>(event: K): void {
    delete this.listeners[event];
  }

  /**
   * Start a new consensus round.
   *
   * Generates a VRF proof for leader election and initializes the round state.
   * The VRF proof should be broadcast to peers as part of the proposal.
   *
   * @param round - The round number to start.
   * @returns The VRF proof for this round.
   */
  startRound(round: number): Uint8Array {
    this.clearRoundTimer();

    this.currentRound = round;
    this.currentPhase = 'propose';
    this.currentProposal = null;
    this.votePool.clear();
    this.roundActive = true;

    // Generate VRF proof for leader election using the crypto provider
    const roundSeed = this.createRoundSeed(round);
    this.currentVrfProof = this.crypto.vrfProve(this.secretKey, roundSeed);

    // Check if this node is the leader
    this._isLeader = this.checkIsLeader();

    // Start round timeout
    this.startRoundTimer();

    // Emit events
    if (this._isLeader) {
      this.emit('onLeaderElected', round);
    }
    this.emitStateChange();

    return this.currentVrfProof!;
  }

  /**
   * Create a block proposal (leader only).
   *
   * @param vdfState - 32-byte VDF state at the checkpoint.
   * @param timestamp - Unix timestamp in milliseconds.
   * @returns The proposed block header.
   */
  propose(vdfState: Uint8Array, timestamp: number): BlockHeader {
    if (!this._isLeader) {
      throw new Error('Only the leader can propose');
    }
    if (!this.currentVrfProof) {
      throw new Error('No VRF proof — call startRound() first');
    }

    const block: BlockHeader = {
      round: this.currentRound,
      prevHash: this.chain.latestHash,
      vdfState,
      vrfProof: this.currentVrfProof,
      proposer: this.publicKey,
      timestamp,
      hash: new Uint8Array(32) // placeholder
    };

    // Compute block hash
    block.hash = this.chain.computeBlockHash(block);

    // Store as current proposal
    this.currentProposal = block;
    this.currentPhase = 'prevote';
    this.emit('onPhaseChange', this.currentRound, 'prevote');
    this.emitStateChange();

    return block;
  }

  /**
   * Receive and validate a block proposal from the leader.
   *
   * Validates the VRF proof for leader election, verifies the block hash,
   * and checks the prevHash chain link.
   *
   * @param proposal - The block header from the leader.
   * @returns true if the proposal is valid.
   */
  receiveProposal(proposal: BlockHeader): boolean {
    // Verify round matches
    if (proposal.round !== this.currentRound) {
      return false;
    }

    // Verify block hash
    const expectedHash = this.chain.computeBlockHash(proposal);
    if (!bytesEqual(proposal.hash, expectedHash)) {
      return false;
    }

    // Verify prevHash links to our chain
    if (!bytesEqual(proposal.prevHash, this.chain.latestHash)) {
      return false;
    }

    // Verify the VRF proof for leader election
    const roundSeed = this.createRoundSeed(proposal.round);
    if (!this.crypto.vrfVerify(proposal.proposer, roundSeed, proposal.vrfProof)) {
      return false;
    }

    // Verify the proposer's VRF output is below tau (won the lottery)
    const vrfOutput = this.crypto.vrfProofToHash(proposal.vrfProof);
    if (compareBytes(vrfOutput, this.tau) >= 0) {
      return false;
    }

    // Accept the proposal
    this.currentProposal = proposal;
    this.currentPhase = 'prevote';
    this.emit('onProposal', this.currentRound, proposal.hash);
    this.emit('onPhaseChange', this.currentRound, 'prevote');
    this.emitStateChange();

    return true;
  }

  /**
   * Cast a prevote for the current round's proposal.
   *
   * @param accept - Whether to accept the proposal.
   * @returns The vote data to broadcast.
   */
  prevote(accept: boolean): { round: number; phase: 'prevote'; blockHash: Uint8Array | null } {
    if (this.currentPhase !== 'prevote' && this.currentPhase !== 'propose') {
      throw new Error(`Cannot prevote in phase: ${this.currentPhase}`);
    }

    const blockHash = accept && this.currentProposal ? this.currentProposal.hash : null;

    // Also add our own vote to the pool
    this.votePool.addPrevote(this.publicKey, blockHash);

    return {
      round: this.currentRound,
      phase: 'prevote',
      blockHash
    };
  }

  /**
   * Cast a precommit for the current round.
   *
   * @param blockHash - The block hash to precommit.
   * @returns The vote data to broadcast.
   */
  precommit(blockHash: Uint8Array): { round: number; phase: 'precommit'; blockHash: Uint8Array } {
    if (this.currentPhase !== 'precommit' && this.currentPhase !== 'prevote') {
      throw new Error(`Cannot precommit in phase: ${this.currentPhase}`);
    }

    // Add our own vote to the pool
    this.votePool.addPrecommit(this.publicKey, blockHash);

    return {
      round: this.currentRound,
      phase: 'precommit',
      blockHash
    };
  }

  /**
   * Receive and aggregate a vote from another validator.
   *
   * @param round - The round number.
   * @param phase - The vote phase ('prevote' or 'precommit').
   * @param blockHash - The block hash (null for nil vote).
   * @param voterPubkey - The voter's public key.
   * @returns Status: 'accepted', 'duplicate', 'prevote-quorum', 'precommit-quorum', or 'rejected'.
   */
  receiveVote(
    round: number,
    phase: 'prevote' | 'precommit',
    blockHash: Uint8Array | null,
    voterPubkey: Uint8Array
  ): string {
    // Validate round
    if (round !== this.currentRound) {
      return 'rejected';
    }

    // Validate voter is in the validator set
    if (!this.isValidator(voterPubkey)) {
      return 'rejected';
    }

    let result: string;

    if (phase === 'prevote') {
      const voteResult = this.votePool.addPrevote(voterPubkey, blockHash);
      if (voteResult.status === 'duplicate') {
        return 'duplicate';
      }
      if (voteResult.status === 'quorum') {
        this.currentPhase = 'precommit';
        this.emit('onPhaseChange', this.currentRound, 'precommit');
        this.emitStateChange();
        return 'prevote-quorum';
      }
      result = 'accepted';
    } else {
      const voteResult = this.votePool.addPrecommit(voterPubkey, blockHash);
      if (voteResult.status === 'duplicate') {
        return 'duplicate';
      }
      if (voteResult.status === 'quorum') {
        this.currentPhase = 'commit';
        this.emit('onPhaseChange', this.currentRound, 'commit');
        this.emitStateChange();
        return 'precommit-quorum';
      }
      result = 'accepted';
    }

    this.emitStateChange();
    return result;
  }

  /**
   * Finalize the current round's block.
   *
   * Should be called after receiving a precommit quorum. Adds the block
   * to the finalized chain.
   *
   * @returns The finalized block header.
   */
  finalize(): BlockHeader {
    if (this.currentPhase !== 'commit') {
      throw new Error(`Cannot finalize in phase: ${this.currentPhase}`);
    }
    if (!this.currentProposal) {
      throw new Error('No proposal to finalize');
    }

    // Verify precommit quorum
    const quorumHash = this.votePool.getPrecommitQuorum();
    if (!quorumHash || !bytesEqual(quorumHash, this.currentProposal.hash)) {
      throw new Error('No precommit quorum for current proposal');
    }

    // Add to chain
    const block = this.currentProposal;
    this.chain.append(block);

    // Clear round timer
    this.clearRoundTimer();

    // Emit commit event
    this.emit('onCommit', this.currentRound, block);

    return block;
  }

  /**
   * Get the current consensus state.
   */
  getState(): ConsensusState {
    return {
      round: this.currentRound,
      phase: this.currentPhase,
      isLeader: this._isLeader,
      prevoteCount: this.votePool.prevoteCount,
      precommitCount: this.votePool.precommitCount,
      chainHeight: this.chain.height
    };
  }

  /** Get the current round number. */
  get round(): number {
    return this.currentRound;
  }

  /** Get the current phase. */
  get phase(): ConsensusPhase {
    return this.currentPhase;
  }

  /** Check if this node is the current round's leader. */
  get isLeader(): boolean {
    return this._isLeader;
  }

  /** Get the finalized chain. */
  get blockChain(): BlockChain {
    return this.chain;
  }

  /** Get the chain height. */
  get chainHeight(): number {
    return this.chain.height;
  }

  /** Get the quorum threshold. */
  get threshold(): number {
    return this.votePool.quorumThreshold;
  }

  /** Get the number of prevotes collected. */
  get prevoteCount(): number {
    return this.votePool.prevoteCount;
  }

  /** Get the number of precommits collected. */
  get precommitCount(): number {
    return this.votePool.precommitCount;
  }

  /**
   * Cleanup resources (timers, etc.).
   */
  dispose(): void {
    this.clearRoundTimer();
  }

  // ─── Private Helpers ──────────────────────────────────────────

  /**
   * Create a deterministic round seed for VRF proof generation.
   * The seed encodes the round number as a big-endian 8-byte value.
   */
  private createRoundSeed(round: number): Uint8Array {
    const seed = new Uint8Array(8);
    new DataView(seed.buffer).setBigUint64(0, BigInt(round), false);
    return seed;
  }

  /**
   * Check if this node is the leader for the current round.
   *
   * Derives the real VRF output from the proof and compares against tau.
   */
  private checkIsLeader(): boolean {
    if (!this.currentVrfProof) return false;

    const vrfOutput = this.crypto.vrfProofToHash(this.currentVrfProof);
    return compareBytes(vrfOutput, this.tau) < 0;
  }

  /**
   * Check if a public key is in the validator set.
   */
  private isValidator(pubkey: Uint8Array): boolean {
    const hex = bytesToHex(pubkey);
    return this.validators.some((v) => bytesToHex(v) === hex);
  }

  /**
   * Start the round timeout timer.
   */
  private startRoundTimer(): void {
    this.clearRoundTimer();
    this.roundTimer = setTimeout(() => {
      this.emit('onRoundTimeout', this.currentRound);
    }, this.roundTimeoutMs);
  }

  /**
   * Clear the round timeout timer.
   */
  private clearRoundTimer(): void {
    if (this.roundTimer) {
      clearTimeout(this.roundTimer);
      this.roundTimer = null;
    }
  }

  /**
   * Emit a state change event.
   */
  private emitStateChange(): void {
    this.emit('onStateChange', this.getState());
  }

  /**
   * Emit an event to registered listeners.
   */
  private emit<K extends keyof ConsensusEvents>(
    event: K,
    ...args: Parameters<ConsensusEvents[K]>
  ): void {
    const listener = this.listeners[event];
    if (listener) {
      (listener as (...a: unknown[]) => void)(...args);
    }
  }
}
