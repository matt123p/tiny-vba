// Test runner only. No preopened directories or inherited environment.
const fs = require('node:fs');
const { WASI } = require('node:wasi');
async function main() {
  if (process.argv.length !== 3) throw new Error('usage: node tools/run_wasi.cjs <smoke.wasm>');
  const wasi = new WASI({ version: 'preview1', args: [], env: {}, preopens: {}, returnOnExit: true });
  // Keep an ordinary JS boundary around native WASI callbacks. Direct native
  // imports SIGSEGV in Node 22 on the form-instance suite; forwarding the same
  // guest bytes passes. Do not coerce results, catch guest errors or add authority.
  const imports = Object.create(null);
  for (const [name, callback] of Object.entries(wasi.wasiImport)) {
    imports[name] = (...args) => callback(...args);
  }
  const { instance } = await WebAssembly.instantiate(fs.readFileSync(process.argv[2]), {
    wasi_snapshot_preview1: imports,
  });
  process.exitCode = wasi.start(instance);
}
main().catch(error => { console.error(error); process.exitCode = 1; });
