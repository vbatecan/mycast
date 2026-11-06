import test from 'node:test';
import assert from 'node:assert/strict';
import { get } from 'svelte/store';
import { commands } from '../src/lib/commands/registry';
import {
  searchCommands,
  homeSection,
  learnedScore,
} from '../src/lib/commands/search';
import { mergeCatalogue } from '../src/lib/stores/applications';
import {
  personalization,
  validatePersonalization,
  keyForCommand,
  toggleFavorite,
  setAliases,
  recordSuccessfulUse,
  clearHistory,
  savedItems,
} from '../src/lib/stores/personalization';
import type { Command } from '../src/lib/types/command';

const now = 1800000000000;
function app(index: number, label = `Tool ${index}`): Command {
  return {
    id: `session-${index}`,
    personalizationKey: `app:v1:${index.toString(16).padStart(64, '0')}`,
    label,
    description: '',
    category: 'Applications',
    icon: 'app-window',
    keywords: [],
    action: { type: 'application', applicationId: `opaque-${index}` },
  };
}

test('relevance precedes pins and learning; aliases precede exact labels', () => {
  const catalogue = [
    app(1, 'Open Editor'),
    app(2, 'Editor'),
    app(3, 'Editor Plus'),
    app(4, 'Other'),
  ];
  const key = (index: number) => catalogue[index].personalizationKey!;
  const options = {
    now,
    favorites: [key(0)],
    aliases: { [key(3)]: ['EDITOR'] },
    usage: {
      [key(0)]: { count: 10000, lastUsed: now },
      [key(2)]: { count: 1000, lastUsed: now },
    },
  };
  assert.deepEqual(
    searchCommands('editor', catalogue, 40, options).map((item) => item.id),
    ['session-4', 'session-2', 'session-3', 'session-1'],
  );
  const peers = [
    app(5, 'Sample First'),
    app(6, 'Sample Second'),
    app(7, 'Sample Third'),
  ];
  const peersOptions = {
    now,
    favorites: [peers[2].personalizationKey!],
    usage: { [peers[1].personalizationKey!]: { count: 3, lastUsed: now } },
  };
  assert.deepEqual(
    searchCommands('sample', peers, 40, peersOptions).map((item) => item.id),
    ['session-7', 'session-6', 'session-5'],
  );
  assert.equal(learnedScore({ count: 3, lastUsed: now }, now), 6);
});

test('home leads with five recents, deduplicates pins, and caps the Recents filter', () => {
  const catalogue = Array.from({ length: 60 }, (_, index) => app(index + 100));
  const favorites = catalogue
    .slice(0, 12)
    .map((item) => item.personalizationKey!);
  const recents = catalogue
    .slice(4, 54)
    .map((item) => item.personalizationKey!);
  const options = { favorites, recents, now };
  const results = searchCommands('', catalogue, 40, options);
  assert.equal(results.length, 40);
  assert.equal(
    new Set(results.map((item) => item.personalizationKey)).size,
    results.length,
  );
  assert.deepEqual(results.slice(0, 5), catalogue.slice(4, 9));
  assert.equal(homeSection(results[0], catalogue, options), 'Recents');
  assert.equal(homeSection(results[5], catalogue, options), 'Favorites');
  assert.equal(homeSection(results[12], catalogue, options), 'Suggested');
  assert.equal(
    searchCommands('Tool 111', catalogue, 40, {
      ...options,
      collection: 'favorites',
    })[0]?.id,
    'session-111',
  );
  assert.equal(
    searchCommands('Tool 153', catalogue, 40, {
      ...options,
      collection: 'recents',
    })[0]?.id,
    undefined,
  );
  assert.equal(
    searchCommands('', catalogue, 40, { ...options, collection: 'recents' })
      .length,
    5,
  );
  assert.equal(
    searchCommands('', catalogue, 40, {
      ...options,
      collection: 'recents',
      recentLimit: 10,
    }).length,
    10,
  );
  assert.equal(
    homeSection(catalogue[13], catalogue, { ...options, recentLimit: 10 }),
    'Recents',
  );
});

test('recent programs and folders lead without preferences or clipboard actions', () => {
  const folder: Command = {
    ...app(701),
    action: { type: 'file', fileId: 'folder-id', kind: 'folder' },
  };
  const program = app(702);
  const about = commands.find((item) => item.action.type === 'view')!;
  const copy = commands.find(
    (item) =>
      item.action.type === 'native' && item.action.name.startsWith('copy-'),
  )!;
  const catalogue = [about, copy, program, folder];
  const options = {
    recents: catalogue.map((item) => item.personalizationKey!),
    favorites: [folder.personalizationKey!],
  };
  const results = searchCommands('', catalogue, 40, options);
  assert.deepEqual(results.slice(0, 2), [program, folder]);
  assert.deepEqual(
    searchCommands('', catalogue, 40, { ...options, collection: 'recents' }),
    [program, folder],
  );
  assert.equal(new Set(results.map((item) => item.id)).size, results.length);
});

