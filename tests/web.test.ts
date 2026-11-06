import test from 'node:test';
import assert from 'node:assert/strict';
import { validSearchTemplate, webResult } from '../src/lib/commands/web';
import { get } from 'svelte/store';
import { preferences } from '../src/lib/stores/preferences';
import { keyForCommand } from '../src/lib/stores/personalization';
test('web queries are encoded and never personalized', () => {
  assert.deepEqual(webResult(' ?  ', 'google', ''), []);
  const result = webResult('? café & #?', 'bing', '')[0];
  assert.equal(result.action.type, 'web-search');
  if (result.action.type === 'web-search')
    assert.equal(new URL(result.action.url).searchParams.get('q'), 'café & #?');
  assert.equal(keyForCommand(result), undefined);
  assert.match(
    webResult('? hello', 'custom', 'https://example.com/?q={query}')[0].label,
    /Custom/,
  );
});
test('templates reject unsafe protocols, credentials and missing or duplicate placeholders', () => {
  assert.ok(validSearchTemplate('https://example.com/?q={query}'));
  for (const template of [
    'javascript:{query}',
    'file:///{query}',
    'https://example.com',
    'https://example.com/{query}?q={query}',
    'https://user:pass@example.com/?q={query}',
  ])
    assert.equal(validSearchTemplate(template), false);
});
test('old preferences receive search defaults', () => {
  preferences.set({ animations: false } as never);
  assert.equal(get(preferences).searchEngine, 'google');
  assert.ok(validSearchTemplate(get(preferences).customSearchUrl));
  preferences.reset();
});

test('Windows results support synonyms, favorites, and aliases', async () => {
  const { searchCommands } = await import('../src/lib/commands/search');
  const { setAliases, toggleFavorite, personalization } =
    await import('../src/lib/stores/personalization');
  const command: import('../src/lib/types/command').Command = {
    id: 'windows-installed-apps',
    personalizationKey: 'windows:installed-apps',
    label: 'Installed apps',
    description: 'Windows Settings',
    category: 'Windows',
    module: 'windows',
    icon: 'settings-2',
    keywords: ['add or remove programs', 'uninstall apps'],
    action: { type: 'windows', id: 'installed-apps' },
  };
  assert.equal(searchCommands('add or remove programs', [command])[0], command);
  assert.equal(setAliases(command, ['removeapps']), null);
  toggleFavorite(command);
  const state = get(personalization);
  assert.ok(state.favorites.includes('windows:installed-apps'));
  assert.equal(searchCommands('removeapps', [command], 40, state)[0], command);
});

test('animation speed validates and migrates', () => {
  preferences.set({ animationSpeed: 99 } as never);
  assert.equal(get(preferences).animationSpeed, 2);
  preferences.set({ animationSpeed: 0 } as never);
  assert.equal(get(preferences).animationSpeed, 0.25);
  preferences.set({ animationSpeed: NaN } as never);
  assert.equal(get(preferences).animationSpeed, 1);
  preferences.reset();
});
