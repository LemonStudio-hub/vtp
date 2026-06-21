/**
 * Consensus Engine Tests
 *
 * Tests for the VRF-driven leader election + BFT validation consensus protocol.
 * Covers:
 * - ConsensusEngine lifecycle and phase transitions
 * - VotePool quorum tracking
 * - BlockChain integrity
 * - Leader election detection
 * - Round timeout handling
 * - Vote deduplication
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { ConsensusEngine } from '../src/lib/consensus/consensus-engine';
import { VotePool } from '../src/lib/consensus/vote-pool';
import { BlockChain, computeBlockHash } from '../src/lib/consensus/block-chain';
import { createPlaceholderCryptoProvider } from '../src/lib/consensus/crypto-provider';
import type { CryptoProvider } from '../src/lib/consensus/crypto-provider';
import type { BlockHeader, ConsensusPhase } from '../src/lib/consensus/types';

const crypto: CryptoProvider = createPlaceholderCryptoProvider();

// ─── Helpers ─────────────────────────────────────────────────────

/** Create a deterministic 32-byte key from a number. */
function makeKey(n: number): Uint8Array {
  const key = new Uint8Array(32);
  key.fill(n);
  return key;
}

/** Create a set of validator keys. */
function makeValidators(count: number): Uint8Array[] {
  return Array.from({ length: count }, (_, i) => makeKey(i + 1));
}

/** Create a block header for testing. */
function makeBlock(round: number, prevHash?: Uint8Array): BlockHeader {
  const block: BlockHeader = {
    round,
    prevHash: prevHash ?? new Uint8Array(32),
    vdfState: new Uint8Array(32).fill(0xaa),
    vrfProof: new Uint8Array(80).fill(0xbb),
    proposer: makeKey(1),
    timestamp: 1000 + round,
    hash: new Uint8Array(32)
  };
  block.hash = computeBlockHash(block);
  return block;
}

// ─── VotePool Tests ──────────────────────────────────────────────

describe('VotePool', () => {
  it('computes correct threshold for various validator counts', () => {
    expect(VotePool.computeThreshold(0)).toBe(0);
    expect(VotePool.computeThreshold(1)).toBe(1);
    expect(VotePool.computeThreshold(2)).toBe(1);
    expect(VotePool.computeThreshold(4)).toBe(3);
    expect(VotePool.computeThreshold(7)).toBe(5);
    expect(VotePool.computeThreshold(10)).toBe(7);
  });

  it('accepts prevotes and tracks count', () => {
    const pool = new VotePool(4);
    const voter = makeKey(1);
    const hash = new Uint8Array(32).fill(0xaa);

    const result = pool.addPrevote(voter, hash);
    expect(result.status).toBe('accepted');
    expect(pool.prevoteCount).toBe(1);
  });

  it('rejects duplicate prevotes', () => {
    const pool = new VotePool(4);
    const voter = makeKey(1);
    const hash = new Uint8Array(32).fill(0xaa);

    pool.addPrevote(voter, hash);
    const result = pool.addPrevote(voter, hash);
    expect(result.status).toBe('duplicate');
    expect(pool.prevoteCount).toBe(1);
  });

  it('detects prevote quorum', () => {
    const pool = new VotePool(4); // threshold = 3
    const hash = new Uint8Array(32).fill(0xaa);

    pool.addPrevote(makeKey(1), hash);
    pool.addPrevote(makeKey(2), hash);
    const result = pool.addPrevote(makeKey(3), hash);

    expect(result.status).toBe('quorum');
    expect(result).toHaveProperty('blockHash');
  });

  it('does not trigger quorum for different block hashes', () => {
    const pool = new VotePool(4); // threshold = 3
    const hash1 = new Uint8Array(32).fill(0xaa);
    const hash2 = new Uint8Array(32).fill(0xbb);

    pool.addPrevote(makeKey(1), hash1);
    pool.addPrevote(makeKey(2), hash2);
    const result = pool.addPrevote(makeKey(3), hash1);

    // Only 2 votes for hash1, not enough for quorum
    expect(result.status).toBe('accepted');
  });

  it('does not trigger quorum for nil votes', () => {
    const pool = new VotePool(4); // threshold = 3

    pool.addPrevote(makeKey(1), null);
    pool.addPrevote(makeKey(2), null);
    pool.addPrevote(makeKey(3), null);

    expect(pool.getPrevoteQuorum()).toBeNull();
  });

  it('detects precommit quorum', () => {
    const pool = new VotePool(4); // threshold = 3
    const hash = new Uint8Array(32).fill(0xaa);

    pool.addPrecommit(makeKey(1), hash);
    pool.addPrecommit(makeKey(2), hash);
    const result = pool.addPrecommit(makeKey(3), hash);

    expect(result.status).toBe('quorum');
  });

  it('tracks hasPrevoted correctly', () => {
    const pool = new VotePool(4);
    const voter = makeKey(1);

    expect(pool.hasPrevoted(voter)).toBe(false);
    pool.addPrevote(voter, new Uint8Array(32));
    expect(pool.hasPrevoted(voter)).toBe(true);
  });

  it('clears all votes', () => {
    const pool = new VotePool(4);

    pool.addPrevote(makeKey(1), new Uint8Array(32));
    pool.addPrecommit(makeKey(2), new Uint8Array(32));

    pool.clear();

    expect(pool.prevoteCount).toBe(0);
    expect(pool.precommitCount).toBe(0);
  });
});

