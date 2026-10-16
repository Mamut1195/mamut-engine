// Shared, dependency-free V1 assertions for Node and browser bindings.
export function equal(actual, expected, label) {
  if (actual !== expected) throw new Error(`${label}: byte mismatch\n${actual}\n!=\n${expected}`);
}
export function check(condition, label) {
  if (!condition) throw new Error(label);
}
function capture(operation) {
  try { return { value: operation() }; }
  catch (error) { return { error: String(error) }; }
}
export function suite(api, native) {
  equal(api.version(), native.version, 'version');
  equal(api.capabilities(), native.capabilities, 'capabilities');
  for (const [kind, schema] of Object.entries(native.schemas)) equal(api.schema(kind), schema, kind);
  equal(api.explain('API-I-CANCELLED'), native.explain, 'explain');
  equal(JSON.parse(capture(() => api.schema('unknown')).error).kind, 'UnknownSchema', 'schema error');
  equal(JSON.parse(capture(() => api.explain('unknown')).error).kind, 'UnknownCode', 'explain error');
  check(native.cases.some(c => c.name.startsWith('valid/')), 'valid fixtures missing');
  check(native.cases.some(c => c.name.startsWith('invalid/')), 'invalid fixtures missing');
  for (const fixture of native.cases) {
    const token = new api.CancelToken();
    try {
      const events = [];
      const validation = capture(() => api.validate(fixture.input, token, json => events.push(json)));
      equal(JSON.stringify(validation), JSON.stringify(fixture.validate), `${fixture.name} validate`);
      equal(JSON.stringify(events), JSON.stringify(fixture.validateProgress), `${fixture.name} validate progress`);
      events.length = 0;
      const result = api.run(fixture.input, token, json => events.push(json));
      equal(result, fixture.run, `${fixture.name} results/1`);
      equal(api.run(fixture.input, token), result, `${fixture.name} repeat without callback`);
      equal(JSON.stringify(events), JSON.stringify(fixture.runProgress), `${fixture.name} run progress`);
      const document = JSON.parse(result);
      equal(document.schema, 'results/1', `${fixture.name} schema`);
      if (fixture.expected) {
        equal(document.status, 'invalid', `${fixture.name} invalid status`);
        check(document.diagnostics.some(d => d.code === fixture.expected.code), `${fixture.name} expected diagnostic`);
        equal(fixture.exitCode, fixture.expected.exit_code, `${fixture.name} exit code`);
      } else check(['ok', 'warning'].includes(document.status), `${fixture.name} valid status`);
    } finally { token.free(); }
  }
  const input = native.cases[0].input;
  const token = new api.CancelToken();
  try {
    check(!token.is_cancelled(), 'new token active');
    equal(api.run(input, token, () => token.cancel()), native.checkpointCancelled, 'callback cancellation');
    check(token.is_cancelled(), 'token cancelled');
    equal(api.run(input, token), native.preCancelled, 'pre-cancelled run');
    equal(JSON.stringify(capture(() => api.validate(input, token))),
      JSON.stringify(native.validationCancelled), 'pre-cancelled validate');
  } finally { token.free(); }
  for (const operation of ['validate', 'run']) {
    const token = new api.CancelToken();
    try {
      equal(capture(() => api[operation](input, token, () => { throw 'callback-failure'; })).error,
        'callback-failure', `${operation} callback exception`);
    } finally { token.free(); }
    const cancelled = new api.CancelToken();
    try {
      const result = capture(() => api[operation](input, cancelled, () => cancelled.cancel()));
      if (operation === 'validate') equal(JSON.parse(result.error).kind, 'Cancelled', 'validate checkpoint cancellation');
    } finally { cancelled.free(); }
  }
  return native.cases.length;
}
