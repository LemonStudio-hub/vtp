/**
 * Tests for shared byte utility functions.
 */

import { describe, it, expect } from 'vitest';
import { bytesToHex, bytesEqual, compareBytes } from '../src/lib/utils/bytes';

describe('bytesToHex', () => {
  it('converts empty array to empty string', () => {
    expect(bytesToHex(new Uint8Array(0))).toBe('');
  });

  it('converts single byte', () => {
    expect(bytesToHex(new Uint8Array([0]))).toBe('00');
    expect(bytesToHex(new Uint8Array([255]))).toBe('ff');
    expect(bytesToHex(new Uint8Array([10]))).toBe('0a');
  });

  it('converts multi-byte array', () => {
    expect(bytesToHex(new Uint8Array([0xde, 0xad, 0xbe, 0xef]))).toBe('deadbeef');
  });

  it('pads single-digit hex values with leading zero', () => {
    expect(bytesToHex(new Uint8Array([0x01, 0x02, 0x0a]))).toBe('01020a');
  });

  it('round-trips consistently', () => {
    const original = new Uint8Array([1, 2, 3, 4, 5]);
    const hex = bytesToHex(original);
    expect(hex).toBe('0102030405');
  });
});

describe('bytesEqual', () => {
  it('returns true for equal arrays', () => {
    const a = new Uint8Array([1, 2, 3]);
    const b = new Uint8Array([1, 2, 3]);
    expect(bytesEqual(a, b)).toBe(true);
  });

  it('returns false for different content', () => {
    const a = new Uint8Array([1, 2, 3]);
    const b = new Uint8Array([1, 2, 4]);
    expect(bytesEqual(a, b)).toBe(false);
  });

  it('returns false for different lengths', () => {
    const a = new Uint8Array([1, 2, 3]);
    const b = new Uint8Array([1, 2]);
    expect(bytesEqual(a, b)).toBe(false);
  });

  it('returns true for empty arrays', () => {
    expect(bytesEqual(new Uint8Array(0), new Uint8Array(0))).toBe(true);
  });

  it('returns false when one is empty', () => {
    expect(bytesEqual(new Uint8Array([1]), new Uint8Array(0))).toBe(false);
  });

  it('handles 32-byte arrays', () => {
    const a = new Uint8Array(32).fill(0xab);
    const b = new Uint8Array(32).fill(0xab);
    expect(bytesEqual(a, b)).toBe(true);

    b[31] = 0xac;
    expect(bytesEqual(a, b)).toBe(false);
  });
});

describe('compareBytes', () => {
  it('returns 0 for equal arrays', () => {
    const a = new Uint8Array([1, 2, 3]);
    const b = new Uint8Array([1, 2, 3]);
    expect(compareBytes(a, b)).toBe(0);
  });

  it('returns negative when a < b', () => {
    const a = new Uint8Array([1, 2, 3]);
    const b = new Uint8Array([1, 2, 4]);
    expect(compareBytes(a, b)).toBeLessThan(0);
  });

  it('returns positive when a > b', () => {
    const a = new Uint8Array([1, 2, 4]);
    const b = new Uint8Array([1, 2, 3]);
    expect(compareBytes(a, b)).toBeGreaterThan(0);
  });

  it('compares by length when prefixes match', () => {
    const a = new Uint8Array([1, 2]);
    const b = new Uint8Array([1, 2, 3]);
    expect(compareBytes(a, b)).toBeLessThan(0);
  });

  it('handles empty arrays', () => {
    expect(compareBytes(new Uint8Array(0), new Uint8Array(0))).toBe(0);
  });

  it('handles different first bytes', () => {
    const a = new Uint8Array([0]);
    const b = new Uint8Array([255]);
    expect(compareBytes(a, b)).toBeLessThan(0);
    expect(compareBytes(b, a)).toBeGreaterThan(0);
  });
});
