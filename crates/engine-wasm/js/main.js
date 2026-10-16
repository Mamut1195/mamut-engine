// One in-flight operation per client. Create multiple clients for independent jobs.
export class EngineWorker {
  constructor({ sharedCancellation = true } = {}) {
    this.sharedCancellation = sharedCancellation;
    this.sequence = 0;
    this.pending = null;
    this.worker = null;
    this.closed = false;
  }

  request(operation, input = '', { onProgress = () => {} } = {}) {
    if (this.closed) throw new Error('Worker client is closed');
    if (this.pending) throw new Error('Worker client is busy');
    if (!this.worker) {
      this.worker = new Worker(new URL('./worker.js', import.meta.url), { type: 'module' });
      this.worker.onmessage = ({ data }) => {
        const pending = this.pending;
        if (!pending || data.id !== pending.id) return;
        if (data.type === 'progress') {
          try { pending.onProgress(data.json); }
          catch (error) { this.stop(error); }
        } else {
          this.pending = null;
          if (data.type === 'result') pending.resolve(data.json);
          else pending.reject(new Error(data.error));
        }
      };
      this.worker.onerror = event => this.stop(new Error(event.message));
      this.worker.onmessageerror = () => this.stop(new Error('Worker message could not be decoded'));
    }
    const id = ++this.sequence;
    const flag = this.sharedCancellation && globalThis.crossOriginIsolated &&
      typeof SharedArrayBuffer === 'function' ? new Int32Array(new SharedArrayBuffer(4)) : null;
    const promise = new Promise((resolve, reject) => {
      this.pending = { id, resolve, reject, onProgress };
    });
    try { this.worker.postMessage({ id, operation, input, flag }); }
    catch (error) { this.stop(error); }
    return {
      promise,
      cancellation: flag ? 'cooperative' : 'terminate',
      cancel: () => {
        if (this.pending?.id !== id) return;
        if (flag) {
          Atomics.store(flag, 0, 1);
          this.worker.postMessage({ id, operation: 'cancel' });
        } else {
          this.stop(new DOMException('Worker terminated by cancellation', 'AbortError'));
        }
      },
    };
  }

  stop(error) {
    this.worker?.terminate();
    this.worker = null;
    this.pending?.reject(error);
    this.pending = null;
  }

  close() {
    this.closed = true;
    this.stop(new DOMException('Worker client closed', 'AbortError'));
  }
}
