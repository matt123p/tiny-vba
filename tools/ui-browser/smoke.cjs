// Maintenance-only browser smoke test. Serves only the locally built, original UI bundle.
const { chromium } = require(process.env.VBA_PLAYWRIGHT_MODULE || 'playwright');
const fs = require('node:fs');
const http = require('node:http');
const path = require('node:path');
const assert = require('node:assert/strict');
(async () => {
  const root = path.resolve(__dirname, '../../dist/ui');
  const report = path.resolve(process.env.VBA_UI_REPORT_DIR || path.join(__dirname, '../../reports/ui'));
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
    const page = await browser.newPage({ viewport: { width: 1200, height: 800 } });
    const errors = [];
    page.on('pageerror', error => errors.push(String(error)));
    await page.goto(`http://127.0.0.1:${server.address().port}/`);
    await page.waitForTimeout(2500);
    await page.locator('#vba-canvas').click({ position: { x: 500, y: 230 } });
    await page.keyboard.press('F8');
    await page.waitForFunction(() => document.querySelector('#vba-status').textContent === 'Break');
    await page.screenshot({ path: path.join(report, 'browser-break.png') });
    await page.keyboard.press('F5');
    await page.waitForFunction(() => document.querySelector('#vba-status').textContent === 'Completed');
    assert.match(await page.locator('#vba-output').textContent(), /10/);
    await page.keyboard.press('Control+g');
    await page.screenshot({ path: path.join(report, 'browser-completed.png') });
    assert.deepEqual(errors, []);
    console.log('BROWSER_UI_PASS: real F8 pause, F5 completion, output 10, no page errors');
  } finally {
    if (browser) await browser.close();
    await new Promise(resolve => server.close(resolve));
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
