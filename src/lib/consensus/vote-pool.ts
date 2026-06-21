/**
 * Vote Pool
 *
 * Collects and aggregates prevotes and precommits for a consensus round.
 * Tracks unique voters (dedup by public key) and detects quorum thresholds.
 *
 * The quorum formula is: threshold = 2 * floor((n-1)/3) + 1
 * This ensures BFT safety — at most one block can be finalized per round.
 */

import { bytesToHex } from '$lib/utils/bytes';

/** A single vote entry. */
export interface VoteEntry {
  /** The voter's 32-byte public key. */
  voter: Uint8Array;
  /** The block hash being voted on (null = nil vote). */
  blockHash: Uint8Array | null;
  /** Timestamp when the vote was received. */
  receivedAt: number;
}

/** Result of adding a vote to the pool. */
export type VoteResult =
  | { status: 'accepted' }
  | { status: 'duplicate' }
  | { status: 'quorum'; blockHash: Uint8Array };

/**
 * Vote pool for a single consensus round.
 *
 * Manages two independent pools — prevotes and precommits — and tracks
 * quorum thresholds for each.
 */
export class VotePool {
  /** Map from voter pubkey hex → prevote entry. */
  private prevotes = new Map<string, VoteEntry>();
  /** Map from voter pubkey hex → precommit entry. */
  private precommits = new Map<string, VoteEntry>();
  /** Quorum threshold (2f+1). */
  private readonly threshold: number;
  /** Number of validators. */
  private readonly validatorCount: number;

  constructor(validatorCount: number) {
    this.validatorCount = validatorCount;
    this.threshold = VotePool.computeThreshold(validatorCount);
  }

  /**
   * Compute the BFT quorum threshold for a given number of validators.
   * threshold = 2 * floor((n-1)/3) + 1
   */
  static computeThreshold(n: number): number {
    if (n === 0) return 0;
    const f = Math.floor((n - 1) / 3);
    return 2 * f + 1;
  }

  /**
   * Add a prevote to the pool.
   *
   * @param voter - 32-byte public key of the voter.
   * @param blockHash - The block hash being voted on, or null for nil vote.
   * @returns Whether the vote was accepted and if quorum was reached.
   */
  addPrevote(voter: Uint8Array, blockHash: Uint8Array | null): VoteResult {
    const key = bytesToHex(voter);
    if (this.prevotes.has(key)) {
      return { status: 'duplicate' };
    }

    this.prevotes.set(key, {
      voter,
      blockHash,
      receivedAt: Date.now()
    });

    // Check quorum for non-nil votes
    if (blockHash) {
      const count = this.countVotesFor(this.prevotes, blockHash);
      if (count >= this.threshold) {
        return { status: 'quorum', blockHash };
      }
    }

    return { status: 'accepted' };
  }

  /**
   * Add a precommit to the pool.
   *
   * @param voter - 32-byte public key of the voter.
   * @param blockHash - The block hash being voted on, or null for nil vote.
   * @returns Whether the vote was accepted and if quorum was reached.
   */
  addPrecommit(voter: Uint8Array, blockHash: Uint8Array | null): VoteResult {
    const key = bytesToHex(voter);
    if (this.precommits.has(key)) {
      return { status: 'duplicate' };
    }

    this.precommits.set(key, {
      voter,
      blockHash,
      receivedAt: Date.now()
    });

    // Check quorum for non-nil votes
    if (blockHash) {
      const count = this.countVotesFor(this.precommits, blockHash);
      if (count >= this.threshold) {
        return { status: 'quorum', blockHash };
      }
    }

    return { status: 'accepted' };
  }

  /** Get the number of prevotes collected. */
  get prevoteCount(): number {
    return this.prevotes.size;
  }

  /** Get the number of precommits collected. */
  get precommitCount(): number {
    return this.precommits.size;
  }

  /** Get the quorum threshold. */
  get quorumThreshold(): number {
    return this.threshold;
  }

  /** Get the number of validators. */
  get size(): number {
    return this.validatorCount;
  }

  /**
   * Check if prevote quorum has been reached for any block hash.
   *
   * @returns The block hash that achieved quorum, or null.
   */
  getPrevoteQuorum(): Uint8Array | null {
    return this.findQuorum(this.prevotes);
  }

  /**
   * Check if precommit quorum has been reached for any block hash.
   *
   * @returns The block hash that achieved quorum, or null.
   */
  getPrecommitQuorum(): Uint8Array | null {
    return this.findQuorum(this.precommits);
  }

  /**
   * Check if a specific voter has already prevoted.
   */
  hasPrevoted(voter: Uint8Array): boolean {
    return this.prevotes.has(bytesToHex(voter));
  }

  /**
   * Check if a specific voter has already precommitted.
   */
  hasPrecommitted(voter: Uint8Array): boolean {
    return this.precommits.has(bytesToHex(voter));
  }

  /** Clear all votes (for round transition). */
  clear(): void {
    this.prevotes.clear();
    this.precommits.clear();
  }

  // ─── Private Helpers ──────────────────────────────────────────

  /**
   * Count votes in a pool for a specific block hash.
   */
  private countVotesFor(pool: Map<string, VoteEntry>, blockHash: Uint8Array): number {
    const targetHex = bytesToHex(blockHash);
    let count = 0;
    for (const entry of pool.values()) {
      if (entry.blockHash && bytesToHex(entry.blockHash) === targetHex) {
        count++;
      }
    }
    return count;
  }

  /**
   * Find a block hash that has achieved quorum in a pool.
   */
  private findQuorum(pool: Map<string, VoteEntry>): Uint8Array | null {
    const counts = new Map<string, { hash: Uint8Array; count: number }>();

    for (const entry of pool.values()) {
      if (!entry.blockHash) continue;
      const hex = bytesToHex(entry.blockHash);
      const existing = counts.get(hex);
      if (existing) {
        existing.count++;
      } else {
        counts.set(hex, { hash: entry.blockHash, count: 1 });
      }
    }

    for (const { hash, count } of counts.values()) {
      if (count >= this.threshold) {
        return hash;
      }
    }

    return null;
  }
}
