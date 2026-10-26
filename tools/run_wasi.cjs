// Test runner only. No preopened directories or inherited environment.
const fs = require('node:fs');
const { WASI } = require('node:wasi');
async function main() {
  if (process.argv.length !== 3) throw new Error('usage: node tools/run_wasi.cjs <smoke.wasm>');
  const wasi = new WASI({ version: 'preview1', args: [], env: {}, preopens: {}, returnOnExit: true });
  const { instance } = await WebAssembly.instantiate(fs.readFileSync(process.argv[2]), {
    wasi_snapshot_preview1: wasi.wasiImport,
  });
  process.exitCode = wasi.start(instance);
}
main().catch(error => { console.error(error); process.exitCode = 1; });