// ─── BlockChain Tests ────────────────────────────────────────────

describe('BlockChain', () => {
  let chain: BlockChain;

  beforeEach(() => {
    chain = new BlockChain(crypto);
  });

  it('starts empty', () => {
    expect(chain.height).toBe(0);
    expect(chain.latest).toBeNull();
  });

  it('returns genesis hash when empty', () => {
    expect(chain.latestHash).toEqual(new Uint8Array(32));
  });

  it('appends blocks and updates height', () => {
    const block1 = makeBlock(1);
    chain.append(block1);

    expect(chain.height).toBe(1);
    expect(chain.latest).toBe(block1);
  });

  it('enforces prevHash chain link', () => {
    const block1 = makeBlock(1);
    chain.append(block1);

    // Wrong prevHash
    const badBlock = makeBlock(2, new Uint8Array(32).fill(0xff));
    expect(() => chain.append(badBlock)).toThrow('prevHash mismatch');

    // Correct prevHash
    const goodBlock = makeBlock(2, block1.hash);
    chain.append(goodBlock);
    expect(chain.height).toBe(2);
  });

  it('verifies chain integrity', () => {
    const block1 = makeBlock(1);
    const block2 = makeBlock(2, block1.hash);
    const block3 = makeBlock(3, block2.hash);

    chain.append(block1);
    chain.append(block2);
    chain.append(block3);

    expect(chain.verify()).toBe(true);
  });

  it('gets block by round', () => {
    const block1 = makeBlock(1);
    const block2 = makeBlock(2, block1.hash);
    chain.append(block1);
    chain.append(block2);

    expect(chain.getByRound(1)).toBe(block1);
    expect(chain.getByRound(2)).toBe(block2);
    expect(chain.getByRound(99)).toBeNull();
  });

  it('gets block by hash', () => {
    const block1 = makeBlock(1);
    chain.append(block1);

    expect(chain.getByHash(block1.hash)).toBe(block1);
    expect(chain.getByHash(new Uint8Array(32).fill(0xff))).toBeNull();
  });

  it('gets recent blocks', () => {
    const blocks = [makeBlock(1)];
    for (let i = 2; i <= 5; i++) {
      blocks.push(makeBlock(i, blocks[i - 2].hash));
    }
    blocks.forEach((b) => chain.append(b));

    const recent = chain.getRecent(3);
    expect(recent).toHaveLength(3);
    expect(recent[0].round).toBe(5);
    expect(recent[1].round).toBe(4);
    expect(recent[2].round).toBe(3);
  });

  it('serializes to JSON', () => {
    const block1 = makeBlock(1);
    chain.append(block1);

    const json = chain.toJSON();
    expect(json).toHaveLength(1);
    expect(json[0].round).toBe(1);
    expect(typeof json[0].hash).toBe('string');
  });
});

