// Native form pair upload and existing designer/runtime/debugger, against the local web build.
const { chromium } = require(process.env.VBA_PLAYWRIGHT_MODULE || 'playwright');
const fs = require('node:fs');
const http = require('node:http');
const path = require('node:path');
const assert = require('node:assert/strict');
(async () => {
  const root = path.resolve(__dirname, '../../dist/ui');
  const report = path.resolve(process.env.VBA_UI_REPORT_DIR || path.join(__dirname, '../../reports/ui'));
  const fixture = process.env.VBA_NATIVE_FIXTURE_DIR;
  assert.ok(fixture, 'Run cargo run -p vba-project --example native_form_fixture -- <directory> first');
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
    await settle();
    if (process.env.VBA_NATIVE_FRAME_INPUT) {
      // Initial focus enters both native Frames. Tab leaves for the outer button;
      // Shift+Tab returns to the same nested TextBox before further real typing.
      await page.keyboard.press('Control+a'); await page.keyboard.insertText('nested😀'); await settle();
      await page.keyboard.press('Tab'); await settle();
      await page.keyboard.press('Shift+Tab'); await settle();
      await page.keyboard.press('End'); await page.keyboard.insertText('!'); await settle();
      assert.equal((await page.locator('#vba-output').textContent()).trim(), 'initialized');
    }
    if (process.env.VBA_NATIVE_IMAGE_INPUT) {
      await click(710, 420);
      assert.equal((await page.locator('#vba-output').textContent()).trim(), 'initialized');
    }
    if (process.env.VBA_NATIVE_TAB_INPUT) {
      await page.keyboard.press('ArrowRight'); await settle();
      assert.equal((await page.locator('#vba-output').textContent()).trim(), 'initialized');
    }
    await page.screenshot({ path: path.join(report, 'm7a-native-browser-runtime.png') });
    if (process.env.VBA_NATIVE_TOOLTIP) {
      await page.mouse.move(565, 435); await page.waitForTimeout(1000);
      assert.equal((await page.locator('#vba-output').textContent()).trim(), 'initialized');
      assert.equal(await page.locator('#vba-status').textContent(), 'Waiting for host');
      await page.screenshot({ path: path.join(report, 'm7a-native-browser-control-tip.png') });
    }
    await click(565, 475);
    await page.waitForFunction(() => document.querySelector('#vba-status').textContent === 'Break');
    await page.screenshot({ path: path.join(report, 'm7a-native-browser-form-break.png') });
    await page.keyboard.press('F5');
    await page.waitForFunction(() => document.querySelector('#vba-status').textContent === 'Completed');
    assert.equal((await page.locator('#vba-output').textContent()).trim(), process.env.VBA_NATIVE_IMAGE_INPUT ? 'initialized\n0\nTrue' : process.env.VBA_NATIVE_TAB_INPUT ? 'initialized\n1\nSecond😀' : process.env.VBA_NATIVE_FRAME_INPUT ? 'initialized\nnested😀!' : 'initialized\ninitial');
    await page.screenshot({ path: path.join(report, 'm7a-native-browser-form-completed.png') });
    // Native edited saving is a separate milestone and must not create a custom archive.
    await page.keyboard.press('Control+e'); await settle();
    assert.equal(await page.locator('#vba-source').textContent(), source);
    assert.deepEqual(errors, []);
    console.log('NATIVE_BROWSER_FORM_PASS: inert paired native upload; designer; real modal handler breakpoint/resume without replay; unchanged authored source; no page errors');
  } finally {
    if (browser) await browser.close();
    await new Promise(resolve => server.close(resolve));
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
