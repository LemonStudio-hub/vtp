/**
 * @module worker-store.test
 *
 * Test suite for the Svelte stores defined in `src/stores/worker`.
 *
 * This file validates the behavior of the reactive state management layer
 * used to coordinate between the main thread and the Web Worker. The stores
 * covered include:
 *
 * - **workerStore**: Holds the Web Worker instance reference (or null before initialization)
 * - **events**: A capped, prepend-only log of worker lifecycle and data events
 * - **workerState**: The primary reactive state object tracking worker execution metrics
 * - **progress**: A derived store computing completion percentage from currentStep / totalSteps
 * - **resetWorkerState**: A convenience function that restores all stores to their initial defaults
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import {
  workerStore,
  events,
  workerState,
  progress,
  addEvent,
  resetWorkerState
} from '../src/stores/worker';

/**
 * Tests for the `workerStore` writable store.
 *
 * This store holds a reference to the instantiated Web Worker. It starts as
 * `null` and is set to a Worker instance once the page mounts and the worker
 * is created.
 */
describe('workerStore', () => {
  /** Verifies that the store is initialized with null before any Worker is created. */
  it('initializes with null', () => {
    expect(get(workerStore)).toBeNull();
  });
});

/**
 * Tests for the `events` writable store.
 *
 * This store maintains an ordered list of worker events (info, checkpoint,
 * winner, error). New events are prepended (most recent first) and the
 * list is capped at 50 entries to prevent unbounded memory growth.
 */
describe('events', () => {
  /** Verifies that the events store starts as an empty array. */
  it('initializes with empty array', () => {
    expect(get(events)).toEqual([]);
  });

  /**
   * Verifies that `addEvent` correctly appends a new event object with
   * the provided type and message, and automatically attaches a numeric
   * `timestamp` field.
   */
  it('stores added events', () => {
    events.set([]);
    addEvent({ type: 'info', message: 'test event' });
    const evts = get(events);
    expect(evts).toHaveLength(1);
    expect(evts[0].type).toBe('info');
    expect(evts[0].message).toBe('test event');
    // Timestamp should be auto-generated as a numeric epoch value
    expect(evts[0].timestamp).toBeTypeOf('number');
  });

  /**
   * Verifies that new events are prepended (placed at index 0), so the
   * most recent event always appears first in the array. This ensures
   * the UI displays the latest events at the top.
   */
  it('prepends new events', () => {
    events.set([]);
    addEvent({ type: 'info', message: 'first' });
    addEvent({ type: 'checkpoint', message: 'second' });

    const evts = get(events);
    expect(evts).toHaveLength(2);
    // 'second' was added last, so it should be at index 0 (prepended)
    expect(evts[0].message).toBe('second');
    expect(evts[1].message).toBe('first');
  });

  /**
   * Verifies that the events store enforces a maximum capacity of 50 entries.
   * When more than 50 events are added, the oldest events are discarded to
   * keep the list at exactly 50 items.
   */
  it('limits events to 50', () => {
    events.set([]);
    // Add 55 events to exceed the cap
    for (let i = 0; i < 55; i++) {
      addEvent({ type: 'info', message: `event ${i}` });
    }
    // Only the most recent 50 should be retained
    expect(get(events)).toHaveLength(50);
  });

  /**
   * Verifies that different event types (info, checkpoint, winner, error)
   * are stored with their correct type values. Since events are prepended,
   * later events appear at lower indices.
   */
  it('preserves event types correctly', () => {
    events.set([]);
    addEvent({ type: 'info', message: 'info' });
    addEvent({ type: 'checkpoint', message: 'checkpoint' });
    addEvent({ type: 'winner', message: 'winner' });
    addEvent({ type: 'error', message: 'error' });

    const evts = get(events);
    // Events are prepended, so the last added (error) is at index 0
    expect(evts[3].type).toBe('info');
    expect(evts[2].type).toBe('checkpoint');
    expect(evts[1].type).toBe('winner');
    expect(evts[0].type).toBe('error');
  });
});

/**
 * Tests for the `workerState` writable store.
 *
 * This store holds the primary reactive state object that tracks the
 * worker's execution metrics including running status, step counts,
 * speed, uptime, and more. Each test resets the state via
 * `resetWorkerState` before running to ensure isolation.
 */
