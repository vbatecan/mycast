import test from 'node:test';
import assert from 'node:assert/strict';
import {
  defaultShortcuts,
  eventShortcut,
  validateShortcuts,
  validShortcut,
} from '../src/lib/shortcuts';

test('default filters map to Alt+Shift+1–6 and use physical number keys', () => {
  const scopes = [
    'all',
    'applications',
    'commands',
    'files',
    'favorites',
    'recents',
  ] as const;
  scopes.forEach((scope, index) => {
    const binding = eventShortcut({
      key: '!',
      code: `Digit${index + 1}`,
      ctrlKey: false,
      altKey: true,
      shiftKey: true,
      metaKey: false,
    });
    assert.equal(binding, defaultShortcuts[scope]);
  });
});

test('shortcut validation preserves typing, rejects duplicate storage, and migrates defaults', () => {
  assert.equal(validShortcut('A'), false);
  assert.equal(validShortcut('Ctrl+C'), false);
  assert.equal(validShortcut('Alt+Tab'), false);
  assert.equal(validShortcut('Ctrl+Shift+Q'), true);
  assert.deepEqual(validateShortcuts(undefined), defaultShortcuts);
  assert.deepEqual(validateShortcuts({ all: 'Alt+Shift+2' }), defaultShortcuts);
  assert.equal(validateShortcuts({ all: 'Ctrl+Shift+Q' }).all, 'Ctrl+Shift+Q');
});
