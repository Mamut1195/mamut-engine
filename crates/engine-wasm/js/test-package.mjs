import { readFileSync, existsSync, statSync } from 'node:fs';
import { posix } from 'node:path';

const root = new URL('../', import.meta.url);
const read = path => readFileSync(new URL(path, root), 'utf8');
const isFile = path => existsSync(new URL(path, root)) && statSync(new URL(path, root)).isFile();
let failures = 0;
function check(condition, message) {
  console[condition ? 'log' : 'error'](`${condition ? 'PASS' : 'FAIL'} ${message}`);
  if (!condition) failures++;
}

function exportTargets(value) {
  if (typeof value === 'string') return [value];
  if (value && typeof value === 'object') return Object.values(value).flatMap(exportTargets);
  throw new Error('Destino de exports inválido');
}

try {
  const pkg = JSON.parse(read('package.json'));
  const cargoPackage = read('Cargo.toml').split(/^\[/m).find(section => section.startsWith('package]'));
  const cargoVersion = cargoPackage?.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
  const files = pkg.files;
  if (!Array.isArray(files)) throw new Error('files debe ser un array');
  const allowedKeys = ['name', 'version', 'private', 'type', 'description', 'license',
    'repository', 'exports', 'files', 'scripts'];
  check(Object.keys(pkg).every(key => allowedKeys.includes(key)), 'claves permitidas del manifiesto');
  const allowedScripts = ['test:package'];
  check(pkg.scripts !== null && typeof pkg.scripts === 'object' && !Array.isArray(pkg.scripts) &&
    Object.keys(pkg.scripts).length === allowedScripts.length &&
    Object.keys(pkg.scripts).every(key => allowedScripts.includes(key)), 'solo el script test:package');
  const dependencyKeys = ['dependencies', 'devDependencies', 'peerDependencies',
    'optionalDependencies', 'bundleDependencies', 'bundledDependencies'];
  check(dependencyKeys.every(key => !Object.hasOwn(pkg, key)), 'sin campos de dependencias');
  const allowedFiles = ['js/main.js', 'js/worker.js', 'js/example.html',
    'pkg/web/engine_wasm.js', 'pkg/web/engine_wasm_bg.wasm', 'pkg/web/engine_wasm.d.ts', 'README.md'];
  check(files.length === allowedFiles.length && new Set(files).size === files.length &&
    files.every(path => allowedFiles.includes(path)), 'files contiene exactamente los archivos permitidos sin duplicados');
  check(pkg.name === 'mamut-engine-wasm', 'nombre del paquete');
  check(pkg.private === true, 'paquete privado');
  check(pkg.type === 'module', 'módulo ES');
  check(pkg.license === 'AGPL-3.0-only', 'licencia AGPL-3.0-only');
  check(Boolean(cargoVersion) && pkg.version === cargoVersion, 'versión igual a Cargo.toml del crate');
  check(pkg.repository?.url?.includes('github.com/Mamut1195/mamut-engine'), 'repositorio del código fuente');
  check(pkg.repository?.type === 'git' && pkg.repository?.directory === 'crates/engine-wasm', 'ubicación del paquete en el repositorio');
  check(pkg.exports?.['.'] === './js/main.js', 'export principal');
  check(pkg.exports?.['./worker.js'] === './js/worker.js', 'export del worker');
  check(pkg.exports?.['./wasm']?.types === './pkg/web/engine_wasm.d.ts' &&
    pkg.exports?.['./wasm']?.default === './pkg/web/engine_wasm.js', 'exports del módulo wasm y tipos');
  check(pkg.exports?.['./package.json'] === './package.json', 'export del manifiesto');

  for (const path of ['js/main.js', 'js/worker.js', 'js/example.html', 'README.md']) {
    check(files.includes(path), `files incluye ${path}`);
    check(isFile(path), `existe ${path}`);
  }

  const worker = read('js/worker.js');
  const imports = [...worker.matchAll(/\bfrom\s*['"]([^'"]+)['"]/g)].map(match => match[1]);
  const workerWasm = imports.find(path => path.startsWith('../pkg/web/') && path.endsWith('.js'));
  check(Boolean(workerWasm), 'worker importa el módulo wasm');
  for (const path of imports) {
    check(files.includes(posix.normalize(posix.join('js', path))), `import del worker incluido en files: ${path}`);
  }

  const buildPath = '../../scripts/build-wasm.sh';
  let outputName;
  if (isFile(buildPath)) {
    const build = read(buildPath);
    outputName = build.match(/--out-name(?:\s+|=)["']?([\w-]+)/)?.[1] ??
      build.match(/\b(engine_wasm)\.wasm\b/)?.[1];
    check(Boolean(outputName), 'nombre de salida identificado en scripts/build-wasm.sh');
  } else {
    outputName = workerWasm && posix.basename(workerWasm, '.js');
    check(Boolean(outputName), 'nombre de salida identificado en el import del worker');
  }
  const generated = ['.js', '_bg.wasm', '.d.ts'].map(extension => `pkg/web/${outputName}${extension}`);
  for (const path of generated) check(files.includes(path), `artefacto del build incluido en files: ${path}`);

  for (const target of exportTargets(pkg.exports)) {
    check(target.startsWith('./'), `destino relativo de exports: ${target}`);
    if (target.startsWith('./js/')) check(isFile(target), `destino JS existente: ${target}`);
    if (target.startsWith('./pkg/web/')) {
      check(files.includes(target.slice(2)), `destino wasm incluido en files: ${target}`);
      check(generated.includes(target.slice(2)), `destino wasm coincide con el build: ${target}`);
    }
  }
  if (existsSync(new URL('pkg/web/', root))) {
    const artifacts = new Set([...generated, ...files.filter(path => path.startsWith('pkg/web/')),
      ...exportTargets(pkg.exports).filter(path => path.startsWith('./pkg/web/'))]);
    for (const path of artifacts) check(isFile(path), `artefacto generado existente: ${path}`);
  } else {
    console.log('PASS validación de metadatos sin pkg/web construido');
  }

  const example = read('js/example.html');
  check(/\bexperimental\b/i.test(example) && /\bno\s+es\s+producto\b/i.test(example),
    'ejemplo avisa que es experimental y no es producto');
  check(/<a\b[^>]*\bhref=["']https:\/\/github\.com\/Mamut1195\/mamut-engine["'][^>]*>\s*código fuente correspondiente\s*<\/a>/i.test(example),
    'ejemplo ofrece enlace al código fuente correspondiente');
  check(example.includes('AGPL-3.0'), 'ejemplo indica la licencia AGPL-3.0');
} catch (error) {
  check(false, error.message);
}

console[failures ? 'error' : 'log'](`${failures ? 'FAIL' : 'PASS'} paquete ES: ${failures} errores`);
if (failures) process.exitCode = 1;
