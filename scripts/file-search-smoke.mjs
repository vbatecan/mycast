// Focused opt-in checks for a debug build with artifacts/file-search-smoke.conf.json.
// Temporarily selects an isolated fixture folder and restores preferences afterward.
import { chromium } from '@playwright/test';
import assert from 'node:assert/strict';
import { mkdir, writeFile, rename, unlink, readFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';
import { spawn } from 'node:child_process';

const executable = resolve('src-tauri/target/debug/mycast.exe');
const fixture = resolve(`artifacts/files-fixture-${Date.now()}`);
await mkdir(join(fixture, 'Pictures'), { recursive: true });
await Promise.all(
  ['Annual Report 2026.txt', 'Résumé notes.txt', 'Budget final.txt'].map(
    (name) =>
      writeFile(join(fixture, name), `Mycast filename fixture: ${name}`),
  ),
);
await writeFile(join(fixture, 'Pictures', 'Photo notes.txt'), 'fixture');
let browser = await chromium.connectOverCDP('http://127.0.0.1:9223');
let page = browser.contexts()[0].pages()[0];
page.setDefaultTimeout(10000);
const errors = [];
page.on('pageerror', (error) => errors.push(error.message));
const invoke = (command, args = {}) =>
  page.evaluate(
    ({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args),
    { command, args },
  );
const pause = (ms) => new Promise((done) => setTimeout(done, ms));
async function until(action, predicate, description) {
  const start = Date.now();
  while (Date.now() - start < 30000) {
    const result = await action();
    if (predicate(result)) return result;
    await pause(100);
  }
  throw new Error(`Timed out: ${description}`);
}
let serial = 0;
const search = (query, options = {}) =>
  invoke('search_files', {
    options: {
      query,
      requestId: Date.now() * 1000 + ++serial,
      browse: true,
      now: Date.now(),
      ...options,
    },
  });
const original = await invoke('get_file_status');
const originalPersonalization = await page.evaluate(() =>
  localStorage.getItem('mycast.personalization.v1'),
);
let changedPreferences = false;
try {
  assert.equal(
    original.state,
    'idle',
    'background launch defers file indexing',
  );
  const revision = original.revision;
  await invoke('set_file_preferences', {
    preferences: {
      ...original.preferences,
      roots: [fixture],
      terminalDirectory: fixture,
    },
  });
  changedPreferences = true;
  await until(
    () => invoke('get_file_status'),
    (state) => state.state === 'ready' && state.revision > revision,
    'initial scan',
  );
  const initial = await search('rpt');
  assert.equal(initial.items[0].name, 'Annual Report 2026.txt');
  const key = initial.items[0].persistentKey;
  const id = initial.items[0].id;
  const alias = await search('quickdoc', {
    aliases: { [key]: ['quickdoc', 'anotherdoc'] },
  });
  assert.equal(alias.items[0].persistentKey, key);
  assert.equal(alias.items[0].relevance, 1100);
  assert.equal((await search('resume')).items[0].name, 'Résumé notes.txt');
  await invoke('remember_file', { id });
  await pause(700);
  await rename(
    join(fixture, 'Annual Report 2026.txt'),
    join(fixture, 'Annual Report renamed.txt'),
  );
  const renamed = await until(
    () => search('renamed'),
    (result) =>
      result.items.some((item) => item.name === 'Annual Report renamed.txt'),
    'rename event',
  );
  assert.equal(
    renamed.items[0].persistentKey,
    key,
    'rename preserves personalized identity',
  );
  await writeFile(join(fixture, 'Created after scan.txt'), 'created');
  await until(
    () => search('Created after scan'),
    (result) =>
      result.items.some((item) => item.name === 'Created after scan.txt'),
    'create event',
  );
  await unlink(join(fixture, 'Created after scan.txt'));
  await until(
    () => search('Created after scan'),
    (result) =>
      !result.items.some((item) => item.name === 'Created after scan.txt'),
    'delete event',
  );
  await assert.rejects(invoke('file_action', { id: 'forged', action: 'open' }));
  // Refresh clears native generation overrides from the API probes before UI work.
  const second = spawn(executable, [], { windowsHide: true, stdio: 'ignore' });
  await new Promise((done) => second.on('exit', done));
  await page.reload();
  await page.getByRole('combobox').fill('Budget final');
  await until(
    () => page.getByRole('option').allTextContents(),
    (rows) => rows.some((row) => row.includes('Budget final.txt')),
    'All search includes files',
  );
  await page.getByRole('button', { name: 'Files', exact: true }).click();
  await page.getByRole('combobox').press('Control+d');
  assert.match(
    await page.getByRole('option', { selected: true }).innerText(),
    /Budget final.txt/,
  );
  await page.getByRole('button', { name: 'Edit aliases', exact: true }).click();
  await page.getByRole('textbox').fill('budgetprobe\nfinanceprobe');
  await page.getByRole('button', { name: 'Save aliases', exact: true }).click();
  await page.reload();
  await page.getByRole('combobox').fill('financeprobe');
  await until(
    () => page.getByRole('option', { selected: true }).allTextContents(),
    (rows) => rows.some((row) => row.includes('Budget final.txt')),
    'file aliases persist',
  );
  const favorite = await page.evaluate(() => {
    const state = JSON.parse(localStorage.getItem('mycast.personalization.v1'));
    return state.favorites.find(
      (key) => state.items[key]?.label === 'Budget final.txt',
    );
  });
  assert.ok(favorite);
  await invoke('quit_launcher').catch(() => {});
  await browser.close().catch(() => {});
  await pause(400);
  await rename(
    join(fixture, 'Budget final.txt'),
    join(fixture, 'Budget final renamed.txt'),
  );
  spawn(executable, ['--background'], {
    windowsHide: true,
    stdio: 'ignore',
  }).unref();
  browser = await until(
    () => chromium.connectOverCDP('http://127.0.0.1:9223').catch(() => null),
    (value) => !!value,
    'restart WebView',
  );
  page = browser.contexts()[0].pages()[0];
  page.setDefaultTimeout(10000);
  page.on('pageerror', (error) => errors.push(error.message));
  await page.waitForFunction(() => Boolean(window.__TAURI_INTERNALS__));
  assert.equal((await invoke('get_file_status')).state, 'idle');
  await until(
    () => invoke('resolve_files', { keys: [favorite] }),
    (items) => items.some((item) => item.persistentKey === favorite),
    'durable file identity after restart',
  );
  const reopened = spawn(executable, [], {
    windowsHide: true,
    stdio: 'ignore',
  });
  await new Promise((done) => reopened.on('exit', done));
  await page.getByRole('combobox').fill('financeprobe');
  await until(
    () => page.getByRole('option', { selected: true }).allTextContents(),
    (rows) => rows.some((row) => row.includes('Budget final renamed.txt')),
    'aliases after process restart',
  );
  await page.screenshot({
    path: 'artifacts/file-search-results.png',
    animations: 'disabled',
  });
  await page.getByRole('combobox').fill('> Write-Output "Mycast probe"');
  assert.match(await page.getByRole('option').innerText(), /Run Write-Output/);
  assert.equal(
    await page.getByRole('button', { name: 'Pin', exact: true }).count(),
    0,
  );
  await page.screenshot({
    path: 'artifacts/run-mode.png',
    animations: 'disabled',
  });
  // Launch a visible terminal, check quoting/Unicode/cwd through an owned output file, then exit it.
  const output = join(fixture, 'terminal-probe.json');
  const script = `@{ cwd = $PWD.Path; text = 'quotes " ; & café 文档' } | ConvertTo-Json | Set-Content -LiteralPath '${output.replaceAll("'", "''")}' -Encoding utf8; exit`;
  await invoke('run_shell_command', { command: script });
  const terminal = await until(
    () => readFile(output, 'utf8').catch(() => ''),
    (text) => text.length > 0,
    'visible terminal execution',
  );
  const parsed = JSON.parse(terminal.replace(/^\uFEFF/, ''));
  assert.equal(parsed.cwd.toLowerCase(), fixture.toLowerCase());
  assert.equal(parsed.text, 'quotes " ; & café 文档');
  assert.deepEqual(errors, []);
  console.log(
    JSON.stringify({
      passed: [
        'hidden startup',
        'fuzzy and Unicode',
        'multiple aliases',
        'rename identity',
        'create/delete updates',
        'opaque ID rejection',
        'All/Files UI',
        'pin selection',
        'alias refresh and process-restart persistence',
        '> mode',
        'terminal quoting and cwd',
      ],
      initialSearchMs: initial.elapsedMs,
    }),
  );
} finally {
  await page
    .evaluate((value) => {
      if (value === null) localStorage.removeItem('mycast.personalization.v1');
      else localStorage.setItem('mycast.personalization.v1', value);
    }, originalPersonalization)
    .catch(() => {});
  if (changedPreferences) {
    const before = await invoke('get_file_status');
    await invoke('set_file_preferences', { preferences: original.preferences });
    await until(
      () => invoke('get_file_status'),
      (state) => state.state === 'ready' && state.revision > before.revision,
      'restore original folders',
    );
  }
  await browser.close();
}