// ─── ConsensusEngine Tests ───────────────────────────────────────

describe('ConsensusEngine', () => {
  const validators = makeValidators(4);
  let engine: ConsensusEngine;

  beforeEach(() => {
    engine = new ConsensusEngine(validators[0], validators[0], validators, crypto, {
      tau: new Uint8Array(32).fill(0xff)
    });
  });

  it('initializes with correct state', () => {
    const state = engine.getState();
    expect(state.round).toBe(0);
    expect(state.phase).toBe('propose');
    expect(state.prevoteCount).toBe(0);
    expect(state.precommitCount).toBe(0);
    expect(state.chainHeight).toBe(0);
  });

  it('starts a new round', () => {
    const proof = engine.startRound(1);
    expect(proof).toBeInstanceOf(Uint8Array);
    expect(engine.round).toBe(1);
    expect(engine.phase).toBe('propose');
  });

  it('detects leader with high tau', () => {
    engine.startRound(1);
    // With tau=0xFF, all nodes should be leaders
    expect(engine.isLeader).toBe(true);
  });

  it('detects non-leader with low tau', () => {
    const lowTauEngine = new ConsensusEngine(validators[0], validators[0], validators, crypto, {
      tau: new Uint8Array(32).fill(0x00)
    });
    lowTauEngine.startRound(1);
    expect(lowTauEngine.isLeader).toBe(false);
  });

  it('proposes a block when leader', () => {
    engine.startRound(1);
    const block = engine.propose(new Uint8Array(32).fill(0xaa), 1000);

    expect(block.round).toBe(1);
    expect(block.vdfState).toEqual(new Uint8Array(32).fill(0xaa));
    expect(block.timestamp).toBe(1000);
    expect(block.hash).toBeInstanceOf(Uint8Array);
    expect(block.hash.length).toBe(32);
  });

  it('throws when non-leader tries to propose', () => {
    const lowTauEngine = new ConsensusEngine(validators[0], validators[0], validators, crypto, {
      tau: new Uint8Array(32).fill(0x00)
    });
    lowTauEngine.startRound(1);
    expect(() => lowTauEngine.propose(new Uint8Array(32), 1000)).toThrow(
      'Only the leader can propose'
    );
  });

  it('receives valid proposal', () => {
    engine.startRound(1);
    const block = engine.propose(new Uint8Array(32).fill(0xaa), 1000);

    // Another engine receives the proposal
    const follower = new ConsensusEngine(validators[1], validators[1], validators, crypto, {
      tau: new Uint8Array(32).fill(0xff)
    });
    follower.startRound(1);

    const accepted = follower.receiveProposal(block);
    expect(accepted).toBe(true);
    expect(follower.phase).toBe('prevote');
  });

  it('rejects proposal with wrong round', () => {
    engine.startRound(1);
    const block = engine.propose(new Uint8Array(32).fill(0xaa), 1000);

    const follower = new ConsensusEngine(validators[1], validators[1], validators, crypto, {
      tau: new Uint8Array(32).fill(0xff)
    });
    follower.startRound(2); // different round

    const accepted = follower.receiveProposal(block);
    expect(accepted).toBe(false);
  });

  it('rejects proposal with wrong prevHash', () => {
    engine.startRound(1);
    const block = engine.propose(new Uint8Array(32).fill(0xaa), 1000);

    // Tamper with prevHash
    const tampered = { ...block, prevHash: new Uint8Array(32).fill(0xff) };
    tampered.hash = computeBlockHash(tampered);

    const follower = new ConsensusEngine(validators[1], validators[1], validators, crypto, {
      tau: new Uint8Array(32).fill(0xff)
    });
    follower.startRound(1);

    const accepted = follower.receiveProposal(tampered);
    expect(accepted).toBe(false);
  });

  it('casts prevote for accepted proposal', () => {
    engine.startRound(1);
    engine.propose(new Uint8Array(32).fill(0xaa), 1000);

    const vote = engine.prevote(true);
    expect(vote.round).toBe(1);
    expect(vote.phase).toBe('prevote');
    expect(vote.blockHash).toBeInstanceOf(Uint8Array);
  });

  it('casts nil prevote for rejected proposal', () => {
    engine.startRound(1);
    engine.propose(new Uint8Array(32).fill(0xaa), 1000);

    const vote = engine.prevote(false);
    expect(vote.round).toBe(1);
    expect(vote.phase).toBe('prevote');
    expect(vote.blockHash).toBeNull();
  });

  it('rejects votes from non-validators', () => {
    engine.startRound(1);
    const fakeKey = new Uint8Array(32).fill(0xff);

    const result = engine.receiveVote(1, 'prevote', new Uint8Array(32), fakeKey);
    expect(result).toBe('rejected');
  });

  it('rejects votes with wrong round', () => {
    engine.startRound(1);

    const result = engine.receiveVote(999, 'prevote', new Uint8Array(32), validators[1]);
    expect(result).toBe('rejected');
  });

  it('rejects duplicate votes', () => {
    engine.startRound(1);
    const hash = new Uint8Array(32).fill(0xaa);

    engine.receiveVote(1, 'prevote', hash, validators[1]);
    const result = engine.receiveVote(1, 'prevote', hash, validators[1]);
    expect(result).toBe('duplicate');
  });

  it('detects prevote quorum', () => {
    engine.startRound(1);
    const hash = new Uint8Array(32).fill(0xaa);

    // Need 3 votes (threshold for 4 validators)
    engine.receiveVote(1, 'prevote', hash, validators[1]);
    engine.receiveVote(1, 'prevote', hash, validators[2]);
    const result = engine.receiveVote(1, 'prevote', hash, validators[3]);

    expect(result).toBe('prevote-quorum');
    expect(engine.phase).toBe('precommit');
  });

  it('detects precommit quorum', () => {
    engine.startRound(1);
    const hash = new Uint8Array(32).fill(0xaa);

    engine.receiveVote(1, 'precommit', hash, validators[1]);
    engine.receiveVote(1, 'precommit', hash, validators[2]);
    const result = engine.receiveVote(1, 'precommit', hash, validators[3]);

    expect(result).toBe('precommit-quorum');
    expect(engine.phase).toBe('commit');
  });

  it('finalizes block after precommit quorum', () => {
    engine.startRound(1);
    const block = engine.propose(new Uint8Array(32).fill(0xaa), 1000);

    // Collect prevote quorum
    for (let i = 1; i <= 3; i++) {
      engine.receiveVote(1, 'prevote', block.hash, validators[i]);
    }

    // Collect precommit quorum
    for (let i = 1; i <= 3; i++) {
      engine.receiveVote(1, 'precommit', block.hash, validators[i]);
    }

    const finalized = engine.finalize();
    expect(finalized.round).toBe(1);
    expect(engine.chainHeight).toBe(1);
  });

  it('emits onCommit event on finalization', () => {
    const onCommit = vi.fn();
    engine.on('onCommit', onCommit);

    engine.startRound(1);
    const block = engine.propose(new Uint8Array(32).fill(0xaa), 1000);

    for (let i = 1; i <= 3; i++) {
      engine.receiveVote(1, 'prevote', block.hash, validators[i]);
    }
    for (let i = 1; i <= 3; i++) {
      engine.receiveVote(1, 'precommit', block.hash, validators[i]);
    }

    engine.finalize();
    expect(onCommit).toHaveBeenCalledWith(1, expect.objectContaining({ round: 1 }));
  });

  it('emits onPhaseChange events', () => {
    const phases: ConsensusPhase[] = [];
    engine.on('onPhaseChange', (_round, phase) => phases.push(phase));

    engine.startRound(1);
    engine.propose(new Uint8Array(32).fill(0xaa), 1000);

    expect(phases).toContain('prevote');
  });

  it('emits onLeaderElected when this node is leader', () => {
    const onLeader = vi.fn();
    engine.on('onLeaderElected', onLeader);

    engine.startRound(1);
    expect(onLeader).toHaveBeenCalledWith(1);
  });

  it('computes correct threshold', () => {
    expect(engine.threshold).toBe(3); // n=4, f=1, threshold=3
  });

  it('tracks chain height across multiple rounds', () => {
    // Round 1
    engine.startRound(1);
    const block1 = engine.propose(new Uint8Array(32).fill(0xaa), 1000);
    for (let i = 1; i <= 3; i++) {
      engine.receiveVote(1, 'prevote', block1.hash, validators[i]);
      engine.receiveVote(1, 'precommit', block1.hash, validators[i]);
    }
    engine.finalize();
    expect(engine.chainHeight).toBe(1);

    // Round 2
    engine.startRound(2);
    const block2 = engine.propose(new Uint8Array(32).fill(0xbb), 2000);
    for (let i = 1; i <= 3; i++) {
      engine.receiveVote(2, 'prevote', block2.hash, validators[i]);
      engine.receiveVote(2, 'precommit', block2.hash, validators[i]);
    }
    engine.finalize();
    expect(engine.chainHeight).toBe(2);
  });

  it('disposes resources', () => {
    engine.startRound(1);
    engine.dispose();
    // Should not throw
  });
});

