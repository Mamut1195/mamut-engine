// Runtime mínimo de los fixtures propios: Node y WebAssembly, sin paquetes.
const fs = require('node:fs');
const assert = require('node:assert/strict');
const path = process.argv[2];
if (!path) throw new Error('Uso: node run-winkler-wasm.cjs <winkler_wasm.wasm>');
const moduleBytes = fs.readFileSync(path);
const wasmModule = new WebAssembly.Module(moduleBytes);
assert.deepEqual(WebAssembly.Module.imports(wasmModule), [], 'El benchmark debe ser autónomo');
const instance = new WebAssembly.Instance(wasmModule, {});
assert.equal(instance.exports.winkler_benchmarks(), 6, 'Deben pasar los seis fixtures Winkler');
console.log('Winkler wasm: 6 fixtures analíticos aprobados');
