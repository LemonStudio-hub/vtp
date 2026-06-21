# Plan: Replace All Placeholder Implementations with Real Functionality

## Problem

The codebase has 6 placeholder/mock implementations that provide zero cryptographic security and break cross-node compatibility:

1. **MessageCodec signing** — XOR-based fake signatures (trivially forgeable)
2. **MessageCodec verification** — XOR-based fake verification (accepts any fake signature)
3. **Consensus VRF proof generation** — XOR-based fake proofs (not real VRF)
4. **Consensus VRF leader election** — XOR-based fake output (not real VRF)
5. **Consensus VRF proof verification** — Skipped entirely (accepts any proposal)
6. **BlockChain SHA-256** — FNV-1a placeholder (incompatible with Rust node)

Additionally, the protobuf generated files are out of sync with the .proto schema (missing 3 consensus message types).

## Strategy

The WASM module already exports `prove`, `proof_to_hash`, `verify` (VRF), and `hash_bytes` (SHA-256). The only missing exports are Ed25519 `sign` and `verify_signature` — these need two new Rust functions.

The consensus engine is synchronous TypeScript. The WASM functions are synchronous Rust. The crypto provider pattern keeps the engine testable while using real crypto at runtime.

---

## Phase 1: Add Ed25519 Sign/Verify to Rust WASM Module

### Step 1.1: Add `ed25519_sign` and `ed25519_verify` to `src/lib/vtp-core/src/utils.rs`

```rust
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};

#[wasm_bindgen]
pub fn ed25519_sign(secret_key: &[u8], data: &[u8]) -> Result<Vec<u8>, JsValue> {
    let key = SigningKey::from_slice(secret_key)
        .map_err(|e| JsValue::from_str(&format!("Invalid secret key: {}", e)))?;
    Ok(key.sign(data).to_bytes().to_vec())
}

#[wasm_bindgen]
pub fn ed25519_verify(public_key: &[u8], data: &[u8], signature: &[u8]) -> bool {
    let Ok(key) = VerifyingKey::from_slice(public_key) else { return false; };
    let Ok(sig) = Signature::from_slice(signature) else { return false; };
    key.verify(data, &sig).is_ok()
}
```

### Step 1.2: Update `src/lib/vtp-core/types/vtp_core.d.ts`

Add declarations:

```typescript
export function ed25519_sign(secret_key: Uint8Array, data: Uint8Array): Uint8Array;
export function ed25519_verify(
  public_key: Uint8Array,
  data: Uint8Array,
  signature: Uint8Array
): boolean;
```

Also fix the stale `Session` declarations (remove non-existent `generate_vdf_proof`/`verify_vdf_proof`, add missing standalone VRF/hash functions).

### Step 1.3: Rebuild WASM

```bash
cd src/lib/vtp-core && wasm-pack build --target web --release
cp pkg/vtp_core_bg.wasm ../../../static/wasm/
cp pkg/vtp_core.js ../../../static/wasm/
cp pkg/vtp_core.d.ts ../../../static/wasm/
```

---

## Phase 2: Create Crypto Provider Interface

### Step 2.1: Create `src/lib/consensus/crypto-provider.ts`

Defines a provider interface that the consensus engine uses for all crypto operations:

```typescript
export interface CryptoProvider {
  hashBytes(data: Uint8Array): Uint8Array;
  vrfProve(secretKey: Uint8Array, alpha: Uint8Array): Uint8Array;
  vrfProofToHash(proof: Uint8Array): Uint8Array;
  vrfVerify(publicKey: Uint8Array, alpha: Uint8Array, proof: Uint8Array): boolean;
}
```

Plus a `createWasmCryptoProvider(wasm: WasmModule)` factory that delegates to the real WASM functions, and a `createPlaceholderCryptoProvider()` for tests.

---

## Phase 3: Replace BlockChain SHA-256 Placeholder

### Step 3.1: Update `src/lib/consensus/block-chain.ts`

