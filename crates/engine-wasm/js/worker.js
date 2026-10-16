import init, * as api from '../pkg/web/engine_wasm.js';

const ready = init();
let active = null;
self.onmessage = async ({ data }) => {
  const { id, operation, input, flag } = data;
  if (operation === 'cancel') {
    if (active?.id === id) active.token.cancel();
    return;
  }
  let token;
  try {
    await ready;
    token = new api.CancelToken();
    active = { id, token };
    const poll = () => {
      if (flag && Atomics.load(flag, 0) !== 0) token.cancel();
    };
    poll();
    const progress = json => {
      poll();
      self.postMessage({ id, type: 'progress', json });
      poll();
    };
    let json;
    switch (operation) {
      case 'run': case 'validate': json = api[operation](input, token, progress); break;
      case 'version': case 'capabilities': json = api[operation](); break;
      case 'schema': case 'explain': json = api[operation](input); break;
      default: throw new Error(`Unknown operation: ${operation}`);
    }
    self.postMessage({ id, type: 'result', json });
  } catch (error) {
    self.postMessage({ id, type: 'error', error: String(error) });
  } finally {
    active = null;
    token?.free();
  }
};
