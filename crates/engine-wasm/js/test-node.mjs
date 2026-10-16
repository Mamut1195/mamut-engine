import { readFileSync, readdirSync } from 'node:fs';
import { createRequire } from 'node:module';
import { suite, equal } from './suite.js';
const require = createRequire(import.meta.url);
const api = require('../pkg/node/engine_wasm.js');
const native = JSON.parse(readFileSync(new URL('../pkg/v1.json', import.meta.url), 'utf8'));
const inventory = ['valid', 'invalid'].flatMap(category =>
  readdirSync(new URL(`../../engine-api/fixtures/input/${category}/`, import.meta.url))
    .filter(name => name.endsWith('.json')).sort().map(name => `${category}/${name}`));
equal(JSON.stringify(native.cases.map(c => c.name)), JSON.stringify(inventory), 'complete fixture inventory');
console.log(`PASS Node V1: ${suite(api, native)} fixtures, results/1 byte-identical to native`);

// Exercise the generated ES module too, then the real worker/client with a Node host adapter.
const web = await import('../pkg/web/engine_wasm.js');
await web.default({ module_or_path: readFileSync(new URL('../pkg/web/engine_wasm_bg.wasm', import.meta.url)) });
console.log(`PASS Node web ES module: ${suite(web, native)} fixtures`);
const { Worker } = await import('node:worker_threads');
globalThis.crossOriginIsolated = true;
globalThis.Worker = class {
  constructor(url) {
    this.thread = new Worker(new URL('./node-worker-adapter.mjs', import.meta.url), { workerData: { url: url.href } });
    this.thread.on('message', data => this.onmessage?.({ data }));
    this.thread.on('error', error => this.onerror?.({ message: error.message }));
    this.thread.on('messageerror', () => this.onmessageerror?.());
  }
  postMessage(data) { this.thread.postMessage(data); }
  terminate() { void this.thread.terminate(); }
};
const { workerSuite } = await import('./worker-suite.js');
const deadline = setTimeout(() => { console.error('FAIL worker suite timeout'); process.exit(1); }, 30000);
try {
  const cancellation = await workerSuite(native);
  console.log(`PASS Node Web Worker adapter: ${native.cases.length} fixtures, progress, ${cancellation} cancellation + termination fallback`);
} finally { clearTimeout(deadline); }