- Remove the 45-line `sha256Sync()` FNV placeholder function
- Accept `CryptoProvider` in `BlockChain` constructor (or as a static config)
- Replace `sha256Sync(data)` calls with `cryptoProvider.hashBytes(data)`
- `computeBlockHash(block)` becomes `computeBlockHash(block, provider)`

This makes block hashes match the Rust implementation, enabling cross-node verification.

---

## Phase 4: Replace Consensus VRF Placeholders

### Step 4.1: Update `src/lib/consensus/consensus-engine.ts`

- Accept `CryptoProvider` in constructor
- Replace `generateVrfProof()`: call `cryptoProvider.vrfProve(secretKey, roundSeed)`
- Replace `computeVrfOutput()`: call `cryptoProvider.vrfProofToHash(proof)`
- Replace `checkIsLeader()`: compare real VRF output against tau
- Add VRF verification in `receiveProposal()`: call `cryptoProvider.vrfVerify(proposerPubkey, roundSeed, vrfProof)`

All these are direct replacements — same function signatures, real crypto.

---

## Phase 5: Replace MessageCodec Signing Placeholders

### Step 5.1: Update `src/lib/network/message-codec.ts`

- Accept `WasmModule` reference in constructor (or load lazily)
- Replace `signSync()`: call `wasm.ed25519_sign(secretKey, data)`
- Replace `verify()`: call `wasm.ed25519_verify(publicKey, data, signature)`

The codec already stores `keypair` — the replacement is a direct function call swap.

---

## Phase 6: Regenerate Protobuf Files

### Step 6.1: Run `node proto/generate.js`

Regenerates `vtp_messages.js`, `vtp_messages.d.ts`, `vtp_messages.json` from the current `.proto` which includes all 9 message types (adds ConsensusProposal, ConsensusVote, NewRound, VotePhase).

---

## Phase 7: Wire WASM into Runtime

### Step 7.1: Update Web Worker (`src/lib/worker/index.ts`)

Export the loaded WASM module reference so the consensus engine and message codec can access it. Currently the worker loads WASM privately — expose a `getWasmModule()` accessor or pass the module to the consensus engine during initialization.

### Step 7.2: Update lifecycle modules (`src/lib/lifecycle/`)

Pass the WASM module reference through the lifecycle chain so the message codec and consensus engine receive the real crypto provider at initialization time.

---

## Phase 8: Update Tests

### Step 8.1: Update consensus tests to use placeholder crypto provider

Tests should use `createPlaceholderCryptoProvider()` for deterministic, fast tests without WASM dependency.

### Step 8.2: Update message-codec tests

The tampered signature test should now reliably detect tampering (real Ed25519 will reject any bit flip).

### Step 8.3: Add cross-hash compatibility test

Verify that `computeBlockHash()` in TypeScript produces the same hash as the Rust implementation for the same input.

---

## Files Modified

| File                                    | Change                                          |
| --------------------------------------- | ----------------------------------------------- |
| `src/lib/vtp-core/src/utils.rs`         | Add `ed25519_sign`, `ed25519_verify`            |
| `src/lib/vtp-core/types/vtp_core.d.ts`  | Update type declarations                        |
| `static/wasm/*`                         | Rebuilt WASM binary + JS + types                |
| `src/lib/consensus/crypto-provider.ts`  | **NEW** — crypto provider interface             |
| `src/lib/consensus/block-chain.ts`      | Replace FNV hash with real SHA-256 via provider |
| `src/lib/consensus/consensus-engine.ts` | Replace VRF placeholders with real calls        |
| `src/lib/network/message-codec.ts`      | Replace Ed25519 placeholders with real calls    |
| `src/lib/worker/index.ts`               | Expose WASM module reference                    |
| `src/lib/lifecycle/`                    | Wire WASM module through initialization         |
| `proto/vtp_messages.js`                 | Regenerated                                     |
| `proto/vtp_messages.d.ts`               | Regenerated                                     |
| `proto/vtp_messages.json`               | Regenerated                                     |
| `tests/consensus-engine.test.ts`        | Use placeholder crypto provider                 |
| `tests/message-codec.test.ts`           | Expect real signature verification              |
