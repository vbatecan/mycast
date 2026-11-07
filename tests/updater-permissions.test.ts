import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

test('the main window can check, install, and close updater resources', () => {
  const capability = JSON.parse(
    readFileSync(
      new URL('../src-tauri/capabilities/default.json', import.meta.url),
      'utf8',
    ),
  );
  assert.ok(capability.windows.includes('main'));
  for (const permission of [
    'updater:default',
    'core:resources:allow-close',
    'process:allow-restart',
  ]) {
    assert.ok(
      capability.permissions.includes(permission),
      `Missing ${permission}`,
    );
  }
});
