/**
 * Background Keep-Alive System
 *
 * A composable system for maintaining browser tab activity and computation
 * continuity. Combines multiple strategies following 2026 best practices:
 *
 * Architecture:
 * ┌─────────────────────────────────────────────────────┐
 * │                  Main Thread                         │
 * │  ┌──────────┐  ┌──────────────┐  ┌───────────────┐ │
 * │  │Visibility │  │AudioKeepAlive│  │  StatePersist │ │
 * │  │ Manager   │  │              │  │   (IndexedDB) │ │
 * │  └────┬─────┘  └──────┬───────┘  └───────┬───────┘ │
 * │       │               │                   │         │
 * │  ┌────▼───────────────▼───────────────────▼───────┐ │
 * │  │              Watchdog (heartbeat monitor)       │ │
 * │  └────────────────────┬────────────────────────────┘ │
 * │                       │ postMessage                   │
 * │  ┌────────────────────▼────────────────────────────┐ │
 * │  │           Web Worker (VDF computation)           │ │
 * │  │  ┌─────────────────────────────────────────┐    │ │
 * │  │  │ Adaptive Heartbeat + Sleep Detection     │    │ │
 * │  │  └─────────────────────────────────────────┘    │ │
 * │  └─────────────────────────────────────────────────┘ │
 * └─────────────────────────────────────────────────────┘
 *
 * Strategy layering (progressive enhancement):
 * 1. Web Worker: Runs in separate thread, NOT throttled by background tab policies
 * 2. AudioContext: Silent oscillator prevents aggressive timer throttling
 * 3. Visibility API: Detects tab state for adaptive behavior
 * 4. IndexedDB: Persists state for recovery after tab discard/sleep
 * 5. Watchdog: Monitors worker health and triggers auto-restart
 *
 * @module background
 */

export { VisibilityManager } from './visibility';
export type {
  VisibilityState,
  VisibilityChangeEvent,
  WakeEvent,
  VisibilityManagerOptions
} from './visibility';

export { AudioKeepAlive } from './audio-keepalive';
export type { AudioKeepAliveOptions } from './audio-keepalive';

export { WorkerWatchdog } from './watchdog';
export type { WatchdogOptions } from './watchdog';

export { StatePersistence } from './persistence';
export type { ComputationSnapshot } from './persistence';
