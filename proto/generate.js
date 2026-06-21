#!/usr/bin/env node
/**
 * Protobuf code generation script for VTP consensus messages.
 *
 * Generates:
 * 1. Static TypeScript module (vtp_messages.ts) — tree-shakeable, type-safe
 * 2. JSON descriptor (vtp_messages.json) — for dynamic loading fallback
 *
 * Usage:  node proto/generate.js
 * Requires: npm install (in proto/ directory)
 */

import { execSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { readFileSync, writeFileSync } from 'node:fs';

const __dirname = dirname(fileURLToPath(import.meta.url));
const PROTO_FILE = resolve(__dirname, 'vtp_messages.proto');
const OUT_JS = resolve(__dirname, 'vtp_messages.js');
const OUT_DTS = resolve(__dirname, 'vtp_messages.d.ts');
const OUT_JSON = resolve(__dirname, 'vtp_messages.json');

function run(cmd) {
  console.log(`> ${cmd}`);
  execSync(cmd, { stdio: 'inherit', cwd: __dirname });
}

// Step 1: Generate static module (ES6)
console.log('\n[vtp-proto] Generating static TypeScript module...');
run(`npx pbjs -t static-module -w es6 --force-long -o ${OUT_JS} ${PROTO_FILE}`);

// Step 2: Generate TypeScript declarations
console.log('\n[vtp-proto] Generating TypeScript declarations...');
run(`npx pbts -o ${OUT_DTS} ${OUT_JS}`);

// Step 3: Generate JSON descriptor (for dynamic loading)
console.log('\n[vtp-proto] Generating JSON descriptor...');
run(`npx pbjs -t json -o ${OUT_JSON} ${PROTO_FILE}`);

// Step 4: Patch the generated .d.ts to add export default
console.log('\n[vtp-proto] Patching declarations for ES module compatibility...');
let dts = readFileSync(OUT_DTS, 'utf-8');
// Add export at the top level if not present
if (!dts.includes('export')) {
  dts = dts.replace(/import \* as \$protobuf/, 'import type * as $protobuf');
}
writeFileSync(OUT_DTS, dts);

console.log('\n[vtp-proto] Done. Generated files:');
console.log(`  ${OUT_JS}`);
console.log(`  ${OUT_DTS}`);
console.log(`  ${OUT_JSON}`);
