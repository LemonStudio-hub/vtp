/**
 * Worker State Management Module
 *
 * Manages Web Worker state and events using Svelte stores.
 * Provides global state management that components can subscribe to and update.
 *
 * Key features:
 * 1. Worker instance management
 * 2. Computation state tracking
 * 3. Event log management
 * 4. Progress calculation
 *
 * @example
 * ```typescript
 * import { workerState, addEvent } from '$stores/worker';
 *
 * // Subscribe to state changes
 * workerState.subscribe(state => {
 *   console.log('Current step:', state.currentStep);
 * });
 *
 * // Add an event
 * addEvent({ type: 'info', message: 'Computation started' });
 * ```
 */

import { writable, derived } from 'svelte/store';

/**
 * VTP Event interface.
 *
 * Represents an event that occurs during a VDF computation session.
 */
export interface VtpEvent {
  /** The event type. */
  type: 'info' | 'checkpoint' | 'winner' | 'error';

  /** Event timestamp (Unix epoch milliseconds). */
  timestamp: number;

  /** A human-readable description of the event. */
  message: string;
}

/**
 * Worker State interface.
 *
 * Represents the current state of the Web Worker, including its
 * runtime status, computation progress, and performance metrics.
 */
export interface WorkerState {
  /** Whether the worker is currently running a computation. */
  isRunning: boolean;

  /** Whether the worker has been paused. */
  isPaused: boolean;

  /** The number of VDF steps completed so far. */
  currentStep: number;

  /** The total target number of VDF steps to compute. */
  totalSteps: number;

  /** The current computation speed (steps per second). */
  speed: number;

  /** The elapsed uptime of the worker (in seconds). */
  uptime: number;

  /** The total number of winning draws (VRF match events). */
  winnerCount: number;

  /** The luck index as a percentage. */
  luckPercentage: number;

  /** The node's public key, or `null` if not yet initialized. */
  publicKey: Uint8Array | null;

  /** The unique node identifier. */
  nodeId: string;
}

/**
 * Worker Instance Store.
 *
 * Holds a reference to the current {@link Worker} instance.
 * Used to send control commands (start, pause, resume, stop) to the worker.
 */
export const workerStore = writable<Worker | null>(null);

/**
 * Event Log Store.
 *
 * Stores the list of events generated during VDF computation.
 * Automatically capped at a maximum of 50 entries; older events are removed.
 */
export const events = writable<VtpEvent[]>([]);

/**
 * Worker State Store.
 *
 * Stores the current state of the worker, including:
 * - Runtime status (running / paused)
 * - Computation progress (current step / total steps)
 * - Performance metrics (speed, uptime)
 * - Node information (public key, node ID)
 */
export const workerState = writable<WorkerState>({
  isRunning: false,
  isPaused: false,
  currentStep: 0,
  totalSteps: 0,
  speed: 0,
  uptime: 0,
  winnerCount: 0,
  luckPercentage: 100,
  publicKey: null,
  nodeId: '---'
});

/**
 * Computation Progress Store (derived).
 *
 * A derived store that computes the current progress as a value in the
 * range `[0, 1]`, based on the ratio of `currentStep` to `totalSteps`
 * in {@link workerState}. Suitable for driving progress bars and progress rings.
 *
 * @example
 * ```typescript
 * import { progress } from '$stores/worker';
 *
 * progress.subscribe(value => {
 *   console.log('Progress:', (value * 100).toFixed(1) + '%');
 * });
 * ```
 */
export const progress = derived(workerState, ($state) => {
  if ($state.totalSteps === 0) return 0;
  return $state.currentStep / $state.totalSteps;
});

/**
 * Add an event to the event log.
 *
 * Prepends a new event to the event list and automatically trims the
 * list to maintain a maximum of 50 entries.
 *
 * @param event - The event to add (the `timestamp` field is omitted and will be auto-populated).
 *
 * @example
 * ```typescript
 * addEvent({ type: 'info', message: 'Computation started' });
 * addEvent({ type: 'winner', message: '🎉 Winner at step 12345' });
 * ```
 *
 * @note
 * - The `timestamp` is automatically set to the current time
 * - The event list is capped at a maximum of 50 entries
 * - New events are prepended to the beginning of the list
 */
export function addEvent(event: Omit<VtpEvent, 'timestamp'>) {
  events.update((current) => [{ ...event, timestamp: Date.now() }, ...current.slice(0, 49)]);
}

/**
 * Reset the Worker state to its initial values.
 *
 * Resets all state fields to their defaults. Commonly used when:
 * - Restarting a computation
 * - Clearing an error state
 * - Initializing the application
 *
 * @example
 * ```typescript
 * resetWorkerState();
 * ```
 *
 * @note
 * - This clears all event logs
 * - This resets all computation state (step count, speed, etc.)
 * - This does **not** terminate the current Worker instance
 */
export function resetWorkerState() {
  workerState.set({
    isRunning: false,
    isPaused: false,
    currentStep: 0,
    totalSteps: 0,
    speed: 0,
    uptime: 0,
    winnerCount: 0,
    luckPercentage: 100,
    publicKey: null,
    nodeId: '---'
  });
  events.set([]);
}
