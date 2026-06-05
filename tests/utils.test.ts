/**
 * @module utils.test
 *
 * Test suite for the utility functions defined in `src/utils/index`.
 *
 * This file covers the following utility modules:
 * - formatBytes: Converts raw byte counts into human-readable strings (B, KB, MB, GB)
 * - formatNumber: Formats numbers with locale-aware thousand separators and decimals
 * - formatSpeed: Formats numeric speed values with K/M suffixes for compact display
 * - formatTime: Converts a number of seconds into HH:MM:SS formatted time strings
 * - generateNodeId: Produces unique 8-character hexadecimal node identifiers
 * - sleep: Async utility that pauses execution for a specified duration
 * - debounce: Higher-order function that delays invocation until after a quiet period
 */

import { describe, it, expect, vi } from 'vitest';
import {
  formatBytes,
  formatNumber,
  formatSpeed,
  formatTime,
  generateNodeId,
  sleep,
  debounce
} from '../src/utils/index';

/**
 * Tests for the `formatBytes` utility function.
 *
 * This function converts a raw byte count into a human-readable string
 * using binary-based units (B, KB, MB, GB). It handles zero values,
 * fractional conversions, and rounding behavior.
 */
describe('formatBytes', () => {
  /** Verifies that zero bytes are formatted as the string "0 B". */
  it('formats zero bytes', () => {
    expect(formatBytes(0)).toBe('0 B');
  });

  /** Verifies that values less than 1024 are displayed in bytes (B). */
  it('formats bytes', () => {
    expect(formatBytes(512)).toBe('512 B');
  });

  /**
   * Verifies that values in the kilobyte range (1024–1048575) are
   * correctly converted to KB with appropriate rounding.
   * - 1024 bytes = exactly 1 KB
   * - 1536 bytes = 1.5 KB
   */
  it('formats kilobytes', () => {
    expect(formatBytes(1024)).toBe('1 KB');
    expect(formatBytes(1536)).toBe('1.5 KB');
  });

  /**
   * Verifies that values in the megabyte range (1048576–1073741823) are
   * correctly converted to MB.
   * - 1048576 bytes = exactly 1 MB
   * - 5242880 bytes = exactly 5 MB
   */
  it('formats megabytes', () => {
    expect(formatBytes(1048576)).toBe('1 MB');
    expect(formatBytes(5242880)).toBe('5 MB');
  });

  /** Verifies that gigabyte-range values (≥ 1073741824) are correctly converted to GB. */
  it('formats gigabytes', () => {
    expect(formatBytes(1073741824)).toBe('1 GB');
  });

  /**
   * Verifies correct rounding behavior for fractional byte values.
   * - 1025 bytes is just over 1 KB, rounds to "1 KB"
   * - 2048 bytes is exactly 2 KB
   */
  it('formats fractional values correctly', () => {
    expect(formatBytes(1025)).toBe('1 KB');
    expect(formatBytes(2048)).toBe('2 KB');
  });
});

/**
 * Tests for the `formatNumber` utility function.
 *
 * This function formats numeric values into locale-aware strings with
 * thousand separators and optional decimal places.
 */
describe('formatNumber', () => {
  /** Verifies that zero is formatted as the string "0". */
  it('formats zero', () => {
    expect(formatNumber(0)).toBe('0');
  });

  /** Verifies that small numbers (below 1000) remain unformatted. */
  it('formats small numbers', () => {
    expect(formatNumber(42)).toBe('42');
  });

  /**
   * Verifies that numbers in the millions are formatted with thousand separators.
   * The result of 1234567 should contain "1", "234", and "567" as grouped segments.
   */
  it('formats thousands', () => {
    const result = formatNumber(1234567);
    expect(result).toContain('1');
    expect(result).toContain('234');
    expect(result).toContain('567');
  });

  /**
   * Verifies that decimal numbers preserve their fractional part
   * and have thousand separators applied to the integer portion.
   * For 1234.567, the result should contain "1" and "234".
   */
  it('formats decimals', () => {
    const result = formatNumber(1234.567);
    expect(result).toContain('1');
    expect(result).toContain('234');
  });
});

/**
 * Tests for the `formatSpeed` utility function.
 *
 * This function formats speed values with compact suffixes:
 * - Values below 1000 are displayed as-is
 * - Values from 1000 to 999999 are displayed with a "K" suffix (e.g., "1.5K")
 * - Values from 1000000 and above are displayed with an "M" suffix (e.g., "2.5M")
 */
