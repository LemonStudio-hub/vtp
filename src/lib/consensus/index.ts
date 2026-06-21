/**
 * Consensus Module
 *
 * VRF-driven leader election + BFT validation consensus protocol.
 *
 * Architecture:
 * ```
 *   ConsensusEngine          VotePool
 *   ┌─────────────────┐      ┌──────────────────┐
 *   │  Round lifecycle │      │  Prevote tracking │
 *   │  Leader election │      │  Precommit track  │
 *   │  Phase FSM       │──────│  Quorum detection │
 *   │  Proposal mgmt   │      └──────────────────┘
 *   └────────┬────────┘
 *            │
 *            ▼
 *   BlockChain
 *   ┌─────────────────┐
 *   │  Finalized chain │
 *   │  Hash verification│
 *   │  Block queries    │
 *   └─────────────────┘
 * ```
 *
 * @example
 * ```typescript
 * import { ConsensusEngine } from '$lib/consensus';
 *
 * const engine = new ConsensusEngine(secretKey, publicKey, validators, {
 *   tau: new Uint8Array(32).fill(0xff),
 *   checkpointInterval: 1000,
 *   roundTimeoutMs: 30_000,
 * });
 *
 * // Register events
 * engine.on('onCommit', (round, block) => {
 *   console.log(`Block finalized at round ${round}`);
 * });
 *
 * // Start a round
 * const vrfProof = engine.startRound(1);
 *
 * if (engine.isLeader) {
 *   const proposal = engine.propose(vdfState, Date.now());
 *   // broadcast proposal to peers
 * }
 * ```
 */

export { ConsensusEngine } from './consensus-engine';
export { VotePool } from './vote-pool';
export { BlockChain, computeBlockHash } from './block-chain';
export { createWasmCryptoProvider, createPlaceholderCryptoProvider } from './crypto-provider';
export type { CryptoProvider } from './crypto-provider';
export type {
  ConsensusPhase,
  VotePhase,
  BlockHeader,
  ConsensusState,
  ConsensusConfig,
  ConsensusEvents,
  ConsensusProposalPayload,
  ConsensusVotePayload,
  NewRoundPayload
} from './types';
