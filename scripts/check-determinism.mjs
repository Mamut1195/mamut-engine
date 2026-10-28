// No npm or wasm-bindgen. Cross-target hashes: results/1 without target or scalar bits.
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';

const [wasmPath, nativePath] = process.argv.slice(2);
assert(wasmPath && nativePath, 'Usage: node scripts/check-determinism.mjs module.wasm hashes-native.txt');
const rows = readFileSync(nativePath, 'utf8').trim().split(/\r?\n/).map(row => row.trim().split(/\s+/));
const repetitions = 10;
assert(rows.length > 0 && rows.length % repetitions === 0, 'Ten hashes are required per case');
const module = await WebAssembly.compile(readFileSync(wasmPath));
assert.deepEqual(WebAssembly.Module.imports(module), [], 'The module must run without imports');
const instance = await WebAssembly.instantiate(module, {});
const { fixture_count, fixture_name_hash, run_hash, run_status } = instance.exports;
const count = fixture_count();
assert.equal(count * repetitions, rows.length);
assert.equal(run_status(count), -1, 'Wasm index out of range');
assert.equal(run_hash(count, 0), 0n);
assert.equal(run_hash(0, 4), 0n, 'Hash word out of range');
const hashWords = (fn, index) => Array.from({ length: 4 }, (_, word) =>
  BigInt.asUintN(64, fn(index, word)).toString(16).padStart(16, '0')).join('');
const names = new Set();
for (let index = 0; index < count; index++) {
  const [name, expected] = rows[index * repetitions];
  assert(!names.has(name), `Duplicate case: ${name}`);
  names.add(name);
  assert.match(expected, /^[0-9a-f]{64}$/);
  assert.equal(hashWords(fixture_name_hash, index), createHash('sha256').update(name).digest('hex'),
    `${name}: native/wasm case identity and order`);
  for (let iteration = 0; iteration < repetitions; iteration++) {
    assert.deepEqual(rows[index * repetitions + iteration], [name, expected], `${name}: native repeatability`);
    assert.equal(run_status(index), 0, `${name}: wasm case failed`);
    assert.equal(hashWords(run_hash, index), expected,
      `${name}: native/wasm bitwise equality, run ${iteration}`);
  }
  console.log(`${name}: ${expected}, nativo=wasm, ${repetitions} corridas`);
}
for (const required of [
  'cantilever', 'inclined-cantilever', 'modal-1dof', 'modal-coupled', 'pdelta',
  'spectrum-log-cqc', 'modal-inclined', 'pdelta-inclined', 'spectrum-inclined',
  'scalar-trig', 'scalar-exp-log', 'scalar-powers-norm', 'load-rotation-bits',
  'scalar-sin-cos', 'frame3d-inclined-static', 'frame3d-inclined-modal',
]) {
  assert(names.has(required), `Missing required coverage: ${required}`);
}
console.log(`OK: ${count} casos, SHA-256 de cruce idénticos entre targets`);