describe('formatSpeed', () => {
  /**
   * Verifies that values below 1000 are returned as plain number strings
   * without any suffix, including edge cases like 0 and 999.
   */
  it('formats values below 1000', () => {
    expect(formatSpeed(500)).toBe('500');
    expect(formatSpeed(0)).toBe('0');
    expect(formatSpeed(999)).toBe('999');
  });

  /**
   * Verifies that values in the thousands range are formatted with the "K" suffix
   * and one decimal place.
   * - 1000 → "1.0K"
   * - 1500 → "1.5K"
   * - 999999 → "1000.0K"
   */
  it('formats thousands', () => {
    expect(formatSpeed(1000)).toBe('1.0K');
    expect(formatSpeed(1500)).toBe('1.5K');
    expect(formatSpeed(999999)).toBe('1000.0K');
  });

  /**
   * Verifies that values in the millions range are formatted with the "M" suffix
   * and one decimal place.
   * - 1000000 → "1.0M"
   * - 1500000 → "1.5M"
   * - 2500000 → "2.5M"
   */
  it('formats millions', () => {
    expect(formatSpeed(1000000)).toBe('1.0M');
    expect(formatSpeed(1500000)).toBe('1.5M');
    expect(formatSpeed(2500000)).toBe('2.5M');
  });

  /**
   * Verifies the exact boundary transitions between unit ranges:
   * - 999 is still plain (no suffix)
   * - 1000 transitions to "K" suffix
   * - 999999 is the largest "K" value
   * - 1000000 transitions to "M" suffix
   */
  it('formats boundary values', () => {
    expect(formatSpeed(999)).toBe('999');
    expect(formatSpeed(1000)).toBe('1.0K');
    expect(formatSpeed(999999)).toBe('1000.0K');
    expect(formatSpeed(1000000)).toBe('1.0M');
  });
});

/**
 * Tests for the `formatTime` utility function.
 *
 * This function converts a number of seconds into an HH:MM:SS formatted
 * time string. It handles zero padding, minute/hour roll-overs, and
 * values exceeding 24 hours.
 */
describe('formatTime', () => {
  /** Verifies that zero seconds produces the "00:00:00" format. */
  it('formats zero seconds', () => {
    expect(formatTime(0)).toBe('00:00:00');
  });

  /**
   * Verifies that values under 60 seconds are shown as seconds only
   * with proper zero-padding.
   * - 30 seconds → "00:00:30"
   * - 59 seconds → "00:00:59"
   */
  it('formats seconds only', () => {
    expect(formatTime(30)).toBe('00:00:30');
    expect(formatTime(59)).toBe('00:00:59');
  });

  /**
   * Verifies minute boundaries and combined minute-second formatting.
   * - 60 seconds (exactly 1 minute) → "00:01:00"
   * - 61 seconds → "00:01:01"
   * - 90 seconds (1 min 30 sec) → "00:01:30"
   */
  it('formats minutes and seconds', () => {
    expect(formatTime(60)).toBe('00:01:00');
    expect(formatTime(61)).toBe('00:01:01');
    expect(formatTime(90)).toBe('00:01:30');
  });

  /**
   * Verifies hour boundaries and combined hour-minute-second formatting.
   * - 3600 seconds (exactly 1 hour) → "01:00:00"
   * - 3661 seconds (1 hour, 1 minute, 1 second) → "01:01:01"
   */
  it('formats hours', () => {
    expect(formatTime(3600)).toBe('01:00:00');
    expect(formatTime(3661)).toBe('01:01:01');
  });

  /**
   * Verifies that values exceeding 24 hours are correctly formatted
   * without day-level truncation.
   * - 86400 seconds (24 hours) → "24:00:00"
   * - 100000 seconds (27 hours, 46 minutes, 40 seconds) → "27:46:40"
   */
  it('formats large values', () => {
    expect(formatTime(86400)).toBe('24:00:00');
    expect(formatTime(100000)).toBe('27:46:40');
  });

  /**
   * Verifies that single-digit seconds values are properly zero-padded
   * to maintain consistent HH:MM:SS width.
   * - 1 second → "00:00:01"
   * - 10 seconds → "00:00:10"
   */
  it('pads single digits', () => {
    expect(formatTime(1)).toBe('00:00:01');
    expect(formatTime(10)).toBe('00:00:10');
  });
});

/**
 * Tests for the `generateNodeId` utility function.
 *
 * This function generates a unique node identifier string used to
 * distinguish different worker nodes. The ID is an 8-character lowercase
 * hexadecimal string.
 */
