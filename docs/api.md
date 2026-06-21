# VTP Node API Reference

<div align="center">

**Complete API documentation for the VTP Node project**

[Overview](#overview) • [Rust Core API](#rust-core-api) • [Consensus API](#consensus-api) • [Network API](#network-api) • [Web Worker API](#web-worker-api) • [Svelte Stores](#svelte-stores) • [Utility Functions](#utility-functions)

</div>

---

## Table of Contents

- [Overview](#overview)
- [Rust Core API](#rust-core-api)
  - [VDF Module](#vdf-module)
  - [VRF Module](#vrf-module)
  - [Session Module](#session-module)
  - [Error Module](#error-module)
  - [Utils Module](#utils-module)
- [Consensus API](#consensus-api)
  - [Rust Consensus Module](#rust-consensus-module)
  - [TypeScript Consensus Engine](#typescript-consensus-engine)
  - [Vote Pool](#vote-pool)
  - [Block Chain](#block-chain)
- [Network API](#network-api)
  - [PeerManager](#peermanager)
  - [PeerConnection](#peerconnection)
  - [SignalingClient](#signalingclient)
  - [Message Codec](#message-codec)
  - [Consensus Message Helpers](#consensus-message-helpers)
- [Web Worker API](#web-worker-api)
  - [Messages (Main Thread → Worker)](#messages-main-thread--worker)
  - [Messages (Worker → Main Thread)](#messages-worker--main-thread)
  - [Message Types](#message-types)
- [Svelte Stores](#svelte-stores)
  - [Worker State Store](#worker-state-store)
  - [Events Store](#events-store)
  - [Progress Store](#progress-store)
- [Utility Functions](#utility-functions)
  - [Formatting Functions](#formatting-functions)
  - [Generation Functions](#generation-functions)
  - [Async Utilities](#async-utilities)
- [TypeScript Interfaces](#typescript-interfaces)
- [Error Codes](#error-codes)
- [Examples](#examples)

---

## Overview

The VTP Node project exposes APIs at multiple levels:

1. **Rust Core API** (`vtp-core`): Low-level cryptographic functions compiled to WebAssembly
2. **Web Worker API**: Message-based interface for background computation
3. **Svelte Stores**: Reactive state management for the UI
4. **Utility Functions**: Helper functions for common operations
5. **Consensus API**: VRF-driven leader election + BFT validation
6. **Network API**: WebRTC P2P networking with signaling

### API Design Principles

- **Type Safety**: All APIs use TypeScript/Rust type system for compile-time checks
- **Immutability**: State updates are immutable where possible
- **Error Handling**: All fallible operations return Result types
- **Performance**: APIs are optimized for browser environments

---

## Rust Core API

The `vtp-core` crate provides the core cryptographic functionality. It is compiled to WebAssembly and can be called from JavaScript.

### VDF Module

The VDF (Verifiable Delay Function) module implements Wesolowski's construction over imaginary quadratic class groups.

#### `vdf_step`

Execute a single VDF step (class group squaring).

```rust
pub fn vdf_step(state: &[u8; 32]) -> [u8; 32]
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `state` | `[u8; 32]` | Current VDF state seed (32 bytes) |

**Returns:**

- `[u8; 32]`: Hash of the squared class group element

**Example:**

```rust
let state = [0u8; 32];
let next_state = vdf_step(&state);
assert_eq!(next_state.len(), 32);
```

---

#### `VdfIterator`

Iterator for batch VDF processing.

```rust
pub struct VdfIterator {
    state: VdfState,
    step: u64,
    total: u64,
    discriminant: BigInt,
    generator: ClassGroupElement,
}
```

##### `VdfIterator::new`

Create a new VDF iterator.

```rust
pub fn new(seed: &[u8], total: u64) -> Self
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `seed` | `&[u8]` | Initial seed (at least 32 bytes) |
| `total` | `u64` | Total steps target |

**Returns:** `VdfIterator` instance

**Panics:** If seed is less than 32 bytes

---

##### `VdfIterator::step`

Get current step count.

```rust
pub fn step(&self) -> u64
```

**Returns:** Current step count

---

##### `VdfIterator::total`

Get total steps target.

```rust
pub fn total(&self) -> u64
```

**Returns:** Total steps target

---

##### `VdfIterator::is_finished`

Check if computation is finished.

```rust
pub fn is_finished(&self) -> bool
```

**Returns:**

- `true`: All steps completed
- `false`: Steps remaining

---

##### `VdfIterator::get_state`

Get current VDF state.

```rust
pub fn get_state(&self) -> Vec<u8>
```

**Returns:** Variable-length state vector (serialised class group element)

---

##### `VdfIterator::next`

Execute single step.

```rust
pub fn next(&mut self) -> bool
```

**Returns:**

- `true`: Step executed successfully
- `false`: Already finished

---

##### `VdfIterator::run_batch`

Execute batch of steps.

```rust
pub fn run_batch(&mut self, max_steps: u64) -> u64
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `max_steps` | `u64` | Maximum steps to execute |

**Returns:** Number of steps actually executed

**Example:**

```rust
let seed = [0u8; 32];
let mut iter = VdfIterator::new(&seed, 1000);
let steps = iter.run_batch(100);
assert_eq!(steps, 100);
```

---

#### `generate_proof`

Generate a Wesolowski proof for a completed VDF computation.

```rust
pub fn generate_proof(seed: &[u8], state_bytes: &[u8], total: u64) -> Vec<u8>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `seed` | `&[u8]` | Original VDF seed (at least 32 bytes) |
| `state_bytes` | `&[u8]` | Serialised final state (output y) |
| `total` | `u64` | Time parameter T |

**Returns:** The Wesolowski proof as bytes (serialised class group element)

---

#### `verify_proof`

Verify a Wesolowski proof. Checks that pi^l \* g^q == y where l = Hash(g, y, T).

```rust
pub fn verify_proof(seed: &[u8], state_bytes: &[u8], total: u64, proof_bytes: &[u8]) -> bool
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `seed` | `&[u8]` | Original VDF seed |
| `state_bytes` | `&[u8]` | Claimed output y |
| `total` | `u64` | Time parameter T |
| `proof_bytes` | `&[u8]` | The proof pi |

**Returns:**

- `true`: The proof is valid
- `false`: The proof is invalid

---

### VRF Module

The VRF (Verifiable Random Function) module implements ECVRF-ED25519.

#### `generate_keypair`

Generate a new VRF keypair.

```rust
pub fn generate_keypair() -> VrfKeypair
```

**Returns:** `VrfKeypair` containing public and secret keys

**Example:**

```rust
let keypair = generate_keypair();
println!("Public key: {:?}", keypair.public_key());
```

---

#### `VrfKeypair`

VRF keypair structure.

```rust
pub struct VrfKeypair {
    public_key: Vec<u8>,
    secret_key: Vec<u8>,
}
```

##### `VrfKeypair::public_key`

Get public key.

```rust
#[wasm_bindgen(getter)]
pub fn public_key(&self) -> Vec<u8>
```

**Returns:** 32-byte public key vector

---

##### `VrfKeypair::secret_key`

Get secret key.

```rust
#[wasm_bindgen(getter)]
pub fn secret_key(&self) -> Vec<u8>
```

**Returns:** 32-byte secret key vector

**Security:** Keep secret key secure, never share

---

#### `prove`

Generate VRF proof.

```rust
pub fn prove(secret_key: &[u8], message: &[u8]) -> Vec<u8>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `secret_key` | `&[u8]` | 32-byte secret key |
| `message` | `&[u8]` | Message to sign |

**Returns:** 64-byte signature vector

**Panics:** If secret_key is not 32 bytes

**Example:**

```rust
let keypair = generate_keypair();
let message = b"challenge data";
let proof = prove(&keypair.secret_key(), message);
```

---

#### `verify`

Verify VRF proof.

```rust
pub fn verify(public_key: &[u8], message: &[u8], proof: &[u8]) -> bool
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `public_key` | `&[u8]` | 32-byte public key |
| `message` | `&[u8]` | Original message |
| `proof` | `&[u8]` | 64-byte signature |

**Returns:**

- `true`: Proof is valid
- `false`: Proof is invalid

---

##### `Session::generate_vdf_proof`

Generate a Wesolowski proof for the completed VDF computation.

```rust
pub fn generate_vdf_proof(&self) -> Vec<u8>
```

**Returns:** The Wesolowski proof as bytes (serialised class group element)

**Panics:** If the VDF computation has not finished yet

---

##### `Session::verify_vdf_proof`

Verify a Wesolowski proof against the current session state.

```rust
pub fn verify_vdf_proof(&self, proof_bytes: &[u8]) -> bool
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `proof_bytes` | `&[u8]` | The Wesolowski proof bytes |

**Returns:**

- `true`: The proof is valid
- `false`: The proof is invalid or verification failed

**Example:**

```rust
let keypair = generate_keypair();
let message = b"challenge data";
let proof = prove(&keypair.secret_key(), message);
assert!(verify(&keypair.public_key(), message, &proof));
```

---

### Session Module

The Session module manages complete VDF challenge lifecycle.

#### `BatchResult`

Batch computation result enum.

```rust
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchResult {
    Progress(u64),
    Winner(u64, Vec<u8>),
    Finished,
    Error(VtpError),
}
```

**Variants:**
| Variant | Description |
|---------|-------------|
| `Progress(u64)` | Computation in progress, returns current step |
| `Winner(u64, Vec<u8>)` | Winner found, returns step and VRF proof |
| `Finished` | VDF computation completed |
| `Error(VtpError)` | Error occurred |

---

#### `SessionState`

Session state structure.

```rust
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub step: u64,
    pub total: u64,
    pub is_active: bool,
    pub is_paused: bool,
    pub error_count: u32,
}
```

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `step` | `u64` | Current completed steps |
| `total` | `u64` | Total steps target |
| `is_active` | `bool` | Session is active (not finished) |
| `is_paused` | `bool` | Session is paused |
| `error_count` | `u32` | Number of errors occurred |

---

#### `Session`

VDF challenge session manager.

```rust
pub struct Session {
    vdf: VdfIterator,
    seed: Vec<u8>,
    keypair: VrfKeypair,
    k: u64,
    tau: Vec<u8>,
    checkpoint_interval: u64,
    error_handler: ErrorHandler,
    is_paused: bool,
}
```

##### `Session::new`

Create new session.

```rust
#[wasm_bindgen(constructor)]
pub fn new(
    seed: &[u8],
    total: u64,
    k: u64,
    tau: &[u8],
    checkpoint_interval: u64,
) -> Self
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `seed` | `&[u8]` | VDF seed (at least 32 bytes) |
| `total` | `u64` | Total VDF steps |
| `k` | `u64` | VRF sampling interval |
| `tau` | `&[u8]` | Threshold (32 bytes) |
| `checkpoint_interval` | `u64` | Checkpoint interval |

**Returns:** New Session instance

**Example:**

```rust
let seed = [0u8; 32];
let tau = [0u8; 32];
let session = Session::new(&seed, 1000000, 1000, &tau, 100000);
```

---

##### `Session::state`

Get current session state.

```rust
#[wasm_bindgen(getter)]
pub fn state(&self) -> SessionState
```

**Returns:** SessionState structure

---

##### `Session::public_key`

Get VRF public key.

```rust
#[wasm_bindgen(getter)]
pub fn public_key(&self) -> Vec<u8>
```

**Returns:** 32-byte public key vector

---

##### `Session::pause`

Pause session.

```rust
pub fn pause(&mut self)
```

---

##### `Session::resume`

Resume session.

```rust
pub fn resume(&mut self)
```

---

##### `Session::is_paused`

Check if session is paused.

```rust
pub fn is_paused(&self) -> bool
```

**Returns:**

- `true`: Session is paused
- `false`: Session is running

---

##### `Session::run_batch`

Execute batch VDF computation.

```rust
pub fn run_batch(&mut self, max_steps: u64) -> BatchResult
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `max_steps` | `u64` | Maximum steps to execute |

**Returns:** BatchResult enum

**Example:**

```rust
let mut session = Session::new(&seed, 1000000, 1000, &tau, 100000);

loop {
    match session.run_batch(1000) {
        BatchResult::Progress(step) => println!("Progress: {}", step),
        BatchResult::Winner(step, proof) => println!("Winner at step {}", step),
        BatchResult::Finished => break,
        BatchResult::Error(err) => eprintln!("Error: {}", err),
    }
}
```

---

##### `Session::get_checkpoint_data`

Get checkpoint data for persistence.

```rust
pub fn get_checkpoint_data(&self) -> Vec<u8>
```

**Returns:** Serialized checkpoint data

**Format:** `[8 bytes step (big-endian)] [variable-length VDF state (class group element)]`

---

##### `Session::verify_winner`

Verify winner proof.

```rust
pub fn verify_winner(&self, step: u64, proof: &[u8]) -> bool
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `step` | `u64` | Winner step |
| `proof` | `&[u8]` | VRF proof |

**Returns:**

- `true`: Proof is valid
- `false`: Proof is invalid

---

### Error Module

Error types and error handler.

#### `VtpError`

VTP error enum.

```rust
#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq)]
pub enum VtpError {
    InvalidInput,
    InvalidState,
    ComputationFailed,
    CheckpointFailed,
    SessionFinished,
    SessionNotStarted,
}
```

**Variants:**
| Variant | Description |
|---------|-------------|
| `InvalidInput` | Invalid input parameters |
| `InvalidState` | Invalid internal state |
| `ComputationFailed` | Computation failed |
| `CheckpointFailed` | Checkpoint save/load failed |
| `SessionFinished` | Session already finished |
| `SessionNotStarted` | Session not started |

---

#### `ErrorHandler`

Error handler with retry logic.

```rust
#[derive(Debug, Clone)]
pub struct ErrorHandler {
    pub last_error: Option<VtpError>,
    pub error_count: u32,
    pub max_retries: u32,
}
```

##### `ErrorHandler::new`

Create new error handler.

```rust
pub fn new(max_retries: u32) -> Self
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `max_retries` | `u32` | Maximum retry attempts |

---

##### `ErrorHandler::handle_error`

Handle error and check if can retry.

```rust
pub fn handle_error(&mut self, error: VtpError) -> bool
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `error` | `VtpError` | Error to handle |

**Returns:**

- `true`: Can continue (not exceeded max retries)
- `false`: Should stop (exceeded max retries)

---

##### `ErrorHandler::reset`

Reset error state.

```rust
pub fn reset(&mut self)
```

---

##### `ErrorHandler::can_retry`

Check if can retry.

```rust
pub fn can_retry(&self) -> bool
```

**Returns:**

- `true`: Can retry
- `false`: Max retries reached

---

### Utils Module

Utility functions.

#### `hash_bytes`

Calculate SHA256 hash.

```rust
#[wasm_bindgen]
pub fn hash_bytes(data: &[u8]) -> Vec<u8>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `data` | `&[u8]` | Data to hash |

**Returns:** 32-byte hash vector

---

#### `bytes_to_hex`

Convert bytes to hex string.

```rust
#[wasm_bindgen]
pub fn bytes_to_hex(bytes: &[u8]) -> String
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `bytes` | `&[u8]` | Bytes to convert |

**Returns:** Hex string (lowercase)

**Example:**

```rust
let bytes = vec![0x00, 0x0f, 0xff];
let hex = bytes_to_hex(&bytes);
assert_eq!(hex, "000fff");
```

---

#### `hex_to_bytes`

Convert hex string to bytes.

```rust
#[wasm_bindgen]
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, JsValue>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `hex` | `&str` | Hex string |

**Returns:**

- `Ok(Vec<u8>)`: Converted bytes
- `Err(JsValue)`: Conversion error

**Errors:**

- Odd length string
- Non-hex characters

---

#### `generate_random_bytes`

Generate random bytes.

```rust
#[wasm_bindgen]
pub fn generate_random_bytes(length: u32) -> Vec<u8>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `length` | `u32` | Number of bytes to generate |

**Returns:** Random bytes vector

**Security:** Uses OS random number generator

---

## Consensus API

The Consensus API implements VRF-driven leader election with BFT-style voting (propose, prevote, precommit, commit). It is available as both a Rust/WASM module and a TypeScript wrapper.

### Rust Consensus Module

The `consensus` module in `vtp-core` provides the low-level consensus primitives compiled to WebAssembly.

#### `ConsensusPhase`

Consensus phase enum.

```rust
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsensusPhase {
    Propose,
    Prevote,
    Precommit,
    Commit,
}
```

**Variants:**
| Variant | Description |
|---------|-------------|
| `Propose` | Leader proposes a block |
| `Prevote` | Validators prevote on proposal |
| `Precommit` | Validators precommit after prevote quorum |
| `Commit` | Block committed after precommit quorum |

---

#### `VotePhase`

Vote phase enum (subset of ConsensusPhase for vote messages).

```rust
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VotePhase {
    Prevote,
    Precommit,
}
```

---

#### `BlockHeader`

Block header structure.

```rust
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub round: u64,
    pub prev_hash: Vec<u8>,
    pub vdf_state: Vec<u8>,
    pub vrf_proof: Vec<u8>,
    pub proposer: Vec<u8>,
    pub timestamp: u64,
}
```

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `round` | `u64` | Consensus round number |
| `prev_hash` | `Vec<u8>` | Hash of the previous block |
| `vdf_state` | `Vec<u8>` | VDF state at time of proposal |
| `vrf_proof` | `Vec<u8>` | VRF proof of leader election |
| `proposer` | `Vec<u8>` | Proposer public key |
| `timestamp` | `u64` | Block timestamp (Unix ms) |

##### `BlockHeader::new`

Create a new block header.

```rust
#[wasm_bindgen(constructor)]
pub fn new(
    round: u64,
    prev_hash: Vec<u8>,
    vdf_state: Vec<u8>,
    vrf_proof: Vec<u8>,
    proposer: Vec<u8>,
    timestamp: u64,
) -> Self
```

##### `BlockHeader::to_bytes`

Serialize the header to bytes.

```rust
pub fn to_bytes(&self) -> Vec<u8>
```

##### `BlockHeader::from_bytes`

Deserialize a header from bytes.

```rust
pub fn from_bytes(bytes: &[u8]) -> Result<BlockHeader, JsValue>
```

##### `BlockHeader::hash`

Compute the SHA-256 hash of this block header.

```rust
pub fn hash(&self) -> Vec<u8>
```

**Returns:** 32-byte hash vector

##### Getters

```rust
pub fn round(&self) -> u64
pub fn prev_hash(&self) -> Vec<u8>
pub fn vdf_state(&self) -> Vec<u8>
pub fn vrf_proof(&self) -> Vec<u8>
pub fn proposer(&self) -> Vec<u8>
pub fn timestamp(&self) -> u64
```

---

#### `ConsensusEngine`

BFT consensus engine with VRF-driven leader election.

```rust
pub struct ConsensusEngine {
    secret_key: Vec<u8>,
    public_key: Vec<u8>,
    validators: Vec<Vec<u8>>,
    tau: Vec<u8>,
    round: u64,
    phase: ConsensusPhase,
    vote_pool: VotePool,
    chain: BlockChain,
}
```

##### `ConsensusEngine::new_native`

Create a new consensus engine (native Rust usage).

```rust
pub fn new_native(
    secret_key: Vec<u8>,
    public_key: Vec<u8>,
    validators: Vec<Vec<u8>>,
    tau: Vec<u8>,
) -> Self
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `secret_key` | `Vec<u8>` | 32-byte secret key |
| `public_key` | `Vec<u8>` | 32-byte public key |
| `validators` | `Vec<Vec<u8>>` | List of validator public keys |
| `tau` | `Vec<u8>` | 32-byte VRF threshold |

---

##### `ConsensusEngine::new` (WASM)

Create a new consensus engine (WASM constructor).

```rust
#[wasm_bindgen(constructor)]
pub fn new(
    secret_key: Vec<u8>,
    public_key: Vec<u8>,
    validator_keys: Vec<u8>,
    tau: Vec<u8>,
) -> Self
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `secret_key` | `Vec<u8>` | 32-byte secret key |
| `public_key` | `Vec<u8>` | 32-byte public key |
| `validator_keys` | `Vec<u8>` | Concatenated 32-byte validator public keys |
| `tau` | `Vec<u8>` | 32-byte VRF threshold |

---

##### `ConsensusEngine::start_round`

Start a new consensus round and generate VRF proof.

```rust
pub fn start_round(&mut self, round: u64) -> Vec<u8>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `round` | `u64` | Round number to start |

**Returns:** VRF proof bytes

---

##### `ConsensusEngine::is_leader`

Check if this node is the leader for the current round.

```rust
pub fn is_leader(&self) -> bool
```

**Returns:** `true` if this node won VRF leader election

---

##### `ConsensusEngine::propose_native`

Create a block proposal (native Rust).

```rust
pub fn propose_native(
    &mut self,
    vdf_state: Vec<u8>,
    timestamp: u64,
) -> Result<Vec<u8>, String>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `vdf_state` | `Vec<u8>` | Current VDF state |
| `timestamp` | `u64` | Block timestamp |

**Returns:** Serialized block proposal bytes

**Errors:** If this node is not the leader or the phase is not Propose

---

##### `ConsensusEngine::prevote_native`

Cast a prevote.

```rust
pub fn prevote_native(&mut self, accept: bool) -> Result<Vec<u8>, String>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `accept` | `bool` | Whether to accept the proposal |

**Returns:** Serialized prevote message bytes

---

##### `ConsensusEngine::precommit_native`

Cast a precommit vote.

```rust
pub fn precommit_native(&mut self, block_hash: Vec<u8>) -> Result<Vec<u8>, String>
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `block_hash` | `Vec<u8>` | Hash of the block to precommit |

**Returns:** Serialized precommit message bytes

---

##### `ConsensusEngine::receive_vote`

Process an incoming vote.

```rust
pub fn receive_vote(&mut self, vote_bytes: Vec<u8>) -> i32
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `vote_bytes` | `Vec<u8>` | Serialized vote message |

**Returns:**
| Value | Meaning |
|-------|---------|
| `0` | Vote accepted |
| `1` | Prevote quorum reached |
| `2` | Precommit quorum reached |
| `-1` | Vote rejected (invalid or duplicate) |

---

##### `ConsensusEngine::finalize_native`

Finalize the current round and append the block to the chain.

```rust
pub fn finalize_native(&mut self) -> Result<Vec<u8>, String>
```

**Returns:** Serialized committed block bytes

**Errors:** If precommit quorum has not been reached

---

##### `ConsensusEngine::state`

Get the current consensus state.

```rust
pub fn state(&self) -> ConsensusState
```

---

##### Getters

```rust
pub fn round(&self) -> u64
pub fn chain_height(&self) -> u64
pub fn threshold(&self) -> u32
pub fn prevote_count(&self) -> u32
pub fn precommit_count(&self) -> u32
pub fn latest_block_hash(&self) -> Vec<u8>
pub fn verify_chain(&self) -> bool
```

---

#### `ConsensusState`

Consensus state snapshot.

```rust
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusState {
    pub round: u64,
    pub phase: ConsensusPhase,
    pub is_leader: bool,
    pub prevote_count: u32,
    pub precommit_count: u32,
    pub chain_height: u64,
}
```

**Fields:**
| Field | Type | Description |
|-------|------|-------------|
| `round` | `u64` | Current consensus round |
| `phase` | `ConsensusPhase` | Current consensus phase |
| `is_leader` | `bool` | Whether this node is the leader |
| `prevote_count` | `u32` | Number of prevotes received |
| `precommit_count` | `u32` | Number of precommits received |
| `chain_height` | `u64` | Number of committed blocks |

---

#### `verify_leader_election`

Verify that a VRF proof is valid for leader election.

```rust
#[wasm_bindgen]
pub fn verify_leader_election(block_bytes: &[u8], tau: &[u8]) -> bool
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `block_bytes` | `&[u8]` | Serialized block header |
| `tau` | `&[u8]` | 32-byte VRF threshold |

**Returns:** `true` if the block's VRF proof is below the threshold

---

#### `compute_threshold`

Compute the BFT quorum threshold for a given validator count.

```rust
#[wasm_bindgen]
pub fn compute_threshold(num_validators: u32) -> u32
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `num_validators` | `u32` | Number of validators |

**Returns:** Quorum threshold (2f + 1)

---

### TypeScript Consensus Engine

The TypeScript `ConsensusEngine` wraps the WASM consensus module with async methods and an event-driven interface.

**Source:** `src/lib/consensus/consensus-engine.ts`

#### Constructor

```typescript
import { ConsensusEngine } from '$lib/consensus';

const engine = new ConsensusEngine(
  secretKey,    // Uint8Array (32 bytes)
  publicKey,    // Uint8Array (32 bytes)
  validators,   // Uint8Array[] — list of validator public keys
  config?,      // ConsensusConfig (optional)
);
```

**ConsensusConfig:**

```typescript
interface ConsensusConfig {
  tau?: Uint8Array; // 32-byte VRF threshold
  checkpointInterval?: number; // VDF checkpoint interval
  roundTimeoutMs?: number; // Round timeout in milliseconds (default: 30000)
}
```

---

#### Methods

##### `startRound`

Start a new consensus round.

```typescript
startRound(round: number): Uint8Array
```

**Returns:** VRF proof bytes

---

##### `propose`

Create a block proposal (leader only).

```typescript
propose(vdfState: Uint8Array, timestamp: number): Uint8Array
```

**Returns:** Serialized proposal bytes

**Throws:** If not the leader or not in Propose phase

---

##### `receiveProposal`

Process an incoming block proposal.

```typescript
receiveProposal(proposalBytes: Uint8Array): boolean
```

**Returns:** `true` if the proposal is valid

---

##### `prevote`

Cast a prevote.

```typescript
prevote(accept: boolean): Uint8Array
```

**Returns:** Serialized prevote bytes

---

##### `precommit`

Cast a precommit vote.

```typescript
precommit(blockHash: Uint8Array): Uint8Array
```

**Returns:** Serialized precommit bytes

---

##### `receiveVote`

Process an incoming vote.

```typescript
receiveVote(voteBytes: Uint8Array): number
```

**Returns:** `0` = accepted, `1` = prevote quorum, `2` = precommit quorum, `-1` = rejected

---

##### `finalize`

Finalize the current round and commit the block.

```typescript
finalize(): Uint8Array
```

**Returns:** Serialized committed block bytes

---

##### `getState`

Get the current consensus state.

```typescript
getState(): ConsensusState
```

---

##### `dispose`

Clean up resources and remove all event listeners.

```typescript
dispose(): void
```

---

#### Events

The TypeScript ConsensusEngine extends `EventEmitter` and emits the following events:

| Event             | Callback Signature                           | Description            |
| ----------------- | -------------------------------------------- | ---------------------- |
| `onPhaseChange`   | `(phase: ConsensusPhase) => void`            | Phase transition       |
| `onLeaderElected` | `(leader: Uint8Array) => void`               | Leader elected via VRF |
| `onProposal`      | `(proposal: Uint8Array) => void`             | Proposal received      |
| `onCommit`        | `(round: number, block: Uint8Array) => void` | Block committed        |
| `onRoundTimeout`  | `(round: number) => void`                    | Round timed out        |
| `onStateChange`   | `(state: ConsensusState) => void`            | State updated          |

**Example:**

```typescript
engine.on('onCommit', (round, block) => {
  console.log(`Block committed at round ${round}`);
});

engine.on('onPhaseChange', (phase) => {
  console.log('Phase:', phase);
});
```

---

### Vote Pool

The `VotePool` tracks prevotes and precommits for a consensus round.

**Source:** `src/lib/consensus/vote-pool.ts`

#### Constructor

```typescript
import { VotePool } from '$lib/consensus';

const pool = new VotePool(validatorCount: number);
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `validatorCount` | `number` | Total number of validators |

---

#### Static Methods

##### `VotePool.computeThreshold`

Compute the BFT quorum threshold.

```typescript
static computeThreshold(n: number): number
```

**Returns:** 2f + 1 where f = floor((n - 1) / 3)

---

#### Methods

##### `addPrevote`

Record a prevote.

```typescript
addPrevote(voterPubkey: Uint8Array, blockHash: Uint8Array): boolean
```

**Returns:** `true` if the vote was added (not a duplicate)

---

##### `addPrecommit`

Record a precommit.

```typescript
addPrecommit(voterPubkey: Uint8Array, blockHash: Uint8Array): boolean
```

**Returns:** `true` if the vote was added (not a duplicate)

---

##### `getPrevoteQuorum`

Check if prevote quorum is reached.

```typescript
getPrevoteQuorum(): boolean
```

---

##### `getPrecommitQuorum`

Check if precommit quorum is reached.

```typescript
getPrecommitQuorum(): boolean
```

---

##### `hasPrevoted`

Check if a validator has already prevoted.

```typescript
hasPrevoted(voterPubkey: Uint8Array): boolean
```

---

##### `hasPrecommitted`

Check if a validator has already precommitted.

```typescript
hasPrecommitted(voterPubkey: Uint8Array): boolean
```

---

##### `clear`

Reset the vote pool.

```typescript
clear(): void
```

---

#### Properties

| Property          | Type     | Description                   |
| ----------------- | -------- | ----------------------------- |
| `prevoteCount`    | `number` | Number of prevotes received   |
| `precommitCount`  | `number` | Number of precommits received |
| `quorumThreshold` | `number` | Required votes for quorum     |
| `size`            | `number` | Total validator count         |

---

### Block Chain

The `BlockChain` stores committed blocks and provides verification.

**Source:** `src/lib/consensus/block-chain.ts`

#### Static Properties

| Property       | Type         | Description                             |
| -------------- | ------------ | --------------------------------------- |
| `GENESIS_HASH` | `Uint8Array` | 32-byte zero hash for the genesis block |

---

#### Properties

| Property     | Type          | Description                |
| ------------ | ------------- | -------------------------- |
| `height`     | `number`      | Number of committed blocks |
| `latestHash` | `Uint8Array`  | Hash of the latest block   |
| `latest`     | `BlockHeader` | Latest block header        |

---

#### Methods

##### `append`

Append a verified block to the chain.

```typescript
append(block: BlockHeader): void
```

**Throws:** If the block's `prev_hash` does not match `latestHash`

---

##### `getByRound`

Get a block by round number.

```typescript
getByRound(round: number): BlockHeader | undefined
```

---

##### `getByHash`

Get a block by its hash.

```typescript
getByHash(hash: Uint8Array): BlockHeader | undefined
```

---

##### `getRecent`

Get the N most recent blocks.

```typescript
getRecent(count: number): BlockHeader[]
```

---

##### `verify`

Verify the entire chain (hash linkage from genesis to tip).

```typescript
verify(): boolean
```

**Returns:** `true` if all blocks are valid and linked correctly

---

##### `toJSON`

Serialize the chain to a JSON-compatible object.

```typescript
toJSON(): object
```

---

## Network API

The Network API provides WebRTC peer-to-peer connectivity with a signaling server for peer discovery and SDP exchange.

### PeerManager

The main P2P orchestrator that manages peer connections, message routing, and network state.

**Source:** `src/lib/network/peer-manager.ts`

#### Constructor

```typescript
import { PeerManager } from '$lib/network';

const manager = new PeerManager({
  signalingUrl: 'wss://signal.example.com',
  roomId: 'my-room',
  keypair: { publicKey, secretKey },
  iceServers?: [{ urls: 'stun:stun.l.google.com:19302' }],
});
```

---

#### Methods

##### `connect`

Connect to the signaling server and join a room.

```typescript
connect(): Promise<void>
```

---

##### `disconnect`

Leave the room and close all peer connections.

```typescript
disconnect(): void
```

---

##### `broadcast`

Send a message to all connected peers.

```typescript
broadcast(message: Uint8Array): void
```

---

##### `sendTo`

Send a message to a specific peer.

```typescript
sendTo(peerId: string, message: Uint8Array): void
```

---

##### `getState`

Get the current network state.

```typescript
getState(): NetworkState
```

**NetworkState:**

```typescript
interface NetworkState {
  status: 'disconnected' | 'connecting' | 'connected';
  peerCount: number;
  peers: Map<string, PeerInfo>;
}
```

---

##### `getPeer`

Get info for a specific peer.

```typescript
getPeer(peerId: string): PeerInfo | undefined
```

---

##### `getPeerCount`

Get the number of connected peers.

```typescript
getPeerCount(): number
```

---

#### Events

| Event                | Callback Signature                           | Description                 |
| -------------------- | -------------------------------------------- | --------------------------- |
| `onPeerConnected`    | `(peerId: string) => void`                   | Peer connection established |
| `onPeerDisconnected` | `(peerId: string) => void`                   | Peer disconnected           |
| `onMessage`          | `(peerId: string, data: Uint8Array) => void` | Message received from peer  |
| `onStateChange`      | `(state: NetworkState) => void`              | Network state changed       |

---

### PeerConnection

A WebRTC wrapper for a single peer-to-peer connection.

**Source:** `src/lib/network/peer-connection.ts`

#### Features

- IPv6-prioritized ICE candidate handling
- DataChannel management with ordered/unordered modes
- Ping/pong RTT measurement
- Automatic reconnection support

---

#### Methods

##### `createOffer`

Create an SDP offer for this connection.

```typescript
createOffer(): Promise<RTCSessionDescriptionInit>
```

---

##### `handleOffer`

Process an incoming SDP offer and generate an answer.

```typescript
handleOffer(offer: RTCSessionDescriptionInit): Promise<RTCSessionDescriptionInit>
```

---

##### `handleAnswer`

Process an incoming SDP answer.

```typescript
handleAnswer(answer: RTCSessionDescriptionInit): Promise<void>
```

---

##### `addIceCandidate`

Add an ICE candidate to the connection.

```typescript
addIceCandidate(candidate: RTCIceCandidateInit): Promise<void>
```

---

##### `send`

Send data over the DataChannel.

```typescript
send(data: Uint8Array): void
```

---

##### `close`

Close the connection and all associated channels.

```typescript
close(): void
```

---

##### `getInfo`

Get connection info including RTT and state.

```typescript
getInfo(): PeerInfo
```

**PeerInfo:**

```typescript
interface PeerInfo {
  peerId: string;
  rtt: number; // Round-trip time in ms
  state: RTCPeerConnectionState;
  connectedAt: number; // Unix timestamp
}
```

---

### SignalingClient

WebSocket client for the Cloudflare Durable Object signaling server.

**Source:** `src/lib/network/signaling-client.ts`

#### Features

- Cloudflare Durable Object signaling protocol
- Auto-reconnect with exponential backoff
- Room-based peer discovery

---

#### Methods

##### `connect`

Connect to the signaling server and join a room.

```typescript
connect(roomId: string): Promise<void>
```

---

##### `disconnect`

Leave the room and close the WebSocket.

```typescript
disconnect(): void
```

---

##### `sendOffer`

Send an SDP offer to a specific peer via signaling.

```typescript
sendOffer(peerId: string, offer: RTCSessionDescriptionInit): void
```

---

##### `sendAnswer`

Send an SDP answer to a specific peer via signaling.

```typescript
sendAnswer(peerId: string, answer: RTCSessionDescriptionInit): void
```

---

##### `sendCandidate`

Send an ICE candidate to a specific peer via signaling.

```typescript
sendCandidate(peerId: string, candidate: RTCIceCandidateInit): void
```

---

##### `leave`

Leave the current room.

```typescript
leave(): void
```

---

### Message Codec

Protobuf-based binary message encoder/decoder for network messages.

**Source:** `src/lib/network/message-codec.ts`

#### `initCodec`

Initialize the message codec with the protobuf module.

```typescript
import { initCodec, MessageCodec } from '$lib/network/message-codec';

await initCodec();
```

**Note:** Must be called once before using `MessageCodec`. Loads the generated protobuf definitions.

---

#### `MessageCodec`

Static encode/decode utility for network messages.

```typescript
class MessageCodec {
  static encode(message: NetworkMessage): Uint8Array;
  static decode(data: Uint8Array): NetworkMessage;
}
```

##### `encode`

Encode a message to binary.

```typescript
static encode(message: NetworkMessage): Uint8Array
```

##### `decode`

Decode a binary message.

```typescript
static decode(data: Uint8Array): NetworkMessage
```

---

#### Message Types

```typescript
type NetworkMessage =
  | PingMessage
  | PongMessage
  | CheckpointMessage
  | WinnerMessage
  | PeerDiscoveryMessage
  | VdfProgressMessage
  | ConsensusProposalMessage
  | ConsensusVoteMessage
  | NewRoundMessage;
```

| Type                 | Description                 |
| -------------------- | --------------------------- |
| `ping`               | Keep-alive ping             |
| `pong`               | Keep-alive pong response    |
| `checkpoint`         | VDF checkpoint data         |
| `winner`             | VRF winner announcement     |
| `peer-discovery`     | Peer list update            |
| `vdf-progress`       | VDF computation progress    |
| `consensus-proposal` | Block proposal broadcast    |
| `consensus-vote`     | Prevote/precommit broadcast |
| `new-round`          | New round announcement      |

---

### Consensus Message Helpers

Helper functions for creating consensus-related network messages.

**Source:** `src/lib/network/message-codec.ts`

#### `createConsensusProposal`

Create a consensus proposal message for broadcast.

```typescript
function createConsensusProposal(
  round: number,
  vdfState: Uint8Array,
  vrfProof: Uint8Array,
  blockHash: Uint8Array,
  prevBlockHash: Uint8Array,
  timestamp: number
): ConsensusProposalMessage;
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `round` | `number` | Consensus round number |
| `vdfState` | `Uint8Array` | VDF state at proposal time |
| `vrfProof` | `Uint8Array` | VRF proof of leader election |
| `blockHash` | `Uint8Array` | Hash of the proposed block |
| `prevBlockHash` | `Uint8Array` | Hash of the previous block |
| `timestamp` | `number` | Proposal timestamp (Unix ms) |

---

#### `createConsensusVote`

Create a consensus vote message (prevote or precommit).

```typescript
function createConsensusVote(
  round: number,
  phase: 'prevote' | 'precommit',
  blockHash: Uint8Array,
  voterPubkey: Uint8Array
): ConsensusVoteMessage;
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `round` | `number` | Consensus round number |
| `phase` | `'prevote' \| 'precommit'` | Vote phase |
| `blockHash` | `Uint8Array` | Hash being voted on |
| `voterPubkey` | `Uint8Array` | Voter's public key |

---

#### `createNewRound`

Create a new round announcement message.

```typescript
function createNewRound(round: number, seed: Uint8Array, validators: Uint8Array[]): NewRoundMessage;
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `round` | `number` | New round number |
| `seed` | `Uint8Array` | Round seed (from previous block hash) |
| `validators` | `Uint8Array[]` | Validator set for the new round |

---

## Web Worker API

The Web Worker runs VDF computation in the background and communicates with the main thread via messages.

### Messages (Main Thread → Worker)

#### `start`

Start VDF computation.

```typescript
interface StartMessage {
  type: 'start';
  seed: Uint8Array; // 32-byte seed
  total: number; // Total steps
  k: number; // VRF sampling interval
  tau: Uint8Array; // 32-byte threshold
  checkpointInterval: number; // Checkpoint interval
}
```

**Example:**

```typescript
worker.postMessage({
  type: 'start',
  seed: new Uint8Array(32),
  total: 1000000,
  k: 1000,
  tau: new Uint8Array(32),
  checkpointInterval: 100000
});
```

---

#### `pause`

Pause computation.

```typescript
interface PauseMessage {
  type: 'pause';
}
```

**Example:**

```typescript
worker.postMessage({ type: 'pause' });
```

---

#### `resume`

Resume computation.

```typescript
interface ResumeMessage {
  type: 'resume';
}
```

**Example:**

```typescript
worker.postMessage({ type: 'resume' });
```

---

#### `stop`

Stop computation.

```typescript
interface StopMessage {
  type: 'stop';
}
```

**Example:**

```typescript
worker.postMessage({ type: 'stop' });
```

---

### Messages (Worker → Main Thread)

#### `started`

Computation started successfully.

```typescript
interface StartedMessage {
  type: 'started';
  publicKey: Uint8Array; // Node's VRF public key
}
```

---

#### `progress`

Progress update (sent every second).

```typescript
interface ProgressMessage {
  type: 'progress';
  step: number; // Current step
  speed: number; // Steps per second
  memoryUsage: number; // Memory usage in bytes
}
```

---

#### `winner`

VRF winner found.

```typescript
interface WinnerMessage {
  type: 'winner';
  step: number; // Winner step
  proof: Uint8Array; // VRF proof
}
```

---

#### `finished`

Computation finished.

```typescript
interface FinishedMessage {
  type: 'finished';
  step: number; // Final step
}
```

---

#### `heartbeat`

Keep-alive signal (sent every 10 seconds).

```typescript
interface HeartbeatMessage {
  type: 'heartbeat';
  timestamp: number; // Unix timestamp (ms)
  status: string; // 'running' | 'paused'
}
```

---

#### `error`

Error occurred.

```typescript
interface ErrorMessage {
  type: 'error';
  code: string; // Error code
  message: string; // Error description
  recoverable: boolean; // Whether error is recoverable
}
```

---

### Message Types

```typescript
type WorkerMessage = StartMessage | PauseMessage | ResumeMessage | StopMessage;

type WorkerResponse =
  | StartedMessage
  | ProgressMessage
  | WinnerMessage
  | FinishedMessage
  | HeartbeatMessage
  | ErrorMessage;
```

---

## Svelte Stores

### Worker State Store

Reactive store for worker state.

```typescript
import { workerState } from '$stores/worker';

// Subscribe to changes
workerState.subscribe((state) => {
  console.log('Is running:', state.isRunning);
  console.log('Current step:', state.currentStep);
});
```

#### `WorkerState` Interface

```typescript
interface WorkerState {
  isRunning: boolean; // Whether computation is running
  isPaused: boolean; // Whether computation is paused
  currentStep: number; // Current VDF step
  totalSteps: number; // Total steps target
  speed: number; // Current speed (steps/sec)
  uptime: number; // Uptime in seconds
  winnerCount: number; // Number of winners found
  luckPercentage: number; // Luck percentage
  publicKey: Uint8Array | null; // Node's public key
  nodeId: string; // Node identifier
}
```

---

### Events Store

Store for event log.

```typescript
import { events } from '$stores/worker';

// Subscribe to events
events.subscribe((eventList) => {
  console.log('Events:', eventList);
});
```

#### `VtpEvent` Interface

```typescript
interface VtpEvent {
  type: 'info' | 'checkpoint' | 'winner' | 'error';
  timestamp: number; // Unix timestamp (ms)
  message: string; // Event message
}
```

---

### Progress Store

Derived store for progress percentage.

```typescript
import { progress } from '$stores/worker';

// Subscribe to progress
progress.subscribe((value) => {
  console.log('Progress:', (value * 100).toFixed(1) + '%');
});
```

**Type:** `Readable<number>` (0 to 1)

---

### Store Functions

#### `addEvent`

Add event to event log.

```typescript
function addEvent(event: Omit<VtpEvent, 'timestamp'>): void;
```

**Parameters:**
| Parameter | Type | Description |
|-----------|------|-------------|
| `event` | `Omit<VtpEvent, 'timestamp'>` | Event without timestamp |

**Example:**

```typescript
addEvent({ type: 'info', message: 'Computation started' });
addEvent({ type: 'winner', message: '🎉 Winner at step 12345' });
```

---

#### `resetWorkerState`

Reset worker state to initial values.

```typescript
function resetWorkerState(): void;
```

**Example:**

```typescript
resetWorkerState();
```

---

## Utility Functions

### Formatting Functions

#### `formatBytes`

Format bytes to human-readable string.

```typescript
function formatBytes(bytes: number): string;
```

**Example:**

```typescript
formatBytes(0); // "0 B"
formatBytes(1024); // "1 KB"
formatBytes(1048576); // "1 MB"
```

---

#### `formatNumber`

Format number with locale-specific separators.

```typescript
function formatNumber(num: number): string;
```

**Example:**

```typescript
formatNumber(1234567); // "1,234,567"
```

---

#### `formatSpeed`

Format speed value.

```typescript
function formatSpeed(speed: number): string;
```

**Example:**

```typescript
formatSpeed(1500000); // "1.5M"
formatSpeed(1500); // "1.5K"
formatSpeed(500); // "500"
```

---

#### `formatTime`

Format seconds to HH:MM:SS.

```typescript
function formatTime(seconds: number): string;
```

**Example:**

```typescript
formatTime(0); // "00:00:00"
formatTime(61); // "00:01:01"
formatTime(3661); // "01:01:01"
```

---

### Generation Functions

#### `generateNodeId`

Generate 8-character hex node ID.

```typescript
function generateNodeId(): string;
```

**Example:**

```typescript
generateNodeId(); // "a1b2c3d4"
```

**Note:** Uses Math.random(), not cryptographically secure

---

### Async Utilities

#### `sleep`

Async sleep function.

```typescript
function sleep(ms: number): Promise<void>;
```

**Example:**

```typescript
async function example() {
  console.log('Start');
  await sleep(1000);
  console.log('End');
}
```

---

#### `debounce`

Create debounced function.

```typescript
function debounce<T extends (...args: any[]) => any>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void;
```

**Example:**

```typescript
const debouncedSearch = debounce((query: string) => {
  console.log('Searching:', query);
}, 300);

debouncedSearch('a');
debouncedSearch('ab');
debouncedSearch('abc');
// Only 'abc' will be logged after 300ms
```

---

## TypeScript Interfaces

### Worker Message Types

```typescript
// Main Thread → Worker
interface WorkerMessage {
  type: string;
  seed?: Uint8Array;
  total?: number;
  k?: number;
  tau?: Uint8Array;
  checkpointInterval?: number;
  maxSteps?: number;
}

// Worker → Main Thread
interface ProgressMessage {
  type: 'progress';
  step: number;
  speed: number;
  memoryUsage: number;
}

interface WinnerMessage {
  type: 'winner';
  step: number;
  proof: Uint8Array;
}

interface HeartbeatMessage {
  type: 'heartbeat';
  timestamp: number;
  status: string;
}

interface ErrorMessage {
  type: 'error';
  code: string;
  message: string;
  recoverable: boolean;
}

interface StartedMessage {
  type: 'started';
  publicKey: Uint8Array;
}

interface StoppedMessage {
  type: 'stopped';
}

interface FinishedMessage {
  type: 'finished';
  step: number;
}

type WorkerResponse =
  | ProgressMessage
  | WinnerMessage
  | HeartbeatMessage
  | ErrorMessage
  | StartedMessage
  | StoppedMessage
  | FinishedMessage;
```

---

## Error Codes

| Code                | Description                   | Recoverable |
| ------------------- | ----------------------------- | ----------- |
| `UNKNOWN_COMMAND`   | Unknown command received      | No          |
| `INVALID_PARAMS`    | Missing or invalid parameters | No          |
| `INIT_FAILED`       | Session initialization failed | Yes         |
| `COMPUTATION_ERROR` | Error during computation      | Yes         |
| `VDF_ERROR`         | VDF computation error         | Yes         |

---

## Examples

### Basic Usage

```typescript
// 1. Create worker
const worker = new Worker(new URL('$lib/worker/index.ts', import.meta.url), {
  type: 'module'
});

// 2. Listen for messages
worker.onmessage = (event) => {
  const msg = event.data;
  switch (msg.type) {
    case 'progress':
      console.log(`Step: ${msg.step}, Speed: ${msg.speed}`);
      break;
    case 'winner':
      console.log(`Winner at step ${msg.step}`);
      break;
    case 'error':
      console.error(`Error: ${msg.message}`);
      break;
  }
};

// 3. Start computation
worker.postMessage({
  type: 'start',
  seed: new Uint8Array(32),
  total: 1000000,
  k: 1000,
  tau: new Uint8Array(32),
  checkpointInterval: 100000
});

// 4. Pause after 10 seconds
setTimeout(() => {
  worker.postMessage({ type: 'pause' });
}, 10000);

// 5. Resume after 5 seconds
setTimeout(() => {
  worker.postMessage({ type: 'resume' });
}, 15000);

// 6. Stop after 1 minute
setTimeout(() => {
  worker.postMessage({ type: 'stop' });
  worker.terminate();
}, 60000);
```

### Using Svelte Stores

```svelte
<script lang="ts">
  import { workerState, events, addEvent, resetWorkerState } from '$stores/worker';

  // Subscribe to state changes
  $: {
    if ($workerState.isRunning) {
      console.log('Running at speed:', $workerState.speed);
    }
  }

  // Add event
  function handleWinner() {
    addEvent({ type: 'winner', message: '🎉 Winner found!' });
  }

  // Reset state
  function handleReset() {
    resetWorkerState();
  }
</script>

<div>
  <p>Speed: {$workerState.speed} steps/sec</p>
  <p>Progress: {$workerState.currentStep} / {$workerState.totalSteps}</p>

  {#each $events as event}
    <p>{event.message}</p>
  {/each}
</div>
```

### Consensus Round Lifecycle

```typescript
// Consensus round lifecycle
import { ConsensusEngine } from '$lib/consensus';

const engine = new ConsensusEngine(secretKey, publicKey, validators, {
  tau: new Uint8Array(32).fill(0xff),
  checkpointInterval: 1000,
  roundTimeoutMs: 30_000
});

engine.on('onCommit', (round, block) => {
  console.log(`Block finalized at round ${round}`);
});

// Start round
const vrfProof = engine.startRound(1);

if (engine.isLeader) {
  const proposal = engine.propose(vdfState, Date.now());
  // broadcast proposal
} else {
  // wait for proposal, validate, vote
}
```

---

## Support

For API questions or issues:

- **GitHub Issues**: [Create an issue](https://github.com/your-org/vtp-node/issues)
- **Email**: api@vtp-node.dev

---

<div align="center">

**[⬆ Back to Top](#vtp-node-api-reference)**

</div>
