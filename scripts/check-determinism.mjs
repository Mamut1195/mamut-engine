// Sin npm ni wasm-bindgen. Hashes de cruce: results/1 sin target o bits escalares.
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import assert from 'node:assert/strict';

const [wasmPath, nativePath] = process.argv.slice(2);
assert(wasmPath && nativePath, 'Uso: node scripts/check-determinism.mjs módulo.wasm hashes-native.txt');
const rows = readFileSync(nativePath, 'utf8').trim().split(/\r?\n/).map(row => row.trim().split(/\s+/));
const repetitions = 10;
assert(rows.length > 0 && rows.length % repetitions === 0, 'Se requieren diez hashes por caso');
const module = await WebAssembly.compile(readFileSync(wasmPath));
assert.deepEqual(WebAssembly.Module.imports(module), [], 'El módulo debe ejecutarse sin imports');
const instance = await WebAssembly.instantiate(module, {});
const { fixture_count, fixture_name_hash, run_hash, run_status } = instance.exports;
const count = fixture_count();
assert.equal(count * repetitions, rows.length);
assert.equal(run_status(count), -1, 'Índice wasm fuera de rango');
assert.equal(run_hash(count, 0), 0n);
assert.equal(run_hash(0, 4), 0n, 'Palabra de hash fuera de rango');
const hashWords = (fn, index) => Array.from({ length: 4 }, (_, word) =>
  BigInt.asUintN(64, fn(index, word)).toString(16).padStart(16, '0')).join('');
const names = new Set();
for (let index = 0; index < count; index++) {
  const [name, expected] = rows[index * repetitions];
  assert(!names.has(name), `Caso duplicado: ${name}`);
  names.add(name);
  assert.match(expected, /^[0-9a-f]{64}$/);
  assert.equal(hashWords(fixture_name_hash, index), createHash('sha256').update(name).digest('hex'),
    `${name}: identidad y orden de casos nativo/wasm`);
  for (let iteration = 0; iteration < repetitions; iteration++) {
    assert.deepEqual(rows[index * repetitions + iteration], [name, expected], `${name}: repetibilidad nativa`);
    assert.equal(run_status(index), 0, `${name}: caso wasm falló`);
    assert.equal(hashWords(run_hash, index), expected,
      `${name}: igualdad bit a bit nativo/wasm, corrida ${iteration}`);
  }
  console.log(`${name}: ${expected}, nativo=wasm, ${repetitions} corridas`);
}
for (const required of [
  'cantilever', 'inclined-cantilever', 'modal-1dof', 'modal-coupled', 'pdelta',
  'spectrum-log-cqc', 'modal-inclined', 'pdelta-inclined', 'spectrum-inclined',
  'scalar-trig', 'scalar-exp-log', 'scalar-powers-norm', 'load-rotation-bits',
  'scalar-sin-cos', 'frame3d-inclined-static', 'frame3d-inclined-modal',
]) {
  assert(names.has(required), `Falta cobertura obligatoria: ${required}`);
}
console.log(`OK: ${count} casos, SHA-256 de cruce idénticos entre targets`);
