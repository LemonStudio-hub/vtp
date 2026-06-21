/**
 * Block Chain
 *
 * Stores the sequence of finalized block headers, forming an immutable chain
 * where each block references the hash of its predecessor. Provides chain
 * queries and integrity verification.
 */

import { bytesToHex, bytesEqual } from '$lib/utils/bytes';
import type { CryptoProvider } from './crypto-provider';
import type { BlockHeader } from './types';

/**
 * Finalized block chain.
 *
 * Maintains an ordered list of finalized blocks with hash chain verification.
 * Each block's `prevHash` must match the previous block's `hash`.
 */
export class BlockChain {
  /** Ordered list of finalized block headers. */
  private blocks: BlockHeader[] = [];

  /** Cryptographic operations provider. */
  private readonly crypto: CryptoProvider;

  /** Genesis hash (all zeros). */
  static readonly GENESIS_HASH = new Uint8Array(32);

  constructor(crypto: CryptoProvider) {
    this.crypto = crypto;
  }

  /** Get the number of finalized blocks. */
  get height(): number {
    return this.blocks.length;
  }

  /**
   * Get the latest block hash.
   * Returns the genesis hash (all zeros) if the chain is empty.
   */
  get latestHash(): Uint8Array {
    if (this.blocks.length === 0) {
      return BlockChain.GENESIS_HASH;
    }
    return this.blocks[this.blocks.length - 1].hash;
  }

  /**
   * Get the latest finalized block.
   * Returns null if the chain is empty.
   */
  get latest(): BlockHeader | null {
    return this.blocks.length > 0 ? this.blocks[this.blocks.length - 1] : null;
  }

  /**
   * Append a finalized block to the chain.
   *
   * @param block - The block header to append.
   * @throws If the block's prevHash doesn't match the chain's latest hash.
   */
  append(block: BlockHeader): void {
    const expectedPrevHash = this.latestHash;
    if (!bytesEqual(block.prevHash, expectedPrevHash)) {
      throw new Error(
        `Block prevHash mismatch: expected ${bytesToHex(expectedPrevHash)}, got ${bytesToHex(block.prevHash)}`
      );
    }
    this.blocks.push(block);
  }

  /**
   * Get a block by its round number.
   *
   * @param round - The round number.
   * @returns The block header, or null if not found.
   */
  getByRound(round: number): BlockHeader | null {
    return this.blocks.find((b) => b.round === round) ?? null;
  }

  /**
   * Get a block by its hash.
   *
   * @param hash - The 32-byte block hash.
   * @returns The block header, or null if not found.
   */
  getByHash(hash: Uint8Array): BlockHeader | null {
    const hex = bytesToHex(hash);
    return this.blocks.find((b) => bytesToHex(b.hash) === hex) ?? null;
  }

  /**
   * Get the last N blocks.
   *
   * @param count - Number of blocks to return.
   * @returns Array of block headers (most recent first).
   */
  getRecent(count: number): BlockHeader[] {
    return this.blocks.slice(-count).reverse();
  }

  /**
   * Verify the integrity of the entire chain.
   *
   * Checks:
   * 1. Each block's hash matches its computed hash.
   * 2. Each block's prevHash matches the previous block's hash.
   * 3. The genesis block's prevHash is all zeros.
   *
   * @returns true if the chain is valid.
   */
  verify(): boolean {
    for (let i = 0; i < this.blocks.length; i++) {
      const block = this.blocks[i];

      // Verify hash matches
      const computedHash = this.computeBlockHash(block);
      if (!bytesEqual(block.hash, computedHash)) {
        return false;
      }

      // Verify prevHash link
      if (i === 0) {
        if (!bytesEqual(block.prevHash, BlockChain.GENESIS_HASH)) {
          return false;
        }
      } else {
        if (!bytesEqual(block.prevHash, this.blocks[i - 1].hash)) {
          return false;
        }
      }
    }

    return true;
  }

