/**
 * Utility Functions Module
 *
 * Provides commonly used utility functions, including:
 * - Number formatting
 * - Byte formatting
 * - Time formatting
 * - ID generation
 * - Async utilities
 * - Function debouncing
 *
 * @example
 * ```typescript
 * import { formatBytes, formatTime, generateNodeId } from '$utils';
 *
 * console.log(formatBytes(1024)); // "1 KB"
 * console.log(formatTime(3661)); // "01:01:01"
 * console.log(generateNodeId()); // "a1b2c3d4"
 * ```
 */

/**
 * Format a byte count into a human-readable string.
 *
 * Converts a byte count to a human-readable format, automatically selecting
 * the most appropriate unit (B, KB, MB, or GB).
 *
 * @param bytes - The number of bytes to format.
 * @returns A formatted string with the appropriate unit suffix.
 *
 * @example
 * ```typescript
 * formatBytes(0) // "0 B"
 * formatBytes(1024) // "1 KB"
 * formatBytes(1048576) // "1 MB"
 * formatBytes(1073741824) // "1 GB"
 * ```
 *
 * @note
 * - Uses 1024 as the base (binary / IEC standard)
 * - Maximum unit is GB
 * - Fractional values are rounded to two decimal places
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

/**
 * Format a number using locale-specific thousands separators.
 *
 * Converts a number to a locale-formatted string with appropriate
 * grouping separators (e.g., commas in English locales).
 *
 * @param num - The number to format.
 * @returns A locale-formatted string representation of the number.
 *
 * @example
 * ```typescript
 * formatNumber(1234567) // "1,234,567"
 * formatNumber(1234.567) // "1,234.567"
 * ```
 *
 * @note
 * - Uses the runtime environment's locale settings
 * - Preserves the original precision of the number
 */
export function formatNumber(num: number): string {
  return num.toLocaleString();
}

/**
 * Format a speed value into a human-readable string.
 *
 * Converts a speed value (in steps per second) to a compact, human-readable
 * format, automatically selecting the most appropriate unit suffix (M, K, or raw).
 *
 * @param speed - The speed value in steps per second.
 * @returns A formatted string with the appropriate unit suffix.
 *
 * @example
 * ```typescript
 * formatSpeed(1500000) // "1.5M"
 * formatSpeed(1500) // "1.5K"
 * formatSpeed(500) // "500"
 * ```
 *
 * @note
 * - Values >= 1,000,000 use "M" (millions)
 * - Values >= 1,000 use "K" (thousands)
 * - All other values are displayed as-is
 * - Fractional values are rounded to one decimal place
 */
export function formatSpeed(speed: number): string {
  if (speed >= 1000000) {
    return `${(speed / 1000000).toFixed(1)}M`;
  } else if (speed >= 1000) {
    return `${(speed / 1000).toFixed(1)}K`;
  }
  return speed.toFixed(0);
}

/**
 * Format a duration in seconds to an HH:MM:SS time string.
 *
 * Converts a total number of seconds into a formatted time string
 * with hours, minutes, and seconds components.
 *
 * @param seconds - The total number of seconds to format.
 * @returns A formatted time string in "HH:MM:SS" format.
 *
 * @example
 * ```typescript
 * formatTime(0) // "00:00:00"
 * formatTime(61) // "00:01:01"
 * formatTime(3661) // "01:01:01"
 * ```
 *
 * @note
 * - Each component is zero-padded to two digits
 * - Supports durations exceeding 24 hours (no day rollover)
 */
export function formatTime(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);
  return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
}

/**
 * Generate a random node ID.
 *
 * Generates an 8-character hexadecimal string suitable for use as a
 * lightweight node identifier.
 *
 * @returns An 8-character lowercase hexadecimal string.
 *
 * @example
 * ```typescript
 * generateNodeId() // "a1b2c3d4"
 * generateNodeId() // "e5f6a7b8"
 * ```
 *
 * @note
 * - Uses {@link Math.random()} internally; **not** cryptographically secure
 * - Generated IDs may collide, though the probability is extremely low
 */
export function generateNodeId(): string {
  const chars = '0123456789abcdef';
  let result = '';
  for (let i = 0; i < 8; i++) {
    result += chars[Math.floor(Math.random() * chars.length)];
  }
  return result;
}

// Re-export sleep from shared module (also importable standalone for Web Worker use)
export { sleep } from '$lib/utils/sleep';

/**
 * Format a timestamp as a relative time string.
 *
 * Returns human-readable relative time like "just now", "2m ago", "1h ago".
 *
 * @param timestamp - Unix timestamp in milliseconds.
 * @returns A relative time string.
 *
 * @example
 * ```typescript
 * formatRelativeTime(Date.now() - 5000)  // "just now"
 * formatRelativeTime(Date.now() - 120000) // "2m ago"
 * formatRelativeTime(Date.now() - 3600000) // "1h ago"
 * ```
 */
export function formatRelativeTime(timestamp: number): string {
  const diff = Date.now() - timestamp;
  const seconds = Math.floor(diff / 1000);

  if (seconds < 60) return 'just now';
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
  if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
  return `${Math.floor(seconds / 86400)}d ago`;
}

/**
 * Calculate estimated time remaining for a computation.
 *
 * @param currentStep - Current progress step.
 * @param totalSteps - Total target steps.
 * @param speed - Current computation speed (steps per second).
 * @returns Estimated seconds remaining, or -1 if speed is 0.
 *
 * @example
 * ```typescript
 * calculateETA(500, 1000, 100) // 5
 * calculateETA(500, 1000, 0)   // -1
 * ```
 */
export function calculateETA(currentStep: number, totalSteps: number, speed: number): number {
  if (speed <= 0 || currentStep >= totalSteps) return -1;
  return Math.ceil((totalSteps - currentStep) / speed);
}

/**
 * Clamp a number between a minimum and maximum value.
 *
 * @param value - The value to clamp.
 * @param min - The minimum bound.
 * @param max - The maximum bound.
 * @returns The clamped value.
 *
 * @example
 * ```typescript
 * clamp(5, 0, 10)   // 5
 * clamp(-1, 0, 10)  // 0
 * clamp(15, 0, 10)  // 10
 * ```
 */
export function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

/**
 * Create a debounced version of a function.
 *
 * Returns a new function that delays invoking the provided function until
 * after the specified wait time has elapsed since the last invocation.
 * Repeated calls within the wait period reset the timer, ensuring only
 * the final call is executed.
 *
 * @typeParam T - The type of the function to debounce.
 * @param func - The function to debounce.
 * @param wait - The debounce delay in milliseconds.
 * @returns A debounced wrapper function with the same parameters as `func`.
 *
 * @example
 * ```typescript
 * const debouncedSearch = debounce((query: string) => {
 *   console.log('Searching:', query);
 * }, 300);
 *
 * // During rapid input, only the last call is executed after 300ms of inactivity
 * debouncedSearch('a');
 * debouncedSearch('ab');
 * debouncedSearch('abc');
 * ```
 *
 * @note
 * - Uses `setTimeout` internally
 * - Each invocation resets the debounce timer
 * - The arguments from the last invocation are preserved and used when the timer fires
 */
export function debounce<T extends (...args: unknown[]) => unknown>(
  func: T,
  wait: number
): (...args: Parameters<T>) => void {
  let timeout: ReturnType<typeof setTimeout>;
  return (...args: Parameters<T>) => {
    clearTimeout(timeout);
    timeout = setTimeout(() => func(...args), wait);
  };
}
