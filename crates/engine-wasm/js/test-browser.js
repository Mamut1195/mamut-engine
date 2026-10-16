import init, * as api from '../pkg/web/engine_wasm.js';
import { workerSuite } from './worker-suite.js';
import { suite, check } from './suite.js';

const output = document.querySelector('#result');
try {
  await init();
  const response = await fetch('../pkg/v1.json');
  check(response.ok, 'native fixture manifest unavailable: run scripts/build-wasm.sh');
  const native = await response.json();
  const count = suite(api, native);
  const cancellation = await workerSuite(native);
  output.textContent = `PASS: ${count} V1 fixtures, direct ES module + Web Worker, native byte parity, progress, cancellation (${cancellation}), fallback`;
  document.documentElement.dataset.status = 'pass';
} catch (error) {
  output.textContent = `FAIL: ${error.stack || error}`;
  document.documentElement.dataset.status = 'fail';
}
