import test from 'node:test';
import assert from 'node:assert/strict';
import { searchCommands } from '../src/lib/commands/search';
import { commands } from '../src/lib/commands/registry';
import type { Command } from '../src/lib/types/command';

function command(
  id: string,
  label: string,
  description = '',
  keywords: string[] = [],
): Command {
  return {
    id,
    label,
    description,
    keywords,
    category: 'Utilities',
    icon: 'equal',
    action: { type: 'calculator-mode' },
  };
}

test('exact and prefix labels rank above substring and fuzzy matches', () => {
  const registry = [
    command('fuzzy', 'C a l c'),
    command('substring', 'Open Calc'),
    command('prefix', 'Calculator'),
    command('exact', 'Calc'),
  ];
  assert.deepEqual(
    searchCommands('calc', registry).map((item) => item.id),
    ['exact', 'prefix', 'substring', 'fuzzy'],
  );
});

test('search is insensitive to case and accents and includes description and keywords', () => {
  const registry = [
    command('one', 'Café'),
    command('two', 'Utility', 'A fresh page'),
    command('three', 'Tool', '', ['unique identifier']),
  ];
  assert.equal(searchCommands('CAFE', registry)[0]?.id, 'one');
  assert.equal(searchCommands('fresh page', registry)[0]?.id, 'two');
  assert.equal(searchCommands('unique identifier', registry)[0]?.id, 'three');
  assert.equal(searchCommands(' ntpd ')[0]?.id, 'notepad');
});

test('multiword searches can span metadata and nonmatches are empty', () => {
  assert.equal(searchCommands('notepad text')[0]?.id, 'notepad');
  assert.deepEqual(searchCommands('zzzznoresults'), []);
});

test('default registry has the required commands and deterministic empty-query order', () => {
  assert.equal(commands.length, 14);
  assert.deepEqual(searchCommands(''), commands);
  assert.equal(
    searchCommands('settings').filter((item) => item.label === 'Open Settings')
      .length,
    2,
  );
});

test('hundreds of commands stay bounded to forty visible results', () => {
  const registry = Array.from({ length: 500 }, (_, index) =>
    command(`sample-${index}`, `Sample command ${index}`),
  );
  assert.equal(searchCommands('', registry).length, 40);
  assert.equal(searchCommands('sample', registry, 500).length, 40);
  assert.equal(searchCommands('sample', registry, 10).length, 10);
  assert.equal(
    searchCommands('sample command 321', registry)[0]?.id,
    'sample-321',
  );
  assert.equal(searchCommands('sample', registry, 0).length, 0);
});
