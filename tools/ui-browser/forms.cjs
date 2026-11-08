// Original M2 form UI acceptance against the local browser build; no test-only model mutation.
const { chromium } = require(process.env.VBA_PLAYWRIGHT_MODULE || 'playwright');
const fs = require('node:fs');
const http = require('node:http');
const path = require('node:path');
const assert = require('node:assert/strict');
(async () => {
  const root = path.resolve(__dirname, '../../dist/ui');
  const report = path.resolve(process.env.VBA_UI_REPORT_DIR || path.join(__dirname, '../../reports/ui'));
  const authored = fs.readFileSync(path.resolve(__dirname, '../../examples/forms/CustomerForm.frm'), 'utf8');
  const source = authored.split('End\r\n').slice(1).join('End\r\n');
  const companion = authored.match(/OleObjectBlob = "([^"]+)"/)[1];
  const resources = fs.readFileSync(path.resolve(__dirname, '../../examples/forms', companion));
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
    const choosing = page.waitForEvent('filechooser');
    await click(100, 140);
    await (await choosing).setFiles([{ name: 'CustomerForm.frm', mimeType: 'text/plain', buffer: Buffer.from(authored, 'utf8') }, {name: companion, mimeType:'application/octet-stream', buffer:resources}]);
    await page.waitForFunction(expected => document.querySelector('#vba-source').textContent === expected, source);
    assert.equal(await page.locator('#vba-status').textContent(), 'Design');
    assert.equal(await page.locator('#vba-output').textContent(), '');
    await page.screenshot({ path: path.join(report, 'm2-browser-designer.png') });
    // The form designer owns F5: it launches the actual modal form.
    await click(130, 110); await page.keyboard.press('F5');
    await page.waitForFunction(() => document.querySelector('#vba-status').textContent === 'Waiting for host');
    await page.screenshot({ path: path.join(report, 'm2-browser-runtime.png') });
    await click(620, 507);
    await page.waitForFunction(() => document.querySelector('#vba-status').textContent === 'Break');
    await page.screenshot({ path: path.join(report, 'm2-browser-form-break.png') });
    await page.keyboard.press('F5');
    await page.waitForFunction(() => document.querySelector('#vba-status').textContent === 'Completed');
    assert.equal((await page.locator('#vba-output').textContent()).trim(), '7');
    await page.screenshot({ path: path.join(report, 'm2-browser-form-completed.png') });
    // Export uses design + original source, never mutated runtime control values or generated support.
    await page.keyboard.press('Control+e'); await settle();
    const downloads=[]; page.on('download',download=>downloads.push(download));
    await click(90,140);
    const deadline=Date.now()+30000;
    while(downloads.length<2 && Date.now()<deadline) await page.waitForTimeout(50);
    assert.equal(downloads.length,2);
    const files=[];
    for(const download of downloads){const chunks=[];for await(const chunk of await download.createReadStream())chunks.push(chunk);files.push({name:download.suggestedFilename(),mimeType:'application/octet-stream',buffer:Buffer.concat(chunks)});}
    const frm=files.find(f=>f.name==='CustomerForm.frm');
    const frx=files.find(f=>f.name.endsWith('.frx'));
    assert.ok(frm && frx);
    assert.ok(frm.buffer.toString('utf8').endsWith(source));
    assert.ok(frm.buffer.toString('utf8').includes('OleObjectBlob = "'+frx.name+'":0000'));
    assert.equal(frx.buffer.readUInt32LE(0),frx.buffer.length-4);
    assert.equal(frx.buffer.subarray(4,12).toString('hex'),'d0cf11e0a1b11ae1');
    await page.reload();
    await page.waitForFunction(()=>document.querySelector('#vba-source').textContent.includes('Attribute VB_Name'));
    await click(17,15);await click(70,66);
    const reimport=page.waitForEvent('filechooser');await click(100,140);await(await reimport).setFiles(files);
    await page.waitForFunction(expected=>document.querySelector('#vba-source').textContent===expected,source);
    assert.equal(await page.locator('#vba-output').textContent(),'');
    assert.equal(await page.locator('#vba-status').textContent(),'Design');
    await page.screenshot({path:path.join(report,'m7a6-browser-native-reopened.png')});
    assert.deepEqual(errors, []);
    console.log('BROWSER_FORM_PASS: inert native pair upload; designer; real modal event breakpoint/resume without replay; exact source/binary pair export and inert reimport; no page errors');
  } finally {
    if (browser) await browser.close();
    await new Promise(resolve => server.close(resolve));
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
