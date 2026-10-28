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
    const settle = () => page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));
    const typeSource = async source => {
      const lines = source.split('\n');
      let expected = '';
      for (let i = 0; i < lines.length; ++i) {
        if (lines[i]) {
          await page.keyboard.insertText(lines[i]);
          expected += lines[i];
          await page.waitForFunction(value => document.querySelector('#vba-source').textContent === value, expected);
        }
        if (i + 1 < lines.length) {
          await page.keyboard.press('Escape');
          await page.keyboard.press('Enter');
          expected += '\n';
          const indent = lines[i].match(/^[ \t]*/)[0];
          await page.waitForFunction(value => document.querySelector('#vba-source').textContent === value, expected + indent);
          if (indent) {
            await page.keyboard.press('Shift+Home');
            await page.keyboard.press('Backspace');
          }
          await page.waitForFunction(value => document.querySelector('#vba-source').textContent === value, expected);
        }
      }
    };
    await page.context().grantPermissions(['clipboard-read', 'clipboard-write']);
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
    // Fresh project for real editor typing, completion and correction (no test-only model mutation).
    await page.reload();
    await page.waitForFunction(() => document.querySelector('#vba-source').textContent.includes('Attribute VB_Name'));
    await page.locator('#vba-canvas').click({ position: { x: 500, y: 230 } });
    await page.keyboard.press('F7');
    await settle();
    await page.keyboard.press('Control+a');
    await settle();
    await typeSource('Sub Main()\n    Debug\nEnd Sub\n');
    await page.waitForFunction(() => document.querySelector('#vba-source').textContent === 'Sub Main()\n    Debug\nEnd Sub\n');
    await page.keyboard.press('Control+Home');
    await page.keyboard.press('ArrowDown');
    await page.keyboard.press('End');
    await page.keyboard.insertText('.');
    await page.waitForFunction(() => document.querySelector('#vba-source').textContent.includes('Debug.\n'));
    await page.screenshot({ path: path.join(report, 'browser-completion.png') });
    await page.keyboard.press('Enter');
    await page.keyboard.insertText(' 8');
    await page.waitForFunction(() => document.querySelector('#vba-source').textContent.includes('Debug.Print 8'));
    await page.waitForFunction(() => document.querySelector('#vba-feedback').textContent === 'Syntax OK');
    const typedSource = await page.locator('#vba-source').textContent();
    await page.keyboard.press('Control+a'); await settle();
    await page.keyboard.press('Control+c');
    await page.waitForFunction(async expected => await navigator.clipboard.readText() === expected, typedSource);
    await page.keyboard.press('Control+x');
    await page.waitForFunction(() => document.querySelector('#vba-source').textContent === '');
    await page.keyboard.press('Control+v');
    await page.waitForFunction(expected => document.querySelector('#vba-source').textContent === expected, typedSource);
    await page.keyboard.press('F5');
    await page.waitForFunction(() => document.querySelector('#vba-status').textContent === 'Completed');
    assert.match(await page.locator('#vba-output').textContent(), /8/);
    await page.keyboard.press('F7');
    await settle();
    await page.keyboard.press('Control+a');
    await settle();
    await typeSource('Sub Main()\nDim counter As Long\ncounter =\nEnd Sub\n');
    await page.waitForFunction(() => document.querySelector('#vba-feedback').textContent !== 'Syntax OK');
    await page.screenshot({ path: path.join(report, 'browser-syntax-error.png') });
    await page.keyboard.press('Control+a');
    await typeSource('Sub Main()\nDim counter As Long\ncounter = 3\nEnd Sub\n');
    await page.waitForFunction(() => document.querySelector('#vba-feedback').textContent === 'Syntax OK');
    assert.deepEqual(errors, []);
    await page.keyboard.press('Control+e');
    await settle();
    await page.waitForTimeout(250);
    await page.screenshot({path:path.join(report,'m10-browser-files.png')});
    const exported = await page.locator('#vba-source').textContent();
    const downloading=page.waitForEvent('download');
    await page.locator('#vba-canvas').click({position:{x:90,y:140}});
    const downloaded=await downloading;
    const chunks=[];for await(const chunk of await downloaded.createReadStream()) chunks.push(chunk);
    assert.equal(Buffer.concat(chunks).toString('utf8'),exported);
    assert.equal(downloaded.suggestedFilename(),'Module1.bas');
    await page.locator('#vba-canvas').click({position:{x:75,y:406}});
    await settle();
    await page.locator('#vba-canvas').click({position:{x:17,y:15}});
    await settle();await page.waitForTimeout(250);
    await page.screenshot({path:path.join(report,'m10-browser-file-menu.png')});
    await page.locator('#vba-canvas').click({position:{x:70,y:66}});
    await settle();await page.waitForTimeout(250);
    const choosing=page.waitForEvent('filechooser');
    await page.locator('#vba-canvas').click({position:{x:100,y:140}});
    const uploaded="' λ\r\nSub Main()\r\nDebug.Print 11\r\nEnd Sub\r\n";
    await (await choosing).setFiles({name:'AuthoredUpload.bas',mimeType:'text/plain',buffer:Buffer.from(uploaded,'utf8')});
    await page.waitForFunction(value=>document.querySelector('#vba-source').textContent===value,uploaded);
    assert.notEqual(await page.locator('#vba-status').textContent(),'Running');
    assert.doesNotMatch(await page.locator('#vba-output').textContent(),/11/);
    await page.screenshot({path:path.join(report,'m10-browser-upload.png')});
    assert.deepEqual(errors,[]);
    console.log('BROWSER_UI_PASS: F8/F5 debugger; completion/output; clipboard; syntax recovery; exact UTF-8 download and Unicode/CRLF file upload without execution; no page errors');
  } finally {
    if (browser) await browser.close();
    await new Promise(resolve => server.close(resolve));
  }
})().catch(error => { console.error(error); process.exitCode = 1; });