test('aliases reject arithmetic, whitespace, duplicates, cross-item conflicts, and excessive lists', () => {
  const first = app(201),
    second = app(202);
  assert.equal(setAliases(first, ['work', 'write']), null);
  assert.match(setAliases(second, ['WORK'])!, /another item/);
  assert.match(setAliases(second, ['a', 'A'])!, /unique/);
  assert.match(setAliases(second, ['2+2'])!, /letter/);
  assert.match(setAliases(second, ['two words'])!, /spaces/);
  assert.match(setAliases(second, ['abc\u0000'])!, /control/);
  assert.match(setAliases(second, ['a'.repeat(33)])!, /32/);
  assert.match(
    setAliases(
      second,
      Array.from({ length: 11 }, (_, index) => `alias${index}`),
    )!,
    /ten/,
  );
  assert.equal(setAliases(first, []), null);
});

test('fallback personalization survives catalogue replacement and saved unavailable items stay manageable', () => {
  const fallback = commands.find((command) => command.id === 'notepad')!;
  const installed = app(301, 'Notepad');
  const merged = mergeCatalogue([installed]);
  assert.equal(merged[0].personalizationKey, fallback.personalizationKey);
  assert.equal(
    merged.some((command) => command.id === fallback.id),
    false,
  );
  toggleFavorite(fallback);
  assert.equal(setAliases(fallback, ['notes-test']), null);
  const snapshot = validatePersonalization(
    JSON.parse(JSON.stringify(get(personalization))),
  );
  assert.equal(
    snapshot.favorites.includes(merged[0].personalizationKey!),
    true,
  );
  assert.deepEqual(searchCommands('notes-test', merged, 40, snapshot), [
    merged[0],
  ]);
  const unavailable = savedItems([], snapshot).find(
    (item) => item.key === fallback.personalizationKey,
  )!;
  assert.equal(unavailable.label, 'Open Notepad');
  assert.equal(unavailable.available, false);
  toggleFavorite(fallback);
  setAliases(fallback, []);
});

test('successful uses retain fifty distinct recents and clear history preserves pins and aliases', () => {
  clearHistory();
  const catalogue = Array.from({ length: 55 }, (_, index) => app(index + 400));
  const pinned = catalogue[0];
  toggleFavorite(pinned);
  setAliases(pinned, ['pinned-test']);
  for (const command of catalogue) recordSuccessfulUse(command, now);
  recordSuccessfulUse(catalogue[54], now + 1);
  const snapshot = get(personalization);
  assert.equal(snapshot.recents.length, 50);
  assert.equal(new Set(snapshot.recents).size, 50);
  assert.equal(snapshot.usage[catalogue[54].personalizationKey!].count, 2);
  // An execution failure never calls recordSuccessfulUse; its identity has no usage.
  assert.equal(snapshot.usage[app(999).personalizationKey!], undefined);
  const transient: Command = {
    ...pinned,
    id: 'calculation',
    label: '2+2',
    action: { type: 'copy', text: '4' },
  };
  assert.equal(keyForCommand(transient), undefined);
  recordSuccessfulUse(transient, now);
  clearHistory();
  assert.deepEqual(get(personalization).recents, []);
  assert.deepEqual(get(personalization).usage, {});
  assert.equal(
    get(personalization).favorites.includes(pinned.personalizationKey!),
    true,
  );
  assert.deepEqual(get(personalization).aliases[pinned.personalizationKey!], [
    'pinned-test',
  ]);
  toggleFavorite(pinned);
  setAliases(pinned, []);
});

test('storage validation excludes session launch IDs and malformed or conflicted entries', () => {
  const first = app(601),
    second = app(602);
  const keyA = first.personalizationKey!,
    keyB = second.personalizationKey!;
  const snapshot = validatePersonalization({
    version: 1,
    favorites: ['session-1', keyA, keyA],
    recents: [keyA],
    aliases: { [keyA]: ['editor', '2+2'], [keyB]: ['EDITOR', 'notes'] },
    usage: { [keyA]: { count: -1, lastUsed: now } },
    items: { [keyA]: { label: 'Saved editor', category: 'Applications' } },
  });
  assert.deepEqual(snapshot.favorites, [keyA]);
  assert.deepEqual(snapshot.recents, []);
  assert.deepEqual(snapshot.aliases[keyA], ['editor']);
  assert.deepEqual(snapshot.aliases[keyB], ['notes']);
  assert.deepEqual(validatePersonalization({ version: 2 }), {
    version: 1,
    favorites: [],
    recents: [],
    aliases: {},
    usage: {},
    items: {},
  });
});
