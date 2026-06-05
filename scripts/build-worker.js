/**
 * Worker Build Script
 *
 * Uses esbuild to compile the TypeScript Worker source code into
 * browser-ready JavaScript.
 *
 * Features:
 * 1. Compiles TypeScript to JavaScript
 * 2. Bundles all dependencies
 * 3. Generates source maps
 * 4. Minifies in production environment
 *
 * Usage:
 * ```bash
 * npm run worker:build
 * ```
 *
 * Build output:
 * - static/worker.js - Compiled Worker code
 * - static/worker.js.map - Source map file
 *
 * Dependencies:
 * - esbuild: High-performance JavaScript/TypeScript bundler
 *
 * @module build-worker
 */

import { build } from 'esbuild';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

// Get the current file path (ES Module environment)
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

/**
 * Build the Worker
 *
 * Compiles and bundles the Worker source code using esbuild.
 *
 * Configuration:
 * - entryPoints: Entry file path
 * - bundle: Bundle all dependencies into a single file
 * - outfile: Output file path
 * - format: Output format as ES Module
 * - target: Target environment is modern browsers
 * - platform: Target platform is browser
 * - sourcemap: Generate source maps for debugging
 * - minify: Minify in production environment
 *
 * @throws {Error} Throws an error if the build fails
 */
async function buildWorker() {
  try {
    await build({
      // Entry point: Worker TypeScript source file
      entryPoints: [join(__dirname, '../src/lib/worker/index.ts')],

      // Bundle all dependencies into a single file
      bundle: true,

      // Output file path
      outfile: join(__dirname, '../static/worker.js'),

      // Output format: ES Module
      format: 'esm',

      // Target environment: modern browsers
      target: 'esnext',

      // Target platform: browser
      platform: 'browser',

      // Generate source maps for debugging
      sourcemap: true,

      // Minify in production environment
      minify: process.env.NODE_ENV === 'production'
    });

    // Worker built successfully
  } catch (error) {
    console.error('Worker build failed:', error);
    process.exit(1);
  }
}

// Execute the build
buildWorker();
