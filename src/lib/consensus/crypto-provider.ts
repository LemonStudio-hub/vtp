/**
 * Crypto Provider Interface
 *
 * Abstracts cryptographic operations for the consensus engine, allowing
 * the same engine code to work with either real WASM crypto (production)
 * or lightweight placeholders (tests).
 *
 * All methods are synchronous — the WASM functions perform in-memory
 * computation and return immediately.
 */

/**
 * Cryptographic operations needed by the consensus engine.
 */
export interface CryptoProvider {
  /** SHA-256 hash, returns 32 bytes. */
  hashBytes(data: Uint8Array): Uint8Array;

  /** Generate an ECVRF proof (RFC 9381). Returns 80-byte proof. */
  vrfProve(secretKey: Uint8Array, alpha: Uint8Array): Uint8Array;

  /** Derive 32-byte VRF output from an 80-byte proof. */
  vrfProofToHash(proof: Uint8Array): Uint8Array;

  /** Verify a VRF proof against a public key and message. */
  vrfVerify(publicKey: Uint8Array, alpha: Uint8Array, proof: Uint8Array): boolean;

  /** Ed25519 sign. Returns 64-byte signature. */
  ed25519Sign(secretKey: Uint8Array, data: Uint8Array): Uint8Array;

  /** Ed25519 verify. Returns true if signature is valid. */
  ed25519Verify(publicKey: Uint8Array, data: Uint8Array, signature: Uint8Array): boolean;
}

/**
 * Create a crypto provider backed by the real WASM module.
 *
 * @param wasm - The loaded vtp-core WASM module.
 */
export function createWasmCryptoProvider(wasm: {
  hash_bytes: (data: Uint8Array) => Uint8Array;
  prove: (secret_key: Uint8Array, alpha: Uint8Array) => Uint8Array;
  proof_to_hash: (proof: Uint8Array) => Uint8Array;
  verify: (public_key: Uint8Array, alpha: Uint8Array, proof: Uint8Array) => boolean;
  ed25519_sign: (secret_key: Uint8Array, data: Uint8Array) => Uint8Array;
  ed25519_verify: (public_key: Uint8Array, data: Uint8Array, signature: Uint8Array) => boolean;
}): CryptoProvider {
  return {
    hashBytes: (data) => wasm.hash_bytes(data),
    vrfProve: (secretKey, alpha) => wasm.prove(secretKey, alpha),
    vrfProofToHash: (proof) => wasm.proof_to_hash(proof),
    vrfVerify: (publicKey, alpha, proof) => wasm.verify(publicKey, alpha, proof),
    ed25519Sign: (secretKey, data) => wasm.ed25519_sign(secretKey, data),
    ed25519Verify: (publicKey, data, signature) => wasm.ed25519_verify(publicKey, data, signature)
  };
}

/**
 * Create a placeholder crypto provider for tests.
 *
 * Uses simple deterministic operations that are fast and don't require WASM.
 * NOT cryptographically secure — for testing only.
 */
export function createPlaceholderCryptoProvider(): CryptoProvider {
  return {
    hashBytes(data: Uint8Array): Uint8Array {
      // FNV-1a-like hash producing 32 bytes (matches old placeholder)
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
    },

    vrfProve(secretKey: Uint8Array, alpha: Uint8Array): Uint8Array {
      const proof = new Uint8Array(80);
      for (let i = 0; i < 80; i++) {
        proof[i] = secretKey[i % 32] ^ alpha[i % alpha.length] ^ (i & 0xff);
      }
      return proof;
    },

    vrfProofToHash(proof: Uint8Array): Uint8Array {
      const output = new Uint8Array(32);
      for (let i = 0; i < 32; i++) {
        let v = 0;
        for (let j = 0; j < proof.length; j += 32) {
          v ^= proof[(i + j) % proof.length];
        }
        output[i] = v;
      }
      return output;
    },

    vrfVerify(_publicKey: Uint8Array, _alpha: Uint8Array, _proof: Uint8Array): boolean {
      // Placeholder always accepts
      return true;
    },

    ed25519Sign(secretKey: Uint8Array, data: Uint8Array): Uint8Array {
      const combined = new Uint8Array(data.length + 32);
      combined.set(secretKey);
      combined.set(data, 32);
      const signature = new Uint8Array(64);
      for (let i = 0; i < 64; i++) {
        signature[i] = combined[i % combined.length] ^ secretKey[i % 32];
      }
      return signature;
    },

    ed25519Verify(_publicKey: Uint8Array, _data: Uint8Array, _signature: Uint8Array): boolean {
      // Placeholder always accepts
      return true;
    }
  };
}
