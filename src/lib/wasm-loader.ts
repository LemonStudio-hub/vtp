/**
 * WASM Module Loader
 *
 * Loads the vtp-core WASM module for use on the main thread.
 * Uses the same fetch + blob import strategy as the Web Worker
 * to bypass Vite's static analysis.
 */

import type { CryptoProvider } from '$lib/consensus/crypto-provider';
import {
  createWasmCryptoProvider,
  createPlaceholderCryptoProvider
} from '$lib/consensus/crypto-provider';

/** Cached WASM module instance. */
let wasmModule: Record<string, unknown> | null = null;

/** Cached crypto provider. */
let cryptoProvider: CryptoProvider | null = null;

/**
 * Load the vtp-core WASM module.
 *
 * @returns The loaded WASM module exports.
 */
export async function loadWasmModule(): Promise<Record<string, unknown>> {
  if (wasmModule) return wasmModule;

  const response = await fetch('/wasm/vtp_core.js');
  const wasmCode = await response.text();

  const blob = new Blob([wasmCode], { type: 'application/javascript' });
  const blobUrl = URL.createObjectURL(blob);
  const mod = await import(/* @vite-ignore */ blobUrl);
  URL.revokeObjectURL(blobUrl);

  await mod.default('/wasm/vtp_core_bg.wasm');
  wasmModule = mod;
  return mod;
}

/**
 * Get or create the crypto provider backed by the WASM module.
 *
 * Falls back to a placeholder provider if WASM loading fails
 * (e.g., in test environments or when WASM is unavailable).
 */
export async function getCryptoProvider(): Promise<CryptoProvider> {
  if (cryptoProvider) return cryptoProvider;

  try {
    const wasm = await loadWasmModule();
    cryptoProvider = createWasmCryptoProvider(
      wasm as Parameters<typeof createWasmCryptoProvider>[0]
    );
    return cryptoProvider;
  } catch (err) {
    console.warn('[WASM] Failed to load WASM module, using placeholder crypto:', err);
    cryptoProvider = createPlaceholderCryptoProvider();
    return cryptoProvider;
  }
}