  /**
   * Compute the SHA-256 hash of a block header.
   *
   * The hash covers: round + prevHash + vdfState + vrfProof + proposer + timestamp.
   * Uses the crypto provider's hashBytes for real SHA-256.
   *
   * @param block - The block header to hash.
   * @returns 32-byte hash.
   */
  computeBlockHash(block: BlockHeader): Uint8Array {
    // Build the hash input: round (8B) + prevHash + vdfState + vrfProof + proposer + timestamp (8B)
    const roundBytes = new Uint8Array(8);
    new DataView(roundBytes.buffer).setBigUint64(0, BigInt(block.round), false);

    const timestampBytes = new Uint8Array(8);
    new DataView(timestampBytes.buffer).setBigUint64(0, BigInt(block.timestamp), false);

    const totalLen =
      8 +
      block.prevHash.length +
      block.vdfState.length +
      block.vrfProof.length +
      block.proposer.length +
      8;

    const data = new Uint8Array(totalLen);
    let offset = 0;
    data.set(roundBytes, offset);
    offset += 8;
    data.set(block.prevHash, offset);
    offset += block.prevHash.length;
    data.set(block.vdfState, offset);
    offset += block.vdfState.length;
    data.set(block.vrfProof, offset);
    offset += block.vrfProof.length;
    data.set(block.proposer, offset);
    offset += block.proposer.length;
    data.set(timestampBytes, offset);

    return this.crypto.hashBytes(data);
  }

  /**
   * Serialize the chain to a JSON-compatible format.
   */
  toJSON(): Array<{
    round: number;
    prevHash: string;
    vdfState: string;
    vrfProof: string;
    proposer: string;
    timestamp: number;
    hash: string;
  }> {
    return this.blocks.map((b) => ({
      round: b.round,
      prevHash: bytesToHex(b.prevHash),
      vdfState: bytesToHex(b.vdfState),
      vrfProof: bytesToHex(b.vrfProof),
      proposer: bytesToHex(b.proposer),
      timestamp: b.timestamp,
      hash: bytesToHex(b.hash)
    }));
  }
}

/**
 * Standalone computeBlockHash for backward compatibility.
 *
 * Uses the FNV-based placeholder hash. For real SHA-256, use
 * `BlockChain.computeBlockHash()` which goes through the crypto provider.
 *
 * @deprecated Use `chain.computeBlockHash(block)` instead.
 */
export function computeBlockHash(block: BlockHeader): Uint8Array {
  const roundBytes = new Uint8Array(8);
  new DataView(roundBytes.buffer).setBigUint64(0, BigInt(block.round), false);

  const timestampBytes = new Uint8Array(8);
  new DataView(timestampBytes.buffer).setBigUint64(0, BigInt(block.timestamp), false);

  const totalLen =
    8 +
    block.prevHash.length +
    block.vdfState.length +
    block.vrfProof.length +
    block.proposer.length +
    8;

  const data = new Uint8Array(totalLen);
  let offset = 0;
  data.set(roundBytes, offset);
  offset += 8;
  data.set(block.prevHash, offset);
  offset += block.prevHash.length;
  data.set(block.vdfState, offset);
  offset += block.vdfState.length;
  data.set(block.vrfProof, offset);
  offset += block.vrfProof.length;
  data.set(block.proposer, offset);
  offset += block.proposer.length;
  data.set(timestampBytes, offset);

  // FNV-1a-like placeholder (same as old implementation for test compatibility)
  const hash = new Uint8Array(32);
  let h1 = 0x811c9dc5;
  let h2 = 0x01000193;
  let h3 = 0xdeadbeef;
  let h4 = 0x12345678;
  for (let i = 0; i < data.length; i++) {
    const byte = data[i];
    h1 ^= byte;
    h1 = Math.imul(h1, 0x01000193) >>> 0;
    h2 ^= byte;
    h2 = Math.imul(h2, 0x811c9dc5) >>> 0;
    h3 ^= byte;
    h3 = Math.imul(h3, 0x1b873593) >>> 0;
    h4 ^= byte;
    h4 = Math.imul(h4, 0xcc9e2d51) >>> 0;
    if ((i & 3) === 3) {
      h1 ^= h2;
      h2 ^= h3;
      h3 ^= h4;
      h4 ^= h1;
    }
  }
  const view = new DataView(hash.buffer);
  view.setUint32(0, h1, false);
  view.setUint32(4, h2, false);
  view.setUint32(8, h3, false);
  view.setUint32(12, h4, false);
  view.setUint32(16, h1 ^ h3, false);
  view.setUint32(20, h2 ^ h4, false);
  view.setUint32(24, h1 ^ h2 ^ h3, false);
  view.setUint32(28, h3 ^ h4 ^ h1, false);
  return hash;
}
