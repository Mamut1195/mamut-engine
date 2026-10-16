import { EngineWorker } from './main.js';
import { equal, check } from './suite.js';

export async function workerSuite(native) {
  const client = new EngineWorker();
  const fallback = new EngineWorker({ sharedCancellation: false });
  try {
    for (const operation of ['version', 'capabilities']) {
      equal(await client.request(operation).promise, native[operation], `worker ${operation}`);
    }
    for (const [kind, schema] of Object.entries(native.schemas)) {
      equal(await client.request('schema', kind).promise, schema, `worker ${kind}`);
    }
    equal(await client.request('explain', 'API-I-CANCELLED').promise, native.explain, 'worker explain');
    for (const fixture of native.cases) {
      const events = [];
      equal(await client.request('run', fixture.input, { onProgress: json => events.push(json) }).promise,
        fixture.run, `worker ${fixture.name} results/1`);
      equal(JSON.stringify(events), JSON.stringify(fixture.runProgress), `worker ${fixture.name} progress`);
      try {
        const value = await client.request('validate', fixture.input).promise;
        equal(value, fixture.validate.value, `worker ${fixture.name} validate`);
      } catch (error) {
        equal(error.message, fixture.validate.error, `worker ${fixture.name} validation error`);
      }
    }
    const input = native.cases[0].input;
    const request = client.request('run', input);
    request.cancel();
    if (request.cancellation === 'cooperative') {
      equal(JSON.parse(await request.promise).status, 'cancelled', 'worker shared cancellation');
      // A larger valid model keeps a solver stage active while the main thread signals cancellation.
      const large = JSON.parse(native.cases.find(c => c.name.endsWith('cantilever-linear-static.json')).input);
      const node = large.model.nodes[0];
      large.model.nodes = Array.from({ length: 200 }, (_, i) => ({ ...node, id: i + 1, coordinates: [4 * i, 0, 0] }));
      const member = large.model.members[0];
      large.model.members = Array.from({ length: 199 }, (_, i) => ({ ...member, id: i + 1, nodes: [i + 1, i + 2] }));
      large.cases[0].loads[0].node = 200;
      const events = [];
      const running = client.request('run', JSON.stringify(large), {
        onProgress: json => { events.push(json); running.cancel(); },
      });
      equal(JSON.parse(await running.promise).status, 'cancelled', 'worker active shared cancellation');
      check(events.length > 0, 'active cancellation requires progress');
    } else {
      try { await request.promise; throw new Error('expected AbortError'); }
      catch (error) { equal(error.name, 'AbortError', 'worker fallback cancellation'); }
    }
    equal(await client.request('run', input).promise, native.cases[0].run, 'client reuse after cancel');
    const terminated = fallback.request('run', input);
    equal(terminated.cancellation, 'terminate', 'explicit fallback');
    terminated.cancel();
    try { await terminated.promise; throw new Error('expected AbortError'); }
    catch (error) { equal(error.name, 'AbortError', 'termination fallback'); }
    equal(await fallback.request('run', input).promise, native.cases[0].run, 'worker recreated after termination');
    return request.cancellation;
  } finally {
    client.close();
    fallback.close();
  }
}