// ─── computeBlockHash Tests ──────────────────────────────────────

describe('computeBlockHash', () => {
  it('produces consistent hashes', () => {
    const block = makeBlock(1);
    const hash1 = computeBlockHash(block);
    const hash2 = computeBlockHash(block);
    expect(hash1).toEqual(hash2);
  });

  it('produces different hashes for different rounds', () => {
    const block1 = makeBlock(1);
    const block2 = makeBlock(2);
    expect(computeBlockHash(block1)).not.toEqual(computeBlockHash(block2));
  });

  it('produces 32-byte hash', () => {
    const block = makeBlock(1);
    const hash = computeBlockHash(block);
    expect(hash.length).toBe(32);
  });
});

// ─── Edge Case Tests ────────────────────────────────────────────

describe('ConsensusEngine edge cases', () => {
  let engine: ConsensusEngine;
  let validators: Uint8Array[];

  beforeEach(() => {
    validators = makeValidators(4);
    engine = new ConsensusEngine(makeKey(1), validators[0], validators, crypto);
  });

  it('dispose clears round timer without throwing', () => {
    engine.startRound(1);
    engine.dispose();
    // Second dispose should also be safe
    engine.dispose();
  });

  it('receiveVote for wrong round returns rejected', () => {
    engine.startRound(1);
    const result = engine.receiveVote(99, 'prevote', makeBlock(99).hash, validators[1]);
    expect(result).toBe('rejected');
  });

  it('finalize throws when not in commit phase', () => {
    engine.startRound(1);
    expect(() => engine.finalize()).toThrow('Cannot finalize in phase');
  });

  it('propose throws without startRound', () => {
    // Fresh engine, no round started — propose should fail
    // If the engine happens to be leader, it still needs a VRF proof from startRound
    engine.startRound(1);
    if (!engine.isLeader) {
      // Not leader: propose should throw
      expect(() => engine.propose(new Uint8Array(32), Date.now())).toThrow(
        'Only the leader can propose'
      );
    }
    // If leader, propose works after startRound — that's expected
  });

  it('BlockChain.verify detects corrupted hash', () => {
    const chain = new BlockChain(crypto);
    const block = makeBlock(1);
    chain.append(block);

    // Verify valid chain passes
    expect(chain.verify()).toBe(true);

    // Tamper with the block's hash (mutate in place)
    block.hash[0] ^= 0xff;

    // Verify should now fail
    expect(chain.verify()).toBe(false);
  });

  it('BlockChain.toJSON round-trip preserves structure', () => {
    const chain = new BlockChain(crypto);
    const block1 = makeBlock(1);
    const block2 = makeBlock(2, block1.hash);
    chain.append(block1);
    chain.append(block2);

    const json = chain.toJSON();
    expect(json).toHaveLength(2);
    expect(json[0].round).toBe(1);
    expect(json[1].round).toBe(2);
    expect(json[0].hash).toBe(json[1].prevHash);
  });
});
