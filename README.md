# VTP Node - Verifiable Time Proof

<div align="center">

![VTP Logo](https://trae-api-cn.mchost.guru/api/ide/v1/text_to_image?prompt=A%20modern%20minimalist%20logo%20for%20VTP%20Node%2C%20featuring%20a%20stylized%20clock%20or%20hourglass%20combined%20with%20blockchain%20elements%2C%20using%20teal%20and%20dark%20blue%20colors%2C%20clean%20geometric%20design%2C%20tech%20startup%20aesthetic&image_size=square)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Svelte](https://img.shields.io/badge/Svelte-4.2+-ff3e00.svg)](https://svelte.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.0+-blue.svg)](https://www.typescriptlang.org/)

**A browser-based implementation of Verifiable Delay Function (VDF) and Verifiable Random Function (VRF) for the VTP protocol.**

[Features](#features) • [Quick Start](#quick-start) • [Documentation](#documentation) • [API Reference](#api-reference) • [Contributing](#contributing)

</div>

---

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Architecture](#architecture)
- [Quick Start](#quick-start)
- [Installation](#installation)
- [Development](#development)
- [Building for Production](#building-for-production)
- [Testing](#testing)
- [Project Structure](#project-structure)
- [API Reference](#api-reference)
- [Configuration](#configuration)
- [Performance](#performance)
- [Browser Compatibility](#browser-compatibility)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgments](#acknowledgments)

---

## Overview

VTP Node is a single-node prototype implementation of the **Verifiable Time Proof (VTP)** protocol. It demonstrates the feasibility of running VDF and VRF computations entirely in the browser using WebAssembly.

### What is VTP?

**Verifiable Time Proof (VTP)** is a cryptographic protocol that combines:

- **VDF (Verifiable Delay Function)**: A function that takes a prescribed amount of time to compute, even with parallel processing, but produces a unique output that can be quickly verified.
- **VRF (Verifiable Random Function)**: A function that produces a pseudorandom output along with a proof that the output was computed correctly.

### Why Browser-Based?

- **Accessibility**: No installation required, runs in any modern browser
- **Transparency**: All computations are visible and verifiable
- **Decentralization**: Each user runs their own node
- **Privacy**: Private keys never leave the browser

---

## Features

### Core Features

| Feature                 | Description                                         | Status      |
| ----------------------- | --------------------------------------------------- | ----------- |
| **VDF Engine**          | Sequential SHA256 iteration compiled to WebAssembly | ✅ Complete |
| **VRF Implementation**  | ECVRF-ED25519 proof generation and verification     | ✅ Complete |
| **Web Worker**          | Background computation with time-slicing            | ✅ Complete |
| **Real-time Dashboard** | Live progress visualization with Canvas 2D          | ✅ Complete |
| **PWA Support**         | Installable Progressive Web App                     | ✅ Complete |
| **Checkpoint System**   | Automatic persistence with IndexedDB                | ✅ Complete |
| **Adaptive Scheduling** | AudioContext-based background execution             | ✅ Complete |

### Performance Targets

| Metric               | Target         | Description                     |
| -------------------- | -------------- | ------------------------------- |
| VDF Speed            | ≥ 2M steps/sec | Sequential SHA256 iterations    |
| VRF Latency          | ≤ 1ms          | Proof generation time           |
| Background Retention | ≥ 50%          | Speed when tab is in background |
| Memory Stability     | < 10MB growth  | Over 30 minutes of operation    |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Browser Environment                      │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │                    Main Thread (UI)                       │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │ │
│  │  │  Dashboard   │  │   Stats     │  │   Event Log     │ │ │
│  │  │  Component   │  │   Panel     │  │   Component     │ │ │
│  │  └──────────────┘  └─────────────┘  └─────────────────┘ │ │
│  └───────────────────────────┬─────────────────────────────┘ │
│                              │ postMessage                    │
│  ┌───────────────────────────┴─────────────────────────────┐ │
│  │                    Web Worker                             │ │
│  │  ┌─────────────────────────────────────────────────────┐ │ │
│  │  │              vtp-core (WebAssembly)                  │ │ │
│  │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │ │ │
│  │  │  │  VDF Engine │  │  VRF Engine │  │   Session   │ │ │ │
│  │  │  │  (SHA256)   │  │ (ED25519)   │  │   Manager   │ │ │ │
│  │  │  └─────────────┘  └─────────────┘  └─────────────┘ │ │ │
│  │  └─────────────────────────────────────────────────────┘ │ │
│  │  ┌─────────────────────────────────────────────────────┐ │ │
│  │  │  Scheduler │  Checkpoint  │  Error Handler         │ │ │
│  │  └─────────────────────────────────────────────────────┘ │ │
│  └──────────────────────────────────────────────────────────┘ │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow

1. **Initialization**: User starts VDF computation with seed and parameters
2. **Computation**: Worker performs sequential SHA256 iterations in batches
3. **Checkpointing**: State is periodically saved to IndexedDB
4. **VRF Sampling**: At each checkpoint interval, VRF proof is generated
5. **Progress Reporting**: Worker sends progress updates to main thread
6. **Visualization**: Dashboard displays real-time statistics and animations

---

## Quick Start

### Prerequisites

Before you begin, ensure you have the following installed:

- **Rust**: 1.70 or later ([Install Rust](https://www.rust-lang.org/tools/install))
- **wasm-pack**: 0.12 or later ([Install wasm-pack](https://rustwasm.github.io/wasm-pack/installer/))
- **Node.js**: 18 or later ([Install Node.js](https://nodejs.org/))
- **npm**: 9 or later (comes with Node.js)

### One-Line Setup

```bash
# Clone and setup in one command
git clone <repository-url> && cd vtp-node && npm install && npm run wasm:build
```

### Step-by-Step Setup

```bash
# 1. Clone the repository
git clone <repository-url>
cd vtp-node

# 2. Install Rust toolchain (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 3. Add WebAssembly target
rustup target add wasm32-unknown-unknown

# 4. Install wasm-pack
cargo install wasm-pack

# 5. Install Node.js dependencies
npm install

# 6. Build the Rust/Wasm library
npm run wasm:build

# 7. Start the development server
npm run dev
```

### Access the Application

Open your browser and navigate to:

```
http://localhost:5173
```

---

## Installation

### Using npm

```bash
npm install
```

### Using pnpm

```bash
pnpm install
```

### Using yarn

```bash
yarn install
```

---

## Development

### Development Server

Start the development server with hot reload:

```bash
npm run dev
```

The server will start at `http://localhost:5173` by default.

### Building Wasm

Build the Rust core library to WebAssembly:

```bash
npm run wasm:build
```

This will compile the `vtp-core` crate and output the Wasm files to `static/wasm/`.

### Watching for Changes

During development, you may want to watch for Rust changes:

```bash
# In terminal 1: Watch and rebuild Wasm
cd src/lib/vtp-core
cargo watch -w src -s 'wasm-pack build --target web --out-dir ../../../static/wasm'

# In terminal 2: Run dev server
npm run dev
```

### Code Formatting

Format all code:

```bash
# Format Rust code
cargo fmt

# Format TypeScript/Svelte code
npm run format
```

### Linting

Run linters:

```bash
# Lint all code
npm run lint

# Lint Rust code
cargo clippy

# Type check
npm run check
```

---

## Building for Production

### Full Build

```bash
npm run build
```

This will:

1. Build the Rust/Wasm library
2. Build the Worker
3. Build the Svelte application
4. Output to the `build/` directory

### Preview Production Build

```bash
npm run preview
```

### Build for Specific Environment

```bash
# Development build
NODE_ENV=development npm run build

# Production build (minified)
NODE_ENV=production npm run build
```

---

## Testing

### Rust Unit Tests

Run all Rust unit tests:

```bash
npm run wasm:test
```

Or directly with cargo:

```bash
cd src/lib/vtp-core
cargo test
```

### Frontend Tests

Run frontend tests with Vitest:

```bash
npm test
```

### Test with UI

Run tests with interactive UI:

```bash
npm run test:ui
```

### Test Coverage

Generate test coverage report:

```bash
npm run test:coverage
```

### Performance Benchmarks

Run Rust benchmarks:

```bash
cd src/lib/vtp-core
cargo bench
```

---

## Project Structure

```
vtp-node/
├── README.md                    # This file
├── package.json                 # Node.js dependencies and scripts
├── svelte.config.js             # SvelteKit configuration
├── vite.config.ts               # Vite build configuration
├── tsconfig.json                # TypeScript configuration
├── Cargo.toml                   # Rust workspace configuration
├── rust-toolchain.toml          # Rust toolchain specification
├── rustfmt.toml                 # Rust formatting configuration
├── .eslintrc.cjs                # ESLint configuration
├── .prettierrc                  # Prettier configuration
├── .editorconfig                # Editor configuration
├── .gitignore                   # Git ignore rules
├── .gitattributes               # Git attributes
├── .env.example                 # Environment variables example
├── LICENSE                      # MIT License
│
├── src/                         # Source code
│   ├── app.html                 # HTML entry point
│   │
│   ├── routes/                  # SvelteKit routes
│   │   ├── +layout.svelte       # Root layout
│   │   └── +page.svelte         # Main page
│   │
│   ├── components/              # Svelte components
│   │   ├── Dashboard.svelte     # Main dashboard
│   │   ├── StatsPanel.svelte    # Statistics panel
│   │   ├── EventLog.svelte      # Event log
│   │   ├── IdentityBadge.svelte # Node identity
│   │   ├── VDFCanvas.svelte     # VDF visualization
│   │   └── PWAInstall.svelte    # PWA install prompt
│   │
│   ├── stores/                  # Svelte stores
│   │   └── worker.ts            # Worker state management
│   │
│   ├── utils/                   # Utility functions
│   │   └── index.ts             # Common utilities
│   │
│   └── lib/                     # Libraries
│       ├── vtp-core/            # Rust core library
│       │   ├── Cargo.toml       # Rust package configuration
│       │   ├── src/             # Rust source code
│       │   │   ├── lib.rs       # Library entry point
│       │   │   ├── vdf.rs       # VDF implementation
│       │   │   ├── vrf.rs       # VRF implementation
│       │   │   ├── session.rs   # Session manager
│       │   │   ├── error.rs     # Error types
│       │   │   └── utils.rs     # Utility functions
│       │   └── tests/           # Rust tests
│       │
│       └── worker/              # Web Worker
│           ├── index.ts         # Worker entry point
│           └── types.ts         # Type definitions
│
├── static/                      # Static files
│   ├── manifest.json            # PWA manifest
│   ├── sw.js                    # Service Worker
│   ├── icons/                   # App icons
│   └── screenshots/             # App screenshots
│
├── tests/                       # Frontend tests
├── scripts/                     # Build scripts
│   └── build-worker.js          # Worker build script
│
└── docs/                        # Documentation
    ├── README.md                # Technical documentation
    ├── api.md                   # API reference
    ├── architecture.md          # Architecture guide
    ├── development.md           # Development guide
    └── deployment.md            # Deployment guide
```

---

## API Reference

### Rust Core Library (vtp-core)

#### VDF Module

```rust
/// Execute a single VDF step (SHA256 iteration)
pub fn vdf_step(state: &[u8; 32]) -> [u8; 32]

/// VDF Iterator for batch processing
pub struct VdfIterator { ... }

impl VdfIterator {
    pub fn new(seed: &[u8], total: u64) -> Self
    pub fn step(&self) -> u64
    pub fn total(&self) -> u64
    pub fn is_finished(&self) -> bool
    pub fn get_state(&self) -> Vec<u8>
    pub fn next(&mut self) -> bool
    pub fn run_batch(&mut self, max_steps: u64) -> u64
}
```

#### VRF Module

```rust
/// Generate a new VRF keypair
pub fn generate_keypair() -> VrfKeypair

/// Generate a VRF proof
pub fn prove(secret_key: &[u8], message: &[u8]) -> Vec<u8>

/// Verify a VRF proof
pub fn verify(public_key: &[u8], message: &[u8], proof: &[u8]) -> bool
```

#### Session Module

```rust
/// VDF Session Manager
pub struct Session { ... }

impl Session {
    pub fn new(seed: &[u8], total: u64, k: u64, tau: &[u8], checkpoint_interval: u64) -> Self
    pub fn state(&self) -> SessionState
    pub fn public_key(&self) -> Vec<u8>
    pub fn pause(&mut self)
    pub fn resume(&mut self)
    pub fn run_batch(&mut self, max_steps: u64) -> BatchResult
    pub fn get_checkpoint_data(&self) -> Vec<u8>
    pub fn verify_winner(&self, step: u64, proof: &[u8]) -> bool
}
```

### Web Worker API

#### Messages (Main Thread → Worker)

| Type     | Description           | Parameters                                        |
| -------- | --------------------- | ------------------------------------------------- |
| `start`  | Start VDF computation | `seed`, `total`, `k`, `tau`, `checkpointInterval` |
| `pause`  | Pause computation     | -                                                 |
| `resume` | Resume computation    | -                                                 |
| `stop`   | Stop computation      | -                                                 |

#### Messages (Worker → Main Thread)

| Type        | Description          | Data                             |
| ----------- | -------------------- | -------------------------------- |
| `started`   | Computation started  | `publicKey`                      |
| `progress`  | Progress update      | `step`, `speed`, `memoryUsage`   |
| `winner`    | VRF winner found     | `step`, `proof`                  |
| `finished`  | Computation finished | `step`                           |
| `heartbeat` | Keep-alive signal    | `timestamp`, `status`            |
| `error`     | Error occurred       | `code`, `message`, `recoverable` |

### Svelte Stores

```typescript
// Worker state store
export const workerState: Writable<WorkerState>;

// Events store
export const events: Writable<VtpEvent[]>;

// Progress store (derived)
export const progress: Readable<number>;

// Functions
export function addEvent(event: Omit<VtpEvent, 'timestamp'>): void;
export function resetWorkerState(): void;
```

---

## Configuration

### Environment Variables

Create a `.env` file in the root directory:

```env
# Application
NODE_ENV=development
PORT=5173

# VDF Configuration
VDF_DEFAULT_TOTAL=1000000
VDF_CHECKPOINT_INTERVAL=100000

# VRF Configuration
VRF_KEYPAIR_SIZE=32

# Performance
WORKER_BATCH_SIZE=1000
WORKER_TIME_SLICE_MS=50

# Logging
LOG_LEVEL=debug
ENABLE_PERFORMANCE_MONITORING=true

# PWA
ENABLE_PWA=true
SERVICE_WORKER_PATH=/sw.js
```

### Vite Configuration

Edit `vite.config.ts` to customize the build:

```typescript
export default defineConfig({
  plugins: [sveltekit()],
  server: {
    port: 5173,
    strictPort: false
  },
  build: {
    target: 'esnext',
    minify: 'esbuild',
    sourcemap: true
  }
});
```

### Rust Configuration

Edit `Cargo.toml` to customize Rust compilation:

```toml
[profile.release]
opt-level = 3      # Maximum optimization
lto = true         # Link-time optimization
codegen-units = 1  # Single codegen unit for better optimization
```

---

## Performance

### Optimization Tips

1. **Batch Size**: Adjust `WORKER_BATCH_SIZE` based on your needs
   - Larger batches = less overhead, but less responsive
   - Smaller batches = more responsive, but more overhead

2. **Time Slice**: Adjust `WORKER_TIME_SLICE_MS` for responsiveness
   - 50ms is a good default for smooth UI
   - Lower values = more responsive UI, but slower computation

3. **Memory Management**: Monitor memory usage in Chrome DevTools
   - Use the Memory tab to detect leaks
   - Check heap snapshots periodically

### Benchmarks

Run benchmarks to measure performance:

```bash
# Rust benchmarks
cd src/lib/vtp-core
cargo bench

# Frontend performance
npm run test:performance
```

### Profiling

Use Chrome DevTools for profiling:

1. Open DevTools (F12)
2. Go to Performance tab
3. Click Record
4. Perform actions
5. Click Stop
6. Analyze results

---

## Browser Compatibility

| Browser       | Version | Status          | Notes                    |
| ------------- | ------- | --------------- | ------------------------ |
| Chrome        | 90+     | ✅ Full Support | Best performance         |
| Edge          | 90+     | ✅ Full Support | Chromium-based           |
| Firefox       | 90+     | ✅ Supported    | No `performance.memory`  |
| Safari        | 15+     | ⚠️ Partial      | AudioContext limitations |
| Mobile Chrome | 90+     | ✅ Supported    | Performance varies       |
| Mobile Safari | 15+     | ⚠️ Partial      | Background limitations   |

### Feature Support

| Feature              | Chrome | Firefox | Safari |
| -------------------- | ------ | ------- | ------ |
| WebAssembly          | ✅     | ✅      | ✅     |
| Web Workers          | ✅     | ✅      | ✅     |
| AudioContext         | ✅     | ✅      | ⚠️     |
| IndexedDB            | ✅     | ✅      | ✅     |
| PWA Install          | ✅     | ❌      | ❌     |
| `performance.memory` | ✅     | ❌      | ❌     |

---

## Documentation

- **[Technical Documentation](docs/README.md)**: Detailed technical specifications
- **[API Reference](docs/api.md)**: Complete API documentation
- **[Architecture Guide](docs/architecture.md)**: System architecture overview
- **[Development Guide](docs/development.md)**: Development setup and workflow
- **[Deployment Guide](docs/deployment.md)**: Production deployment instructions

---

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Quick Contribution Steps

1. **Fork** the repository
2. **Clone** your fork:
   ```bash
   git clone https://github.com/your-username/vtp-node.git
   ```
3. **Create** a feature branch:
   ```bash
   git checkout -b feature/amazing-feature
   ```
4. **Commit** your changes:
   ```bash
   git commit -m 'feat: add amazing feature'
   ```
5. **Push** to the branch:
   ```bash
   git push origin feature/amazing-feature
   ```
6. **Open** a Pull Request

### Commit Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add new feature
fix: bug fix
docs: documentation changes
style: formatting changes
refactor: code refactoring
test: adding tests
chore: maintenance tasks
```

### Code Review Process

1. All submissions require review
2. Tests must pass
3. Code must follow style guidelines
4. Documentation must be updated

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

```
MIT License

Copyright (c) 2026 VTP Team

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

## Acknowledgments

- [curve25519-dalek](https://github.com/dalek-cryptography/curve25519-dalek) - ECVRF implementation
- [wasm-pack](https://github.com/rustwasm/wasm-pack) - WebAssembly tooling
- [Svelte](https://svelte.dev/) - Frontend framework
- [Vite](https://vitejs.dev/) - Build tool
- [Workbox](https://developers.google.com/web/tools/workbox) - PWA tooling

---

## Support

- **Issues**: [GitHub Issues](https://github.com/your-org/vtp-node/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/vtp-node/discussions)
- **Email**: support@vtp-node.dev

---

<div align="center">

**[⬆ Back to Top](#vtp-node---verifiable-time-proof)**

</div>