describe('workerState', () => {
  /** Reset worker state before each test to ensure a clean baseline. */
  beforeEach(() => {
    resetWorkerState();
  });

  /**
   * Verifies that the store initializes with the expected default values:
   * - Not running, not paused
   * - Steps and speed at zero
   * - Luck at 100% (no data yet)
   * - No public key assigned
   * - Node ID set to placeholder "---"
   */
  it('initializes with correct defaults', () => {
    const state = get(workerState);
    expect(state.isRunning).toBe(false);
    expect(state.isPaused).toBe(false);
    expect(state.currentStep).toBe(0);
    expect(state.totalSteps).toBe(0);
    expect(state.speed).toBe(0);
    expect(state.uptime).toBe(0);
    expect(state.winnerCount).toBe(0);
    expect(state.luckPercentage).toBe(100);
    expect(state.publicKey).toBeNull();
    expect(state.nodeId).toBe('---');
  });

  /**
   * Verifies that the store supports partial updates via the Svelte
   * `update` method, correctly merging new values with the existing state.
   */
  it('can be updated', () => {
    workerState.update((s) => ({
      ...s,
      isRunning: true,
      currentStep: 1000,
      speed: 5000
    }));

    const state = get(workerState);
    expect(state.isRunning).toBe(true);
    expect(state.currentStep).toBe(1000);
    expect(state.speed).toBe(5000);
  });
});

/**
 * Tests for the `progress` derived store.
 *
 * This store computes a normalized progress value (0 to 1) by dividing
 * `currentStep` by `totalSteps`. It handles edge cases like division
 * by zero and reacts to changes in the underlying `workerState`.
 */
describe('progress', () => {
  /** Reset worker state before each test to ensure a clean baseline. */
  beforeEach(() => {
    resetWorkerState();
  });

  /**
   * Verifies that progress returns 0 when totalSteps is 0, preventing
   * a division-by-zero scenario (NaN is avoided).
   */
  it('returns 0 when totalSteps is 0', () => {
    expect(get(progress)).toBe(0);
  });

  /**
   * Verifies that progress is correctly calculated as currentStep / totalSteps.
   * 500 / 1000 = 0.5 (50%).
   */
  it('calculates correct progress', () => {
    workerState.update((s) => ({
      ...s,
      currentStep: 500,
      totalSteps: 1000
    }));
    expect(get(progress)).toBe(0.5);
  });

  /**
   * Verifies that progress reaches 1.0 (100%) when currentStep equals
   * totalSteps, indicating task completion.
   */
  it('returns 1 when complete', () => {
    workerState.update((s) => ({
      ...s,
      currentStep: 1000,
      totalSteps: 1000
    }));
    expect(get(progress)).toBe(1);
  });

  /**
   * Verifies that the derived progress store reactively updates when
   * the underlying workerState changes, without needing manual recalculation.
   * First update yields 0.25 (250/1000), second yields 0.75 (750/1000).
   */
  it('updates reactively', () => {
    workerState.update((s) => ({ ...s, currentStep: 250, totalSteps: 1000 }));
    expect(get(progress)).toBe(0.25);

    // Update only currentStep; totalSteps remains 1000
    workerState.update((s) => ({ ...s, currentStep: 750 }));
    expect(get(progress)).toBe(0.75);
  });
});

/**
 * Tests for the `resetWorkerState` function.
 *
 * This function resets both `workerState` and `events` back to their
 * initial default values, effectively clearing all accumulated state.
 * It is typically called when the user restarts or re-initializes the worker.
 */
describe('resetWorkerState', () => {
  /**
   * Verifies that `resetWorkerState` fully restores the workerState store
   * to its default values and clears all accumulated events. First mutates
   * the state and adds an event, then resets and confirms everything is cleared.
   */
  it('resets all state to defaults', () => {
    // Mutate state to non-default values
    workerState.update((s) => ({
      ...s,
      isRunning: true,
      currentStep: 5000,
      speed: 1000,
      winnerCount: 3
    }));
    addEvent({ type: 'info', message: 'test' });

    resetWorkerState();

    // Verify all state fields are restored to defaults
    const state = get(workerState);
    expect(state.isRunning).toBe(false);
    expect(state.currentStep).toBe(0);
    expect(state.speed).toBe(0);
    expect(state.winnerCount).toBe(0);
    // Verify the events log is also cleared
    expect(get(events)).toEqual([]);
  });
});
