# VTP Node Development Guide

<div align="center">

**Development setup, workflow, and best practices for VTP Node**

[Getting Started](#getting-started) • [Development Environment](#development-environment) • [Workflow](#workflow) • [Testing](#testing) • [Code Style](#code-style) • [Debugging](#debugging)

</div>

---

## Table of Contents

- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [IDE Setup](#ide-setup)
- [Workflow](#workflow)
  - [Development Server](#development-server)
  - [Building](#building)
  - [Hot Reload](#hot-reload)
  - [Git Workflow](#git-workflow)
- [Testing](#testing)
  - [Rust Tests](#rust-tests)
  - [Frontend Tests](#frontend-tests)
  - [Performance Tests](#performance-tests)
  - [Browser Testing](#browser-testing)
- [Code Style](#code-style)
  - [Rust Style](#rust-style)
  - [TypeScript Style](#typescript-style)
  - [Svelte Style](#svelte-style)
  - [Commit Convention](#commit-convention)
- [Debugging](#debugging)
  - [Rust Debugging](#rust-debugging)
  - [Frontend Debugging](#frontend-debugging)
  - [Worker Debugging](#worker-debugging)
  - [Performance Profiling](#performance-profiling)
- [Common Tasks](#common-tasks)
- [Troubleshooting](#troubleshooting)

---

## Getting Started

### Quick Start

```bash
# Clone the repository
git clone <repository-url>
cd vtp-node

# Install dependencies
npm install

# Build WebAssembly
npm run wasm:build

# Start development server
npm run dev
```

### First Steps

1. **Explore the codebase**: Read through the project structure
2. **Run the tests**: Ensure everything works
3. **Start the dev server**: See the application in action
4. **Make a change**: Try modifying a component
5. **Run the linter**: Check your code style

---

## Development Environment

### Prerequisites

#### Required Software

| Software | Version | Installation |
|----------|---------|--------------|
| **Rust** | 1.70+ | [rustup.rs](https://rustup.rs/) |
| **Node.js** | 18+ | [nodejs.org](https://nodejs.org/) |
| **npm** | 9+ | Comes with Node.js |
| **wasm-pack** | 0.12+ | `cargo install wasm-pack` |
| **Git** | 2.30+ | [git-scm.com](https://git-scm.com/) |

#### Optional Software

| Software | Purpose | Installation |
|----------|---------|--------------|
| **cargo-watch** | Auto-rebuild on changes | `cargo install cargo-watch` |
| **cargo-tarpaulin** | Code coverage | `cargo install cargo-tarpaulin` |
| **pnpm** | Faster package manager | `npm install -g pnpm` |

### Installation

#### Step 1: Install Rust

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Source the environment
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version

# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Install wasm-pack
cargo install wasm-pack
```

#### Step 2: Install Node.js

```bash
# Using nvm (recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 18
nvm use 18

# Or download from nodejs.org
# https://nodejs.org/en/download/
```

#### Step 3: Clone and Setup

```bash
# Clone repository
git clone <repository-url>
cd vtp-node

# Install dependencies
npm install

# Build WebAssembly
npm run wasm:build

# Verify setup
npm run check
```

### IDE Setup

#### VS Code (Recommended)

Install these extensions:

1. **rust-analyzer**: Rust language support
2. **Svelte for VS Code**: Svelte support
3. **ESLint**: JavaScript/TypeScript linting
4. **Prettier**: Code formatting
5. **EditorConfig**: Editor configuration

**VS Code Settings** (`.vscode/settings.json`):

```json
{
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.cargo.target": "wasm32-unknown-unknown",
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "[svelte]": {
    "editor.defaultFormatter": "svelte.svelte-vscode"
  }
}
```

#### WebStorm/IntelliJ

1. Install Rust plugin
2. Install Svelte plugin
3. Configure TypeScript support

---

## Workflow

### Development Server

Start the development server with hot reload:

```bash
npm run dev
```

The server will start at `http://localhost:5173`.

**Features:**
- Hot Module Replacement (HMR)
- Error overlay
- Source maps
- Auto-reload on changes

### Building

#### Build WebAssembly

```bash
# Build for development
npm run wasm:build

# Build for production
cd src/lib/vtp-core
wasm-pack build --target web --out-dir ../../../static/wasm --release
```

#### Build Frontend

```bash
# Development build
npm run build

# Production build
NODE_ENV=production npm run build
```

#### Build Worker

```bash
npm run worker:build
```

### Hot Reload

#### Rust Changes

When you modify Rust code, you need to rebuild the WebAssembly:

```bash
# Option 1: Manual rebuild
npm run wasm:build

# Option 2: Watch mode (requires cargo-watch)
cd src/lib/vtp-core
cargo watch -w src -s 'wasm-pack build --target web --out-dir ../../../static/wasm'
```

#### TypeScript/Svelte Changes

Changes to TypeScript and Svelte files are automatically detected by Vite.

### Git Workflow

#### Branch Strategy

```
main
├── develop
│   ├── feature/xxx
│   ├── bugfix/xxx
│   └── refactor/xxx
└── release/v1.0.0
```

#### Commit Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance

**Examples:**
```
feat(vdf): add SIMD optimization for SHA256
fix(worker): handle memory limit exceeded
docs(api): update VRF module documentation
test(session): add unit tests for checkpoint recovery
```

#### Pull Request Process

1. **Create branch**: `git checkout -b feature/amazing-feature`
2. **Make changes**: Implement your feature
3. **Run tests**: `npm test && npm run wasm:test`
4. **Run linter**: `npm run lint`
5. **Commit**: `git commit -m 'feat: add amazing feature'`
6. **Push**: `git push origin feature/amazing-feature`
7. **Create PR**: Open a pull request
8. **Code review**: Address feedback
9. **Merge**: Merge after approval

---

## Testing

### Rust Tests

#### Run All Tests

```bash
npm run wasm:test
```

#### Run Specific Test

```bash
cd src/lib/vtp-core
cargo test test_vdf_step
```

#### Run with Output

```bash
cd src/lib/vtp-core
cargo test -- --nocapture
```

#### Test Coverage

```bash
cd src/lib/vtp-core
cargo tarpaulin --out Html
```

### Frontend Tests

#### Run All Tests

```bash
npm test
```

#### Run with UI

```bash
npm run test:ui
```

#### Run Specific Test

```bash
npm test -- --grep "Dashboard"
```

#### Coverage Report

```bash
npm run test:coverage
```

### Performance Tests

#### Rust Benchmarks

```bash
cd src/lib/vtp-core
cargo bench
```

#### Frontend Performance

```bash
npm run test:performance
```

### Browser Testing

#### Manual Testing Checklist

- [ ] Application loads without errors
- [ ] Start/Pause/Resume buttons work
- [ ] VDF progress updates in real-time
- [ ] Statistics display correctly
- [ ] Event log shows events
- [ ] PWA install prompt appears
- [ ] Background execution works
- [ ] Checkpoint saves and restores

#### Automated Browser Tests

```bash
# Install Playwright
npm install -D @playwright/test

# Run tests
npx playwright test
```

---

## Code Style

### Rust Style

#### Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

#### Linting

```bash
# Run clippy
cargo clippy

# Run clippy with all targets
cargo clippy --all-targets --all-features
```

#### Conventions

```rust
// Use snake_case for functions and variables
fn calculate_vdf_step(state: &[u8; 32]) -> [u8; 32] {
    // Implementation
}

// Use PascalCase for types and structs
struct VdfIterator {
    state: [u8; 32],
    step: u64,
}

// Use SCREAMING_SNAKE_CASE for constants
const MAX_BATCH_SIZE: u64 = 1000;

// Document public items
/// Execute a single VDF step
pub fn vdf_step(state: &[u8; 32]) -> [u8; 32] {
    // Implementation
}
```

### TypeScript Style

#### Formatting

```bash
# Format code
npm run format

# Check formatting
npm run format:check
```

#### Linting

```bash
# Run ESLint
npm run lint

# Fix auto-fixable issues
npm run lint:fix
```

#### Conventions

```typescript
// Use camelCase for functions and variables
function calculateProgress(current: number, total: number): number {
  return current / total;
}

// Use PascalCase for types and interfaces
interface WorkerState {
  isRunning: boolean;
  currentStep: number;
}

// Use UPPER_SNAKE_CASE for constants
const MAX_BATCH_SIZE = 1000;

// Use JSDoc for documentation
/**
 * Calculate VDF progress percentage
 * @param current - Current step
 * @param total - Total steps
 * @returns Progress percentage (0-1)
 */
function getProgress(current: number, total: number): number {
  return current / total;
}
```

### Svelte Style

#### Component Structure

```svelte
<script lang="ts">
  // 1. Imports
  import { onMount } from 'svelte';
  
  // 2. Props
  export let title: string;
  
  // 3. State
  let count = 0;
  
  // 4. Reactive declarations
  $: doubled = count * 2;
  
  // 5. Lifecycle
  onMount(() => {
    console.log('Component mounted');
  });
  
  // 6. Functions
  function increment() {
    count += 1;
  }
</script>

<!-- Template -->
<div>
  <h1>{title}</h1>
  <p>Count: {count}</p>
  <button on:click={increment}>Increment</button>
</div>

<style>
  /* Styles */
  div {
    padding: 1rem;
  }
</style>
```

### Commit Convention

#### Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

#### Types

| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation |
| `style` | Formatting (no code change) |
| `refactor` | Code refactoring |
| `perf` | Performance improvement |
| `test` | Adding tests |
| `chore` | Maintenance |
| `ci` | CI/CD changes |

#### Examples

```bash
# Feature
git commit -m "feat(vdf): add batch processing optimization"

# Bug fix
git commit -m "fix(worker): handle memory limit gracefully"

# Documentation
git commit -m "docs(api): update VRF module documentation"

# Refactor
git commit -m "refactor(session): extract checkpoint logic"

# Test
git commit -m "test(vdf): add unit tests for edge cases"
```

---

## Debugging

### Rust Debugging

#### Console Logging

```rust
use web_sys::console;

console::log_1(&"Hello from Rust!".into());

// Log with formatting
console::log_1(&format!("Step: {}", step).into());
```

#### Error Handling

```rust
// Use console_error_panic_hook
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

// Use Result types
pub fn risky_operation() -> Result<String, JsValue> {
    // Implementation
}
```

#### Debug Mode

```bash
# Build with debug info
cd src/lib/vtp-core
wasm-pack build --target web --out-dir ../../../static/wasm --dev
```

### Frontend Debugging

#### Browser DevTools

1. **Console**: Check for errors and logs
2. **Network**: Monitor API calls and assets
3. **Performance**: Profile application
4. **Memory**: Detect memory leaks
5. **Application**: Inspect storage

#### Svelte DevTools

Install the Svelte DevTools browser extension:
- [Chrome](https://chrome.google.com/webstore/detail/svelte-devtools/)
- [Firefox](https://addons.mozilla.org/en-US/firefox/addon/svelte-devtools/)

#### Source Maps

Source maps are enabled by default in development:

```typescript
// vite.config.ts
export default defineConfig({
  build: {
    sourcemap: true
  }
});
```

### Worker Debugging

#### Console Logging

```typescript
// In worker
console.log('Worker started');
console.log('Step:', step);

// Log errors
console.error('Error:', error);
```

#### Chrome DevTools

1. Open DevTools
2. Go to **Sources** tab
3. Expand **Threads** section
4. Click on the worker thread
5. Set breakpoints and debug

#### Error Handling

```typescript
// Global error handler
self.onerror = (error) => {
  console.error('Worker error:', error);
};

// Message error handler
self.onmessageerror = (error) => {
  console.error('Message error:', error);
};
```

### Performance Profiling

#### Chrome Performance Tab

1. Open DevTools (F12)
2. Go to **Performance** tab
3. Click **Record**
4. Perform actions
5. Click **Stop**
6. Analyze results

#### Memory Profiling

1. Open DevTools
2. Go to **Memory** tab
3. Take **Heap Snapshot**
4. Perform actions
5. Take another snapshot
6. Compare snapshots

#### Rust Profiling

```bash
# Build with profiling
cd src/lib/vtp-core
RUSTFLAGS='-C profile-generate=/tmp/profile' \
  wasm-pack build --target web --out-dir ../../../static/wasm

# Analyze with perf (Linux)
perf report
```

---

## Common Tasks

### Adding a New Rust Function

1. **Add function to appropriate module**

```rust
// src/lib/vtp-core/src/vdf.rs
pub fn new_function(input: &[u8]) -> Vec<u8> {
    // Implementation
}
```

2. **Add Wasm binding**

```rust
// src/lib/vtp-core/src/lib.rs
#[wasm_bindgen]
pub fn new_function(input: &[u8]) -> Vec<u8> {
    vdf::new_function(input)
}
```

3. **Add tests**

```rust
// src/lib/vtp-core/tests/vdf_test.rs
#[test]
fn test_new_function() {
    let input = [0u8; 32];
    let result = new_function(&input);
    assert_eq!(result.len(), 32);
}
```

4. **Rebuild Wasm**

```bash
npm run wasm:build
```

### Adding a New Svelte Component

1. **Create component file**

```svelte
<!-- src/components/NewComponent.svelte -->
<script lang="ts">
  export let title: string;
</script>

<div class="new-component">
  <h2>{title}</h2>
  <slot />
</div>

<style>
  .new-component {
    padding: 1rem;
  }
</style>
```

2. **Import and use**

```svelte
<!-- src/routes/+page.svelte -->
<script>
  import NewComponent from '$components/NewComponent.svelte';
</script>

<NewComponent title="Hello">
  <p>Content</p>
</NewComponent>
```

### Adding a New Store

1. **Create store**

```typescript
// src/stores/newStore.ts
import { writable } from 'svelte/store';

interface NewState {
  value: number;
}

export const newStore = writable<NewState>({
  value: 0
});

export function updateValue(newValue: number) {
  newStore.update(state => ({
    ...state,
    value: newValue
  }));
}
```

2. **Use in component**

```svelte
<script>
  import { newStore, updateValue } from '$stores/newStore';
</script>

<p>Value: {$newStore.value}</p>
<button on:click={() => updateValue(42)}>Update</button>
```

---

## Troubleshooting

### Common Issues

#### Wasm Build Fails

**Error**: `wasm-pack build` fails

**Solution**:
```bash
# Clean and rebuild
cd src/lib/vtp-core
cargo clean
wasm-pack build --target web --out-dir ../../../static/wasm
```

#### TypeScript Errors

**Error**: Type errors after Wasm rebuild

**Solution**:
```bash
# Regenerate types
npm run wasm:build
npm run check
```

#### Worker Not Loading

**Error**: Worker fails to load

**Solution**:
```bash
# Rebuild worker
npm run worker:build

# Check browser console for errors
```

#### Memory Leaks

**Error**: Memory usage grows over time

**Solution**:
1. Check for unclosed resources
2. Verify event listeners are removed
3. Use Chrome Memory profiler
4. Check for circular references

#### Performance Issues

**Error**: Application is slow

**Solution**:
1. Check batch size settings
2. Verify time-slice configuration
3. Profile with Chrome DevTools
4. Check for unnecessary re-renders

### Getting Help

- **GitHub Issues**: Report bugs
- **GitHub Discussions**: Ask questions
- **Discord**: Join community
- **Email**: dev@vtp-node.dev

---

## Support

For development questions:
- **GitHub Discussions**: [Join the discussion](https://github.com/your-org/vtp-node/discussions)
- **Email**: dev@vtp-node.dev

---

<div align="center">

**[⬆ Back to Top](#vtp-node-development-guide)**

</div>
