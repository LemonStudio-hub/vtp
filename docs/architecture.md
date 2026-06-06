# VTP Node Architecture Guide

<div align="center">

**System architecture and design decisions for the VTP Node project**

[Overview](#overview) • [System Architecture](#system-architecture) • [Component Design](#component-design) • [Data Flow](#data-flow) • [Performance](#performance) • [Security](#security)

</div>

---

## Table of Contents

- [Overview](#overview)
- [System Architecture](#system-architecture)
  - [High-Level Architecture](#high-level-architecture)
  - [Layer Descriptions](#layer-descriptions)
  - [Technology Stack](#technology-stack)
- [Component Design](#component-design)
  - [Rust Core Library](#rust-core-library)
  - [Web Worker](#web-worker)
  - [Svelte Frontend](#svelte-frontend)
  - [PWA Layer](#pwa-layer)
- [Data Flow](#data-flow)
  - [Initialization Flow](#initialization-flow)
  - [Computation Flow](#computation-flow)
  - [Checkpoint Flow](#checkpoint-flow)
  - [Communication Protocol](#communication-protocol)
- [Performance](#performance)
  - [Optimization Strategies](#optimization-strategies)
  - [Memory Management](#memory-management)
  - [Background Execution](#background-execution)
- [Security](#security)
  - [Cryptographic Design](#cryptographic-design)
  - [Key Management](#key-management)
  - [Data Protection](#data-protection)
- [Browser Compatibility](#browser-compatibility)
- [Future Considerations](#future-considerations)

---

## Overview

VTP Node is designed as a browser-based implementation of the Verifiable Time Proof protocol. The architecture prioritizes:

1. **Performance**: Efficient computation using WebAssembly
2. **Responsiveness**: Non-blocking UI with Web Workers
3. **Reliability**: Robust error handling and state persistence
4. **Security**: Cryptographic operations in isolated environment

### Design Goals

- Run VDF computation entirely in the browser
- Maintain responsive UI during computation
- Support long-running computations (hours/days)
- Provide real-time visualization
- Enable PWA installation for better UX

### Constraints

- Browser sandbox limitations
- Single-threaded JavaScript (mitigated with Web Workers)
- Limited background execution (mitigated with AudioContext)
- No direct filesystem access (mitigated with IndexedDB)

---

## System Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Browser Environment                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │                      Presentation Layer                          │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │ │
│  │  │  Dashboard   │  │   Stats     │  │      Event Log          │ │ │
│  │  │  Component   │  │   Panel     │  │      Component          │ │ │
│  │  └──────────────┘  └─────────────┘  └─────────────────────────┘ │ │
│  │                            │                                     │ │
│  │                     Svelte Store                                 │ │
│  └────────────────────────────┼─────────────────────────────────────┘ │
│                               │                                       │
│  ┌────────────────────────────┴─────────────────────────────────────┐ │
│  │                      Communication Layer                         │ │
│  │                      (postMessage API)                           │ │
│  └────────────────────────────┬─────────────────────────────────────┘ │
│                               │                                       │
│  ┌────────────────────────────┴─────────────────────────────────────┐ │
│  │                        Computation Layer                         │ │
│  │  ┌─────────────────────────────────────────────────────────────┐ │ │
│  │  │                    Web Worker                                │ │ │
│  │  │  ┌─────────────────────────────────────────────────────┐   │ │ │
│  │  │  │              vtp-core (WebAssembly)                  │   │ │ │
│  │  │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │   │ │ │
│  │  │  │  │  VDF Engine │  │  VRF Engine │  │   Session   │ │   │ │ │
│  │  │  │  │ (Class Grp) │  │ (ED25519)   │  │   Manager   │ │   │ │ │
│  │  │  │  └─────────────┘  └─────────────┘  └─────────────┘ │   │ │ │
│  │  │  └─────────────────────────────────────────────────────┘   │ │ │
│  │  │  ┌─────────────────────────────────────────────────────┐   │ │ │
│  │  │  │  Scheduler │  Checkpoint  │  Error Handler         │   │ │ │
│  │  │  └─────────────────────────────────────────────────────┘   │ │ │
│  │  └─────────────────────────────────────────────────────────────┘ │ │
│  └──────────────────────────────────────────────────────────────────┘ │
│                                                                       │
│  ┌──────────────────────────────────────────────────────────────────┐ │
│  │                        Storage Layer                              │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │ │
│  │  │  IndexedDB   │  │   Cache     │  │   Local Storage         │ │ │
│  │  │ (Checkpoints)│  │   Storage   │  │   (Settings)            │ │ │
│  │  └─────────────┘  └─────────────┘  └─────────────────────────┘ │ │
│  └──────────────────────────────────────────────────────────────────┘ │
│                                                                       │
└─────────────────────────────────────────────────────────────────────┘
```

### Layer Descriptions

#### Presentation Layer

- **Dashboard Component**: Main UI container
- **Stats Panel**: Real-time statistics display
- **Event Log**: Event history display
- **Identity Badge**: Node identity visualization
- **VDF Canvas**: Progress visualization

#### Communication Layer

- **postMessage API**: Thread-safe communication
- **Message Protocol**: Typed message format
- **Serialization**: Efficient binary transfer

#### Computation Layer

- **Web Worker**: Background thread
- **vtp-core (Wasm)**: Cryptographic operations
- **Scheduler**: Time-sliced execution
- **Checkpoint**: State persistence
- **Error Handler**: Error recovery

#### Storage Layer

- **IndexedDB**: Checkpoint storage
- **Cache Storage**: PWA caching
- **Local Storage**: User preferences

### Technology Stack

| Layer            | Technology               | Purpose                    |
| ---------------- | ------------------------ | -------------------------- |
| **Frontend**     | Svelte 4                 | Reactive UI framework      |
| **Build Tool**   | Vite 5                   | Fast development and build |
| **Language**     | TypeScript 5             | Type-safe JavaScript       |
| **Core Library** | Rust + wasm-pack         | High-performance Wasm      |
| **Crypto**       | Wesolowski VDF + ED25519 | VDF and VRF operations     |
| **Styling**      | CSS                      | Component styling          |
| **Storage**      | IndexedDB                | Checkpoint persistence     |
| **PWA**          | Workbox                  | Service worker caching     |

---

## Component Design

### Rust Core Library

The `vtp-core` crate is the heart of the system, providing cryptographic operations compiled to WebAssembly.

#### Module Structure

```
vtp-core/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Entry point and Wasm bindings
│   ├── vdf.rs          # VDF implementation
│   ├── vrf.rs          # VRF implementation
│   ├── session.rs      # Session management
│   ├── error.rs        # Error types
│   └── utils.rs        # Utility functions
└── tests/
```

#### Key Design Decisions

1. **Wesolowski VDF over Imaginary Quadratic Class Groups**: Chosen for VDF because:
   - Provably sequential: computing g^(2^T) requires T sequential squarings
   - Efficient verification: O(log l) group operations regardless of T
   - Well-studied security based on class group hardness assumption
   - Compatible with WebAssembly via `num-bigint` for large-integer arithmetic

2. **ED25519 for VRF**: Chosen because:
   - Fast signature generation
   - Small signature size (64 bytes)
   - Well-supported in Rust ecosystem
   - Good security properties

3. **wasm-bindgen**: Used for JavaScript interop because:
   - Automatic type conversion
   - Memory management
   - Error handling
   - Well-maintained

#### Memory Layout

```
┌─────────────────────────────────────────────┐
│                Wasm Memory                   │
├─────────────────────────────────────────────┤
│  Stack                                        │
├─────────────────────────────────────────────┤
│  Heap                                         │
│  ┌─────────────┐  ┌─────────────────────┐  │
│  │ VDF State   │  │ VRF Keys            │  │
│  │ (BQF elem)  │  │ (64 bytes)          │  │
│  └─────────────┘  └─────────────────────┘  │
│  ┌─────────────┐  ┌─────────────────────┐  │
│  │ Session     │  │ Temporary Buffers   │  │
│  │ State       │  │                     │  │
│  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────┤
│  Static Data                                  │
└─────────────────────────────────────────────┘
```

---

### Web Worker

The Web Worker manages the computation lifecycle and communicates with the main thread.

#### Architecture

```
┌─────────────────────────────────────────────┐
│                 Web Worker                    │
├─────────────────────────────────────────────┤
│                                               │
│  ┌─────────────────────────────────────────┐ │
│  │           Message Handler                │ │
│  │  ┌──────────┐  ┌──────────┐  ┌────────┐ │ │
│  │  │  start   │  │  pause   │  │ resume │ │ │
│  │  └──────────┘  └──────────┘  └────────┘ │ │
│  └─────────────────────────────────────────┘ │
│                     │                         │
│  ┌──────────────────┴───────────────────────┐ │
│  │           Main Loop                       │ │
│  │  ┌─────────────────────────────────────┐ │ │
│  │  │  Time Slice Manager                 │ │ │
│  │  │  - Check elapsed time               │ │ │
│  │  │  - Yield to browser if needed       │ │ │
│  │  └─────────────────────────────────────┘ │ │
│  │  ┌─────────────────────────────────────┐ │ │
│  │  │  Batch Processor                    │ │ │
│  │  │  - Execute VDF steps                │ │ │
│  │  │  - Check for checkpoints            │ │ │
│  │  │  - Generate VRF proofs              │ │ │
│  │  └─────────────────────────────────────┘ │ │
│  │  ┌─────────────────────────────────────┐ │ │
│  │  │  Progress Reporter                  │ │ │
│  │  │  - Calculate speed                  │ │ │
│  │  │  - Send updates                     │ │ │
│  │  └─────────────────────────────────────┘ │ │
│  └─────────────────────────────────────────┘ │
│                                               │
│  ┌─────────────────────────────────────────┐ │
│  │           Support Services               │ │
│  │  ┌──────────┐  ┌──────────┐  ┌────────┐ │ │
│  │  │ Scheduler│  │Checkpoint│  │ Error  │ │ │
│  │  │          │  │ Manager  │  │Handler │ │ │
│  │  └──────────┘  └──────────┘  └────────┘ │ │
│  └─────────────────────────────────────────┘ │
│                                               │
└─────────────────────────────────────────────┘
```

#### Time-Slicing Strategy

```typescript
async function runMainLoop() {
  const TIME_SLICE_MS = 50; // 50ms per slice

  while (isRunning) {
    const start = performance.now();

    // Execute batch
    const result = session.run_batch(1000);

    // Check time
    const elapsed = performance.now() - start;
    if (elapsed < TIME_SLICE_MS) {
      // Yield to browser
      await sleep(TIME_SLICE_MS - elapsed);
    }
  }
}
```

---

### Svelte Frontend

The frontend uses Svelte's reactive system for efficient UI updates.

#### Component Hierarchy

```
App
├── Dashboard
│   ├── IdentityBadge
│   │   ├── Canvas (Identicon)
│   │   └── NodeInfo
│   ├── VDFCanvas
│   │   ├── ProgressRing
│   │   └── ParticleSystem
│   ├── StatsPanel
│   │   ├── SpeedDisplay
│   │   ├── StepCounter
│   │   ├── UptimeDisplay
│   │   └── LuckDisplay
│   ├── ControlButtons
│   │   ├── StartButton
│   │   ├── PauseButton
│   │   └── ResumeButton
│   └── EventLog
│       └── EventItem
└── PWAInstall
```

#### State Management

```typescript
// Store hierarchy
workerStore          // Worker instance
├── workerState      // Computed state
│   ├── isRunning
│   ├── currentStep
│   ├── speed
│   └── ...
└── events           // Event log
    └── VtpEvent[]
```

#### Reactive Updates

```svelte
<script>
  import { workerState } from '$stores/worker';

  // Reactive declaration
  $: progress = $workerState.currentStep / $workerState.totalSteps;
  $: formattedSpeed = formatSpeed($workerState.speed);
</script>

<div>
  <p>Progress: {(progress * 100).toFixed(1)}%</p>
  <p>Speed: {formattedSpeed}</p>
</div>
```

---

### PWA Layer

The PWA layer enables installation and offline support.

#### Service Worker Strategy

```javascript
// Cache strategies
const strategies = {
  // Static assets: Cache-first
  static: new CacheFirst({
    cacheName: 'static-v1',
    plugins: [new ExpirationPlugin({ maxAgeSeconds: 30 * 24 * 60 * 60 })]
  }),

  // API calls: Network-first
  api: new NetworkFirst({
    cacheName: 'api-v1',
    plugins: [new ExpirationPlugin({ maxEntries: 50 })]
  }),

  // Images: Cache-first with expiration
  images: new CacheFirst({
    cacheName: 'images-v1',
    plugins: [new ExpirationPlugin({ maxEntries: 100, maxAgeSeconds: 7 * 24 * 60 * 60 })]
  })
};
```

---

## Data Flow

### Initialization Flow

```
User                    Main Thread              Worker                    Wasm
 │                         │                       │                         │
 │   Click Start           │                       │                         │
 ├────────────────────────►│                       │                         │
 │                         │   postMessage(start)  │                         │
 │                         ├──────────────────────►│                         │
 │                         │                       │   Session::new()        │
 │                         │                       ├────────────────────────►│
 │                         │                       │◄────────────────────────┤
 │                         │                       │   Session created       │
 │                         │   postMessage(started)│                         │
 │                         │◄──────────────────────┤                         │
 │   Update UI             │                       │                         │
 │◄────────────────────────┤                       │                         │
```

### Computation Flow

```
Worker                    Wasm                    Main Thread
 │                         │                         │
 │   run_batch(1000)       │                         │
 ├────────────────────────►│                         │
 │                         │   Execute 1000 steps    │
 │                         │◄────────────────────────┤
 │   BatchResult           │                         │
 │◄────────────────────────┤                         │
 │                         │                         │
 │   postMessage(progress) │                         │
 ├─────────────────────────────────────────────────►│
 │                         │                         │   Update UI
 │                         │                         │◄────────────────────┤
 │                         │                         │
 │   sleep(50ms)           │                         │
 ├────────────────────────►│                         │
 │   Yield to browser      │                         │
 │◄────────────────────────┤                         │
```

### Checkpoint Flow

```
Worker                    IndexedDB                 Main Thread
 │                         │                         │
 │   is_checkpoint_step()  │                         │
 ├────────────────────────►│                         │
 │   true                  │                         │
 │◄────────────────────────┤                         │
 │                         │                         │
 │   get_checkpoint_data() │                         │
 ├────────────────────────►│                         │
 │   checkpoint data       │                         │
 │◄────────────────────────┤                         │
 │                         │                         │
 │   Save to IndexedDB     │                         │
 ├────────────────────────►│                         │
 │                         │   Write checkpoint      │
 │                         │◄────────────────────────┤
 │                         │   Success               │
 │                         ├────────────────────────►│
 │                         │                         │
 │   postMessage(checkpoint)                         │
 ├─────────────────────────────────────────────────►│
```

### Communication Protocol

#### Message Format

```typescript
// All messages use this base structure
interface Message {
  type: string; // Message type identifier
  timestamp?: number; // Optional timestamp
}
```

#### Transferable Objects

```typescript
// Use Transferable for large data
worker.postMessage(
  {
    type: 'data',
    buffer: largeArrayBuffer
  },
  [largeArrayBuffer]
); // Transfer ownership
```

#### Error Handling

```typescript
// Worker error handling
try {
  const result = session.run_batch(1000);
} catch (error) {
  self.postMessage({
    type: 'error',
    code: 'COMPUTATION_ERROR',
    message: error.message,
    recoverable: true
  });
}
```

---

## Performance

### Optimization Strategies

#### 1. WebAssembly Optimization

```toml
# Cargo.toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = true           # Link-time optimization
codegen-units = 1    # Single codegen unit
```

#### 2. Batch Processing

```typescript
// Process in batches to reduce overhead
const BATCH_SIZE = 1000;

function runBatch() {
  const start = performance.now();

  for (let i = 0; i < BATCH_SIZE; i++) {
    vdfStep();
  }

  const elapsed = performance.now() - start;
  const stepsPerSecond = BATCH_SIZE / (elapsed / 1000);
}
```

#### 3. Time-Slicing

```typescript
// Yield to browser every 50ms
const TIME_SLICE_MS = 50;

async function runWithTimeSlice() {
  while (isRunning) {
    const start = performance.now();

    // Execute batch
    executeBatch();

    // Yield if needed
    const elapsed = performance.now() - start;
    if (elapsed < TIME_SLICE_MS) {
      await sleep(TIME_SLICE_MS - elapsed);
    }
  }
}
```

#### 4. Efficient Communication

```typescript
// Use typed arrays for binary data
const state = new Uint8Array(32);
worker.postMessage({ type: 'state', data: state }, [state.buffer]);
```

### Memory Management

#### Wasm Memory

```rust
// Rust automatically manages memory
// Use Vec<u8> for byte buffers
// Memory is freed when values go out of scope
```

#### JavaScript Memory

```typescript
// Monitor memory usage
function getMemoryUsage(): number {
  const perf = performance as any;
  if (perf.memory) {
    return perf.memory.usedJSHeapSize;
  }
  return 0;
}

// Check periodically
setInterval(() => {
  const usage = getMemoryUsage();
  if (usage > MAX_MEMORY) {
    // Trigger garbage collection
  }
}, 10000);
```

### Background Execution

#### AudioContext Strategy

```typescript
// Use AudioContext for background execution
const audioContext = new AudioContext();
const oscillator = audioContext.createOscillator();
const gain = audioContext.createGain();

// Silent audio keeps the tab active
gain.gain.value = 0;
oscillator.connect(gain);
gain.connect(audioContext.destination);
oscillator.start();
```

#### Fallback Strategy

```typescript
// Progressive enhancement
function getScheduler() {
  if (typeof AudioContext !== 'undefined') {
    return new AudioContextScheduler();
  } else if (typeof setInterval !== 'undefined') {
    return new IntervalScheduler();
  } else {
    return new TimeoutScheduler();
  }
}
```

---

## Security

### Cryptographic Design

#### VDF Security

- **Sequential**: Class group squarings cannot be parallelised (sequential squaring assumption)
- **Deterministic**: Same input produces same output
- **Verifiable**: Wesolowski proof allows O(log l) verification regardless of delay T

#### VRF Security

- **Uniqueness**: Each input produces unique output
- **Unpredictability**: Output is unpredictable without secret key
- **Verifiability**: Anyone can verify with public key

### Key Management

#### Key Generation

```rust
// Generate keypair using OS random
let signing_key = SigningKey::generate(&mut OsRng);
```

#### Key Storage

- Keys are generated in the browser
- Private keys never leave the browser
- Keys are stored in memory only
- Keys are lost when tab is closed

### Data Protection

#### In-Memory

```rust
// Sensitive data is zeroed when dropped
impl Drop for SecretKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}
```

#### In-Transit

```typescript
// Use Transferable to avoid copying
worker.postMessage({ key: secretKey }, [secretKey.buffer]);
```

#### In-Storage

```typescript
// IndexedDB storage
const db = await openDB('vtp-db', 1, {
  upgrade(db) {
    db.createObjectStore('checkpoints');
  }
});

// Store checkpoint (no sensitive data)
await db.put('checkpoints', checkpointData, checkpointId);
```

---

## Browser Compatibility

### Feature Detection

```typescript
// Check for required features
function checkCompatibility(): boolean {
  return (
    typeof WebAssembly !== 'undefined' &&
    typeof Worker !== 'undefined' &&
    typeof IndexedDB !== 'undefined' &&
    typeof AudioContext !== 'undefined'
  );
}
```

### Progressive Enhancement

```typescript
// Fallback chain
const features = {
  wasm: typeof WebAssembly !== 'undefined',
  worker: typeof Worker !== 'undefined',
  audioContext: typeof AudioContext !== 'undefined',
  performanceMemory: 'memory' in performance
};
```

### Browser-Specific Considerations

| Browser | Considerations                 |
| ------- | ------------------------------ |
| Chrome  | Full support, best performance |
| Firefox | No `performance.memory`        |
| Safari  | AudioContext limitations       |
| Mobile  | Background execution limits    |

---

## Future Considerations

### Potential Improvements

1. **SIMD Support**: Enable WebAssembly SIMD128 for faster big-integer operations
2. **SharedArrayBuffer**: Enable true multi-threading
3. **WebGPU**: Offload computation to GPU
4. **Web Locks API**: Better multi-tab coordination

### Scalability

1. **Multi-Node**: Support multiple VDF challenges
2. **Networking**: Add peer-to-peer communication
3. **Consensus**: Implement consensus protocol
4. **Storage**: Distributed storage system

### Research Directions

1. **Alternative VDF**: Explore different VDF constructions
2. **VRF Variants**: Implement different VRF schemes
3. **Optimization**: Further performance improvements
4. **Formal Verification**: Prove correctness of implementation

---

## Support

For architecture questions or discussions:

- **GitHub Discussions**: [Join the discussion](https://github.com/your-org/vtp-node/discussions)
- **Email**: architecture@vtp-node.dev

---

<div align="center">

**[⬆ Back to Top](#vtp-node-architecture-guide)**

</div>
