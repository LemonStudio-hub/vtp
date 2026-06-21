/**
 * Sleep Utility
 *
 * Promise-based delay function. Standalone module with no framework
 * dependencies, importable from both main thread and Web Worker contexts.
 */

/**
 * Pause execution for the given number of milliseconds.
 *
 * @param ms - Duration to sleep in milliseconds.
 * @returns A promise that resolves after the specified delay.
 */
export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
