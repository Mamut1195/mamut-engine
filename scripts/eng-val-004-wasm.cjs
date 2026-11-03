'use strict';

const fs = require('node:fs');
const path = require('node:path');
const crypto = require('node:crypto');
const { spawnSync } = require('node:child_process');
const { performance } = require('node:perf_hooks');
const { TextDecoder } = require('node:util');

const TIMEOUT_MS = 120000;
const MAX_OUTPUT = 64 * 1024 * 1024;
const MAX_INPUT = 8 * 1024 * 1024;
const hash = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
const decode = bytes => new TextDecoder('utf-8', { fatal: true }).decode(bytes);

function memoryBytes(api) {
  const memory = api.memory || (api.__wasm && api.__wasm.memory);
  return memory instanceof WebAssembly.Memory ? memory.buffer.byteLength : null;
}

function worker(modulePath) {
  const input = decode(fs.readFileSync(0));
  const api = require(modulePath);
  const token = new api.CancelToken();
  try {
    const rssBefore = process.memoryUsage().rss;
    const memoryBefore = memoryBytes(api);
    const start = performance.now();
    const result = api.run(input, token, undefined, undefined);
    const elapsedMs = performance.now() - start;
    const rssAfter = process.memoryUsage().rss;
    const memoryAfter = memoryBytes(api);
    process.stdout.write(JSON.stringify({
      result, elapsedMs, rssBefore, rssAfter, memoryBefore, memoryAfter
    }));
  } finally {
    token.free();
  }
}

function canonical(value) {
  if (Array.isArray(value)) return '[' + value.map(canonical).join(',') + ']';
  if (value !== null && typeof value === 'object') {
    return '{' + Object.keys(value).sort().map(key =>
      JSON.stringify(key) + ':' + canonical(value[key])).join(',') + '}';
  }
  return JSON.stringify(value);
}

function rejectNonFinite(value) {
  if (typeof value === 'number' && !Number.isFinite(value)) {
    throw new Error('Non-finite number in results/1');
  }
  if (value !== null && typeof value === 'object') {
    for (const child of Object.values(value)) rejectNonFinite(child);
  }
}

function normalizeTarget(native, wasm) {
  if (!native || typeof native !== 'object' || Array.isArray(native) ||
      typeof native.target !== 'string' ||
      !/^[A-Za-z0-9_]+(?:-[A-Za-z0-9_]+){2,3}$/.test(native.target) ||
      /^wasm/i.test(native.target) ||
      !wasm || typeof wasm !== 'object' || Array.isArray(wasm) ||
      wasm.target !== 'wasm32-unknown-unknown') {
    throw new Error('Invalid results/1 targets');
  }
  delete native.target;
  delete wasm.target;
}

function checkedChild(binary, args, input, env) {
  const start = performance.now();
  const child = spawnSync(binary, args, {
    input, env, timeout: TIMEOUT_MS, killSignal: 'SIGKILL', maxBuffer: MAX_OUTPUT
  });
  const elapsedMs = performance.now() - start;
  if (child.error) throw new Error(`Process failed: ${child.error.message}`);
  if (child.signal) throw new Error(`Process terminated by ${child.signal}`);
  return { ...child, elapsedMs };
}

function main() {
  if (process.argv[2] === '--worker') return worker(process.argv[3]);
  if (process.argv.length !== 5) {
    throw new Error('Usage: node scripts/eng-val-004-wasm.cjs <engine_wasm.js> <input.json> <native-cli>');
  }
  const modulePath = path.resolve(process.argv[2]);
  const inputPath = path.resolve(process.argv[3]);
  const nativePath = path.resolve(process.argv[4]);
  if (!modulePath.endsWith('.js')) throw new Error('Expected generated Node .js module');
  const wasmPath = modulePath.slice(0, -3) + '_bg.wasm';
  const input = fs.readFileSync(inputPath);
  if (input.length > MAX_INPUT) throw new Error('Input exceeds 8 MiB');
  decode(input);
  const wasmHash = hash(fs.readFileSync(wasmPath));
  const env = { ...process.env };
  delete env.MAMUT_ENGINE_CANCEL_FILE;
  delete env.MAMUT_ENGINE_TIMEOUT;
  delete env.NODE_OPTIONS;
  const native = checkedChild(nativePath, ['run', '--input', '-'], input, env);
  if (![0, 1].includes(native.status)) throw new Error(`Native exit ${native.status}: ${native.stderr.toString('utf8')}`);
  const envelope = JSON.parse(decode(native.stdout));
  if (envelope.operation !== 'run' || envelope.exitCode !== native.status ||
      envelope.outcome !== (native.status === 0 ? 'ok' : 'warning') ||
      !Array.isArray(envelope.engineResults) || envelope.engineResults.length !== 1) {
    throw new Error('Unexpected native run envelope');
  }
  const wasm = checkedChild(process.execPath, ['--max-old-space-size=1024', __filename, '--worker', modulePath], input, env);
  if (wasm.status !== 0) throw new Error(`Wasm worker exit ${wasm.status}: ${wasm.stderr.toString('utf8')}`);
  const measured = JSON.parse(decode(wasm.stdout));
  const nativeResult = envelope.engineResults[0];
  const wasmResult = JSON.parse(measured.result);
  rejectNonFinite(nativeResult);
  rejectNonFinite(wasmResult);
  if (nativeResult.schema !== 'results/1' || wasmResult.schema !== 'results/1' ||
      !['ok', 'warning'].includes(nativeResult.status) ||
      nativeResult.status !== wasmResult.status || nativeResult.status !== envelope.outcome) {
    throw new Error('Unsuccessful or inconsistent results/1 status');
  }
  normalizeTarget(nativeResult, wasmResult);
  const equal = canonical(nativeResult) === canonical(wasmResult);
  console.log(JSON.stringify({
    case: { input: inputPath, inputSha256: hash(input), wasm: wasmPath, wasmSha256: wasmHash },
    comparison: { equal, normalization: 'target only', status: wasmResult.status },
    native: { exitCode: native.status, processWallMs: native.elapsedMs },
    wasm: { callWallMs: measured.elapsedMs, rssBeforeBytes: measured.rssBefore,
      rssAfterBytes: measured.rssAfter, exportedMemoryBeforeBytes: measured.memoryBefore,
      exportedMemoryAfterBytes: measured.memoryAfter,
      exportedMemoryAvailability: measured.memoryBefore === null ? 'unavailable' : 'available' },
    limits: { processTimeoutMs: TIMEOUT_MS, outputBytesPerStream: MAX_OUTPUT,
      wasmWorkerV8OldSpaceMiB: 1024,
      note: 'V8 old-space limit does not cap wasm linear memory or total RSS; native memory is not capped. RSS includes the wasm worker process, not native child RSS.' }
  }, null, 2));
  if (!equal) process.exitCode = 1;
}

try {
  main();
} catch (error) {
  console.error(String(error && error.stack || error));
  process.exitCode = 1;
}
