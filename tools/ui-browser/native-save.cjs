// Authored native CommandButton click focus through the production web UI and debugger.
const { chromium } = require(process.env.VBA_PLAYWRIGHT_MODULE || 'playwright');
const fs = require('node:fs');
const http = require('node:http');
const path = require('node:path');
const assert = require('node:assert/strict');
(async () => {
  const root = path.resolve(__dirname, '../../dist/ui');
  const report = path.resolve(process.env.VBA_UI_REPORT_DIR || path.join(__dirname, '../../reports/ui'));
  const fixture = process.env.VBA_NATIVE_FIXTURE_DIR;
  assert.ok(fixture, 'Run cargo run -p vba-project --example native_form_fixture -- <directory> --button-focus first');
  const form = fs.readFileSync(path.join(fixture, 'NativeForm.frm'));
  const resource = fs.readFileSync(path.join(fixture, 'NativeForm.frx'));
  const source = form.toString('ascii').split('End\r\n').slice(1).join('End\r\n');
  fs.mkdirSync(report, { recursive: true });
  const server = http.createServer((req, res) => {
    const relative = decodeURIComponent(new URL(req.url, 'http://localhost').pathname);
    const file = path.resolve(root, '.' + (relative === '/' ? '/index.html' : relative));
    if (!file.startsWith(root + path.sep)) { res.writeHead(403).end(); return; }
    fs.readFile(file, (error, data) => {
      if (error) { res.writeHead(404).end(); return; }
      const ext = path.extname(file);
      res.setHeader('Content-Type', ext === '.wasm' ? 'application/wasm' : ext === '.js' ? 'text/javascript' : 'text/html');
      res.end(data);
    });
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  let browser;
  try {
    browser = await chromium.launch({ headless: true, args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader'] });
    const page = await browser.newPage({ viewport: { width: 1400, height: 1000 } });
    const errors = []; page.on('pageerror', error => errors.push(String(error)));
    const settle = () => page.waitForTimeout(250);
    const click = async (x, y) => { await page.locator('#vba-canvas').click({ position: { x, y } }); await settle(); };
    await page.goto(`http://127.0.0.1:${server.address().port}/`);
    await page.waitForFunction(() => document.querySelector('#vba-source').textContent.includes('Attribute VB_Name'));
    await click(17, 15); await click(70, 66);
    await page.screenshot({path:path.join(report,'m7a5-native-import-dialog.png')});
    const choosing = page.waitForEvent('filechooser');
    await click(100, 140);
    await (await choosing).setFiles([{ name: 'NativeForm.frm', mimeType: 'text/plain', buffer: form }, { name: 'NativeForm.frx', mimeType: 'application/octet-stream', buffer: resource }]);
    await page.waitForFunction(expected => document.querySelector('#vba-source').textContent === expected, source);
    assert.equal(await page.locator('#vba-status').textContent(), 'Design');
    assert.equal(await page.locator('#vba-output').textContent(), '');
    await page.screenshot({ path: path.join(report, 'm7a-native-browser-designer.png') });
    await click(1320,193);await page.keyboard.press('Control+a');await page.keyboard.type('Saved native');await page.keyboard.press('Enter');await settle();
    await click(600,350);
    await page.keyboard.press('Control+e'); await settle();
    const downloads=[];page.on('download', download=>downloads.push(download));
    await page.screenshot({path:path.join(report,'m7a5-native-save-dialog.png')});
    await click(90,140);
    const deadline=Date.now()+30000;
    while(downloads.length<2&&Date.now()<deadline) await page.waitForTimeout(50);
    assert.equal(downloads.length,2,'native export requires both files');
    const files=[];
    for(const download of downloads){const chunks=[];for await(const chunk of await download.createReadStream())chunks.push(chunk);files.push({name:download.suggestedFilename(),mimeType:'application/octet-stream',buffer:Buffer.concat(chunks)});}
    const frm=files.find(file=>file.name==='NativeForm.frm');
    const frx=files.find(file=>file.name.endsWith('.frx'));
    assert.ok(frm&&frx);
    const text=frm.buffer.toString('latin1');
    assert.ok(text.endsWith(source));
    assert.ok(text.includes('Caption = "Saved native"'),'designer caption must be persisted');
    assert.ok(text.includes('OleObjectBlob = "'+frx.name+'":0000'));
    assert.equal(frx.buffer.readUInt32LE(0),frx.buffer.length-4);
    assert.equal(frx.buffer.subarray(4,12).toString('hex'),'d0cf11e0a1b11ae1');
    assert.equal(await page.locator('#vba-status').textContent(),'Design');
    assert.equal(await page.locator('#vba-output').textContent(),'');
    await page.screenshot({path:path.join(report,'m7a5-native-save-downloads.png')});
    await page.reload();
    await page.waitForFunction(()=>document.querySelector('#vba-source').textContent.includes('Attribute VB_Name'));
    await click(17,15);await click(70,66);
    const reimport=page.waitForEvent('filechooser');await click(100,140);await(await reimport).setFiles(files);
    await page.waitForFunction(expected=>document.querySelector('#vba-source').textContent===expected,source);
    assert.equal(await page.locator('#vba-output').textContent(),'');
    await click(130,110);await page.keyboard.press('F5');
    await page.waitForFunction(()=>document.querySelector('#vba-status').textContent==='Waiting for host');
    assert.equal((await page.locator('#vba-output').textContent()).trim(),'button ready\tFalse\t11');
    await page.screenshot({path:path.join(report,'m7a5-native-save-reopened.png')});
    assert.deepEqual(errors,[]);
    console.log('NATIVE_SAVE_BROWSER_PASS: inert native two-file binary export, exact authored source, explicit reimport and real execution; no page errors');
  } finally {
    if (browser) await browser.close();
    await new Promise(resolve => server.close(resolve));
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
