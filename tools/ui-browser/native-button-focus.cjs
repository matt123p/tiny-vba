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
    const choosing = page.waitForEvent('filechooser');
    await click(100, 140);
    await (await choosing).setFiles([{ name: 'NativeForm.frm', mimeType: 'text/plain', buffer: form }, { name: 'NativeForm.frx', mimeType: 'application/octet-stream', buffer: resource }]);
    await page.waitForFunction(expected => document.querySelector('#vba-source').textContent === expected, source);
    assert.equal(await page.locator('#vba-status').textContent(), 'Design');
    assert.equal(await page.locator('#vba-output').textContent(), '');
    await page.screenshot({ path: path.join(report, 'm7a-native-browser-designer.png') });
    // The form designer owns F5: it launches the actual modal form.
    await click(130, 110); await page.keyboard.press('F5');
    await page.waitForFunction(() => document.querySelector('#vba-status').textContent === 'Waiting for host');
    await page.screenshot({ path: path.join(report, 'm7a-native-browser-runtime.png') });
    assert.equal((await page.locator('#vba-output').textContent()).trim(), 'button ready\tFalse\t11');
    await click(580, 530);
    await page.waitForFunction(() => document.querySelector('#vba-status').textContent === 'Break');
    assert.equal((await page.locator('#vba-output').textContent()).trim(), 'button ready\tFalse\t11\ndown\tEntry');
    await page.screenshot({path:path.join(report,'m7a4-native-button-focus-break.png')});
    await page.keyboard.press('F5');
    await page.waitForFunction(() => document.querySelector('#vba-output').textContent.includes('click\tEntry\t1\t2'));
    assert.equal((await page.locator('#vba-output').textContent()).trim(), 'button ready\tFalse\t11\ndown\tEntry\ncontinued\tEntry\nclick\tEntry\t1\t2');
    await page.screenshot({path:path.join(report,'m7a4-native-button-focus-selection.png')});
    await page.keyboard.type('Z');
    await page.waitForFunction(() => document.querySelector('#vba-output').textContent.includes('typed\taZde'));
    await click(580, 530);
    await page.waitForFunction(() => document.querySelector('#vba-status').textContent === 'Completed');
    assert.equal((await page.locator('#vba-output').textContent()).trim(), 'button ready\tFalse\t11\ndown\tEntry\ncontinued\tEntry\nclick\tEntry\t1\t2\ntyped\taZde\nclick\tApply\t2\t0\ndone\taZde\tTrue');
    await page.screenshot({path:path.join(report,'m7a4-native-button-focus-completed.png')});
    // Native edited saving is a separate milestone and must not create a custom archive.
    await page.keyboard.press('Control+e'); await settle();
    assert.equal(await page.locator('#vba-source').textContent(), source);
    assert.deepEqual(errors, []);
    console.log('NATIVE_BUTTON_FOCUS_BROWSER_PASS: inert native False/Boolean; click waits across Stop/Continue; selection/focus retained for typing; runtime True takes focus on next click; exact source; no page errors');
  } finally {
    if (browser) await browser.close();
    await new Promise(resolve => server.close(resolve));
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
