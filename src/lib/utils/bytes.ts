/**
 * Shared Byte Utilities
 *
 * Common byte manipulation functions used across the consensus and network
 * modules. Extracted to eliminate duplication and provide a single source
 * of truth for these operations.
 */

/**
 * Compare two Uint8Arrays lexicographically.
 *
 * @returns Negative if a < b, 0 if equal, positive if a > b.
 */
export function compareBytes(a: Uint8Array, b: Uint8Array): number {
  const len = Math.min(a.length, b.length);
  for (let i = 0; i < len; i++) {
    if (a[i] !== b[i]) return a[i] - b[i];
  }
  return a.length - b.length;
}

/**
 * Compare two Uint8Arrays for byte-level equality.
 */
export function bytesEqual(a: Uint8Array, b: Uint8Array): boolean {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return false;
  }
  return true;
}

/**
 * Convert a Uint8Array to a lowercase hex string.
 */
export function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}
