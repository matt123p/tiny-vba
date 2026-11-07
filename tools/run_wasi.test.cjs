// Independently authored WASI guests; no Office output or external fixtures.
const assert = require('node:assert/strict');
const { spawnSync } = require('node:child_process');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const test = require('node:test');
function size(n) {
  const out = [];
  do { const b = n & 127; n >>>= 7; out.push(b | (n ? 128 : 0)); } while (n);
  return out;
}
const text = value => { const bytes = [...Buffer.from(value)]; return [...size(bytes.length), ...bytes]; };
const section = (id, body) => [id, ...size(body.length), ...body];
const signature = (args, result) => [0x60, args.length, ...args, result.length, ...result];
const i32 = n => { assert.ok(n >= 0 && n < 64); return [0x41, n]; };
const call = (index, ...args) => [...args.flat(), 0x10, index];
const failIf = [0x04, 0x40, 0x00, 0x0b];
const load = address => [...i32(address), 0x28, 2, 0];
function guest(body, locals = [0], firstImport = 'fd_write') {
  const i = 0x7f;
  const types = [signature([i,i,i,i],[i]), signature([i,i],[i]), signature([i],[]), signature([i,0x7e,i,i],[i]), signature([],[])];
  const imports = [[firstImport,0],['environ_sizes_get',1],['args_sizes_get',1],['fd_prestat_get',1],['proc_exit',2],['fd_seek',3]];
  const code = [...locals, ...body, 0x0b];
  return Buffer.from([0,97,115,109,1,0,0,0,
    ...section(1,[types.length,...types.flat()]),
    ...section(2,[imports.length,...imports.flatMap(([name,type])=>[...text('wasi_snapshot_preview1'),...text(name),0,type])]),
    ...section(3,[1,4]), ...section(5,[1,1,1,64]),
    ...section(7,[2,...text('memory'),2,0,...text('_start'),0,imports.length]),
    ...section(10,[1,...size(code.length),...code])]);
}
function run(bytes) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(),'tiny-vba-wasi-runner-'));
  try {
    const module = path.join(dir,'guest.wasm'); fs.writeFileSync(module,bytes);
    return spawnSync(process.execPath,[path.join(__dirname,'run_wasi.cjs'),module],{
      encoding:'utf8',timeout:30000,env:{...process.env,TINY_VBA_WASI_PARENT_ONLY:'must stay outside guest'}});
  } finally { fs.rmSync(dir,{recursive:true,force:true}); }
}
test('WASI exit status crosses the import boundary unchanged',()=>{
  const result=run(guest(call(4,i32(7)))); assert.equal(result.status,7,result.stderr); assert.equal(result.signal,null);
});
test('guest traps remain failures',()=>{
  const result=run(guest([0])); assert.equal(result.status,1,result.stderr); assert.match(result.stderr,/unreachable/);
});
test('import namespace exposes only explicit WASI callbacks',()=>{
  const result=run(guest([], [0], 'constructor'));
  assert.equal(result.status,1,result.stderr); assert.match(result.stderr,/LinkError/);
});
test('guest receives no inherited environment, arguments or preopened directory',()=>{
  const result=run(guest([
    ...call(1,i32(0),i32(4)),...failIf,...load(0),...failIf,...load(4),...failIf,
    ...call(2,i32(8),i32(12)),...failIf,...load(8),...failIf,...load(12),...failIf,
    ...call(3,i32(3),i32(16)),0x45,...failIf]));
  assert.equal(result.status,0,result.stderr); assert.equal(result.signal,null);
});
test('memory growth, native writes and i64 arguments retain WASI errors and values',()=>{
  const result=run(guest([
    ...i32(32),0x21,0,0x02,0x40,0x03,0x40,
    ...i32(1),0x40,0,0x1a,
    ...call(0,i32(1),i32(0),i32(1),i32(24)),...failIf,
    ...call(5,i32(3),[0x42,0],i32(0),i32(32)),0x45,...failIf,
    0x20,0,...i32(1),0x6b,0x22,0,0x45,0x0d,1,0x0c,0,0x0b,0x0b], [1,1,0x7f]));
  assert.equal(result.status,0,result.stderr); assert.equal(result.signal,null);
});
