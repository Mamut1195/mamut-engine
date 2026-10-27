// Runtime for API fixtures and independent derivations; without packages or network access.
const fs = require('node:fs');
const assert = require('node:assert/strict');
const path = process.argv[2];
if (!path) throw new Error('Usage: node run-axial-wasm.cjs <axial_wasm.wasm>');
const wasmModule = new WebAssembly.Module(fs.readFileSync(path));
assert.deepEqual(WebAssembly.Module.imports(wasmModule), [], 'El benchmark debe ser autónomo');
const instance = new WebAssembly.Instance(wasmModule, {});
assert.equal(instance.exports.axial_benchmarks(), 5, 'Deben pasar cinco fixtures axiales');
console.log('Axiales wasm: 5 fixtures aprobados');
