/**
 * Visibility State Store
 *
 * Reactive Svelte stores for page visibility and background keep-alive state.
 * Components can subscribe to these stores to adapt their behavior based on
 * tab visibility (e.g., reduce canvas animation frame rate when hidden).
 *
 * @module stores/visibility
 */

import { writable, derived } from 'svelte/store';

/**
 * Visibility state store.
 * Updated by the VisibilityManager in +page.svelte.
 */
export const isVisible = writable(true);

/**
 * Whether the page was recently restored from sleep or BFCache.
 * Set to true on wake, automatically resets after 3 seconds.
 */
export const isWaking = writable(false);

/**
 * Whether the AudioContext keep-alive is active.
 */
export const audioKeepAliveActive = writable(false);

/**
 * Worker health status.
 * Updated by the WorkerWatchdog.
 */
export const workerHealthy = writable(true);

/**
 * Number of consecutive missed heartbeats.
 */
export const missedHeartbeats = writable(0);

/**
 * Effective animation frame budget.
 * When visible: 16ms (60fps), when hidden: 1000ms (1fps).
 * Used by VDFCanvas and StatsPanel to throttle rendering.
 */
export const animationBudget = derived(isVisible, ($isVisible) => ($isVisible ? 16 : 1000));

/**
 * Whether the tab is in "reduced activity" mode.
 * True when hidden AND audio keep-alive is not active.
 * In this mode, the UI should minimize resource usage.
 */
export const isReducedActivity = derived(
  [isVisible, audioKeepAliveActive],
  ([$isVisible, $audioActive]) => !$isVisible && !$audioActive
);

/**
 * Signal that a wake event occurred.
 * Automatically resets after 3 seconds.
 */
export function signalWake(): void {
  isWaking.set(true);
  setTimeout(() => isWaking.set(false), 3000);
}
