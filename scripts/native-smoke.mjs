// Opt-in Windows integration smoke test. Opens a blank Notepad window.
// Uses WebView2's test-only debugging port; ordinary launches do not enable it.
import { chromium } from '@playwright/test';
import { spawn, execFileSync } from 'node:child_process';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import assert from 'node:assert/strict';

const debugPort = 9223;
if (process.platform !== 'win32') throw new Error('Native smoke testing requires Windows.');
if (!process.env.MYCAST_SMOKE_EXECUTABLE) {
  // Explicit options work for elevated test hosts that ignore WEBVIEW2_* env.
  // This override builds only the debug target; production config stays intact.
  await mkdir('artifacts', { recursive: true });
  const config = JSON.parse(await readFile('src-tauri/tauri.conf.json', 'utf8'));
  const override = { app: { windows: config.app.windows.map((window) => ({
    ...window, additionalBrowserArgs: `--remote-debugging-port=${debugPort}`, devtools: true,
  })) } };
  await writeFile('artifacts/native-smoke.conf.json', JSON.stringify(override));
  execFileSync(process.env.ComSpec || 'cmd.exe', ['/d', '/s', '/c', 'npm run tauri build -- --debug --no-bundle --config artifacts/native-smoke.conf.json'], {
    stdio: 'inherit', windowsHide: true,
  });
}
const executable = resolve(process.env.MYCAST_SMOKE_EXECUTABLE || 'src-tauri/target/debug/mycast.exe');
console.log('Native test executable:', executable);
const launcher = spawn(executable, [], {
  windowsHide: false,
  stdio: ['ignore', 'pipe', 'pipe'],
  env: {
    ...process.env,
    WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${debugPort}`,
  },
});
launcher.on('error', (error) => {
  throw error;
});
launcher.stdout.on('data', (data) => process.stdout.write(data));
launcher.stderr.on('data', (data) => process.stderr.write(data));
let browser;
let page;
const pause = (ms) => new Promise((done) => setTimeout(done, ms));
const invoke = (command, args = {}) =>
  page.evaluate(
    ({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args),
    { command, args },
  );
const clipboard = () =>
  execFileSync(
    'powershell.exe',
    ['-NoProfile', '-Command', 'Get-Clipboard -Raw'],
    { encoding: 'utf8', windowsHide: true },
  ).trim();
const reopen = async () => {
  const second = spawn(executable, [], { windowsHide: false, stdio: 'ignore' });
  await new Promise((done, reject) => {
    second.on('error', reject);
    second.on('exit', done);
  });
  await page.getByRole('combobox').waitFor();
};

try {
  for (let attempt = 0; attempt < 240; attempt++) {
    try {
      browser = await chromium.connectOverCDP(`http://127.0.0.1:${debugPort}`);
      break;
    } catch {
      if (launcher.exitCode !== null)
        throw new Error('Launcher exited before WebView2 initialized');
      await pause(250);
    }
  }
  assert.ok(browser, 'WebView2 debugging endpoint started');
  page = browser.contexts()[0].pages()[0];
  await page.getByRole('combobox').waitFor();
  const status = await invoke('get_launcher_status');
  assert.ok(
    status.shortcut || status.shortcutError,
    'Native shortcut registration has an honest outcome',
  );
  console.log('Native status:', JSON.stringify(status));
  assert.equal(
    await page
      .getByRole('combobox')
      .evaluate((input) => input === document.activeElement),
    true,
  );

  await page.getByRole('combobox').fill('(50 + 20) / 2');
  // The automation host can steal native foreground focus. Dispatch the DOM
  // handler explicitly here; normal keyboard input is covered by browser tests.
  await page.getByRole('combobox').dispatchEvent('keydown', { key: 'Enter', bubbles: true });
  await page.getByRole('status').filter({ hasText: 'Copied' }).waitFor();
  assert.equal(clipboard(), '35');

  await invoke('execute_native', { action: 'copy-date' });
  assert.match(clipboard(), /^\d{4}-\d{2}-\d{2}$/);
  await invoke('execute_native', { action: 'copy-time' });
  assert.match(clipboard(), /^\d{2}:\d{2}:\d{2}$/);
  await invoke('execute_native', { action: 'copy-uuid' });
  assert.match(
    clipboard(),
    /^[a-f0-9]{8}-[a-f0-9]{4}-4[a-f0-9]{3}-[89ab][a-f0-9]{3}-[a-f0-9]{12}$/,
  );
  console.log('Calculator and date/time/UUID clipboard actions passed.');
  await assert.rejects(
    invoke('execute_native', { action: 'arbitrary-executable' }),
  );
  await assert.rejects(invoke('copy_text', { text: 'x'.repeat(4097) }));
  await assert.rejects(invoke('plugin:clipboard-manager|read_text'));
  console.log(
    'Native action allowlist, clipboard bound, and denied read permission passed.',
  );

  await page.getByRole('combobox').fill('');
  await mkdir('artifacts', { recursive: true });
  await page.screenshot({
    path: 'artifacts/native-launcher.png',
    animations: 'disabled',
  });
  console.log('Application process tree (automation attached):');
  console.log(
    execFileSync(
      'powershell.exe',
      [
        '-NoProfile',
        '-File',
        'scripts/measure-memory.ps1',
        '-LauncherProcessId',
        String(launcher.pid),
        '-State',
        'automation-before-hide-request',
      ],
      { encoding: 'utf8', windowsHide: true },
    ),
  );

  await page.getByRole('combobox').dispatchEvent('keydown', { key: 'Escape', bubbles: true });
  await pause(200);
  console.log(
    'Visibility after Escape:',
    await page.evaluate(() => document.visibilityState),
  );
  console.log(
    execFileSync(
      'powershell.exe',
      [
        '-NoProfile',
        '-File',
        'scripts/measure-memory.ps1',
        '-LauncherProcessId',
        String(launcher.pid),
        '-State',
        'automation-after-hide-request',
      ],
      { encoding: 'utf8', windowsHide: true },
    ),
  );
  await reopen();
  assert.equal(
    launcher.exitCode,
    null,
    'Second instance reuses original process',
  );
  assert.equal(
    await page
      .getByRole('combobox')
      .evaluate((input) => input === document.activeElement),
    true,
  );

  const opened = await invoke('execute_native', { action: 'notepad' });
  assert.match(opened.message, /Notepad/);
  console.log('Approved Notepad dispatch:', opened.message);
  await reopen();
  console.log(
    'Native smoke test passed; global physical keypress and visual tray interaction still require manual desktop QA.',
  );
} finally {
  try {
    if (page) await invoke('quit_launcher');
  } catch {
    /* Process exit can close the IPC response. */
  }
  if (browser) await browser.close().catch(() => {});
  if (launcher.exitCode === null) launcher.kill();
}
