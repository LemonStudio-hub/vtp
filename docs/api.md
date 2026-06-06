# VTP Node API Reference

<div align="center">

**Complete API documentation for the VTP Node project**

[Overview](#overview) • [Rust Core API](#rust-core-api) • [Web Worker API](#web-worker-api) • [Svelte Stores](#svelte-stores) • [Utility Functions](#utility-functions)

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

---

## Support

For API questions or issues:

- **GitHub Issues**: [Create an issue](https://github.com/your-org/vtp-node/issues)
- **Email**: api@vtp-node.dev

---

<div align="center">

**[⬆ Back to Top](#vtp-node-api-reference)**

</div>