describe('generateNodeId', () => {
  /** Verifies that the generated node ID is exactly 8 characters long. */
  it('returns 8 character string', () => {
    expect(generateNodeId()).toHaveLength(8);
  });

  /** Verifies that the node ID contains only valid lowercase hexadecimal characters (0-9, a-f). */
  it('contains only hex characters', () => {
    const id = generateNodeId();
    expect(id).toMatch(/^[0-9a-f]{8}$/);
  });

  /**
   * Verifies that consecutive calls to generateNodeId produce distinct IDs,
   * confirming uniqueness (or at least very high entropy).
   */
  it('generates different IDs', () => {
    const id1 = generateNodeId();
    const id2 = generateNodeId();
    expect(id1).not.toBe(id2);
  });
});

/**
 * Tests for the `sleep` async utility function.
 *
 * This function returns a Promise that resolves after a specified number
 * of milliseconds, useful for introducing delays in async workflows.
 */
describe('sleep', () => {
  /**
   * Verifies that sleep actually pauses execution for approximately the
   * specified duration. A 50ms sleep should result in at least 40ms of
   * elapsed wall-clock time (allowing for minor timer inaccuracy).
   */
  it('resolves after specified time', async () => {
    const start = Date.now();
    await sleep(50);
    const elapsed = Date.now() - start;
    expect(elapsed).toBeGreaterThanOrEqual(40);
  });

  /**
   * Verifies that sleep resolves with undefined (void), confirming
   * it does not return any meaningful value.
   */
  it('resolves with void', async () => {
    const result = await sleep(1);
    expect(result).toBeUndefined();
  });
});

/**
 * Tests for the `debounce` higher-order utility function.
 *
 * Debounce wraps a function so that it is only invoked after a specified
 * quiet period (wait time in ms) has elapsed since the last call. This is
 * commonly used to rate-limit event handlers (e.g., resize, input).
 *
 * These tests use Vitest's fake timers (`vi.useFakeTimers` / `vi.advanceTimersByTime`)
 * to deterministically control time progression.
 */
describe('debounce', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  /**
   * Verifies that calling the debounced function does not immediately invoke
   * the underlying function, and that it is invoked exactly once after the
   * specified delay (100ms) has elapsed.
   */
  it('delays function execution', () => {
    const fn = vi.fn();
    const debounced = debounce(fn, 100);

    debounced();
    expect(fn).not.toHaveBeenCalled();

    vi.advanceTimersByTime(100);
    expect(fn).toHaveBeenCalledOnce();
  });

  /**
   * Verifies that rapid successive calls within the debounce window
   * reset the timer, resulting in only a single invocation of the
   * underlying function after the final call's delay expires.
   */
  it('cancels previous call on rapid invocations', () => {
    const fn = vi.fn();
    const debounced = debounce(fn, 100);

    debounced();
    debounced();
    debounced();

    vi.advanceTimersByTime(100);
    expect(fn).toHaveBeenCalledOnce();
  });

  /**
   * Verifies that the arguments passed to the debounced function are
   * correctly forwarded to the underlying function when it is finally invoked.
   */
  it('passes arguments to the debounced function', () => {
    const fn = vi.fn();
    const debounced = debounce(fn, 100);

    debounced('arg1', 'arg2');
    vi.advanceTimersByTime(100);

    expect(fn).toHaveBeenCalledWith('arg1', 'arg2');
  });

  /**
   * Verifies that when multiple rapid calls are made with different arguments,
   * only the arguments from the LAST call are forwarded to the underlying function.
   * This is the expected "trailing edge" debounce behavior.
   */
  it('uses last arguments on rapid calls', () => {
    const fn = vi.fn();
    const debounced = debounce(fn, 100);

    debounced('first');
    debounced('second');
    debounced('third');

    vi.advanceTimersByTime(100);
    expect(fn).toHaveBeenCalledWith('third');
  });

  /**
   * Verifies that the debounced function can be invoked multiple times
   * sequentially as long as each call is separated by at least the full
   * debounce wait period. Each call should result in a separate invocation.
   */
  it('allows multiple executions after wait period', () => {
    const fn = vi.fn();
    const debounced = debounce(fn, 100);

    debounced('first');
    vi.advanceTimersByTime(100);

    debounced('second');
    vi.advanceTimersByTime(100);

    expect(fn).toHaveBeenCalledTimes(2);
  });
});
