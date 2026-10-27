// Transport an engine-input/1 document through stdin and results/1 through stdout.
// The Rust runner validates source and compares both targets against the same oracle.
const fs = require('node:fs');
const assert = require('node:assert/strict');
const wasmPath = process.argv[2];
if (!wasmPath) throw new Error('Usage: node run-fixture-wasm.cjs module.wasm < input.json');
const wasmModule = new WebAssembly.Module(fs.readFileSync(wasmPath));
assert.deepEqual(WebAssembly.Module.imports(wasmModule), [], 'El módulo debe tener cero imports');
const { input_clear, input_push, execute, output_byte } = new WebAssembly.Instance(wasmModule, {}).exports;
assert.equal(input_clear(), 0);
assert.equal(input_push(256), 1, 'La ABI rechaza valores fuera de byte');
assert.equal(output_byte(0), 256, 'Salida vacía');
for (const byte of fs.readFileSync(0)) assert.equal(input_push(byte), 0, 'Límite de entrada o fallo de ABI');
const size = execute();
assert(size > 0, 'La ABI debe devolver un documento');
assert.equal(output_byte(size), 256, 'Índice fuera de rango');
const output = Buffer.alloc(size);
for (let i = 0; i < size; i++) {
  const byte = output_byte(i);
  assert(byte <= 255, 'Byte de salida inválido');
  output[i] = byte;
}
const document = JSON.parse(output.toString('utf8'));
assert.equal(document.schema, 'results/1');
assert.equal(document.target, 'wasm32-unknown-unknown', 'Procedencia del target wasm');
process.stdout.write(output);
