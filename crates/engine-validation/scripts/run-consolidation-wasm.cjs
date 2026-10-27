// Runtime for independent fixtures, without packages or network access.
const fs = require('node:fs');
const assert = require('node:assert/strict');
const path = process.argv[2];
if (!path) throw new Error('Usage: node run-consolidation-wasm.cjs <consolidation_wasm.wasm>');
const wasmModule = new WebAssembly.Module(fs.readFileSync(path));
assert.deepEqual(WebAssembly.Module.imports(wasmModule), [], 'El benchmark debe ser autónomo');
const instance = new WebAssembly.Instance(wasmModule, {});
assert.equal(instance.exports.consolidation_benchmarks(), 7, 'Deben pasar siete fixtures y la convergencia');
console.log('Consolidación wasm: 7 fixtures analíticos y convergencia aprobados');
