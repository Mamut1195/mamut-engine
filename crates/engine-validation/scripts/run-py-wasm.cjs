// Independent fixtures, without packages or network access; same runner as tests/py.rs.
const fs = require('node:fs');
const assert = require('node:assert/strict');
const path = process.argv[2];
if (!path) throw new Error('Usage: node run-py-wasm.cjs <py_wasm.wasm>');
const wasmModule = new WebAssembly.Module(fs.readFileSync(path));
assert.deepEqual(WebAssembly.Module.imports(wasmModule), [], 'El benchmark debe ser autónomo');
const instance = new WebAssembly.Instance(wasmModule, {});
assert.equal(instance.exports.py_benchmarks(), 3, 'Deben pasar tres fixtures p-y');
console.log('p-y wasm: 3 fixtures analíticos aprobados');
