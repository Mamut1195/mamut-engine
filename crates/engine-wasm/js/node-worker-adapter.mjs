// Test-only Web Worker host for Node. Runs the actual browser worker.js and web binding.
import { parentPort, workerData } from 'node:worker_threads';
import { readFile } from 'node:fs/promises';

globalThis.self = globalThis;
self.postMessage = data => parentPort.postMessage(data);
const fetchNative = globalThis.fetch;
globalThis.fetch = async (url, options) => {
  if (url instanceof URL && url.protocol === 'file:') {
    return new Response(await readFile(url), { headers: { 'Content-Type': 'application/wasm' } });
  }
  return fetchNative(url, options);
};
await import(workerData.url);
parentPort.on('message', data => self.onmessage({ data }));
