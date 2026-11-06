import { get, writable } from 'svelte/store';
import type { Command, CommandCategory } from '../types/command';
import { commands } from '../commands/registry';
import { isNative, rememberFile } from '../native';

export interface Usage {
  count: number;
  lastUsed: number;
}
export interface SavedMetadata {
  label: string;
  category: CommandCategory;
}
export interface Personalization {
  version: 1;
  favorites: string[];
  recents: string[];
  aliases: Record<string, string[]>;
  usage: Record<string, Usage>;
  items: Record<string, SavedMetadata>;
}
export interface SavedItem extends SavedMetadata {
  key: string;
  command?: Command;
  available: boolean;
  favorite: boolean;
  aliases: string[];
  usage?: Usage;
}

export const personalizationStorageKey = 'mycast.personalization.v1';
const builtinKeys = new Set(
  commands.map((command) => command.personalizationKey),
);
const categories = new Set<CommandCategory>([
  'Applications',
  'Utilities',
  'Quick Actions',
  'Preferences',
  'Calculator',
  'Files',
  'Windows',
]);
const validKey = (key: unknown): key is string =>
  typeof key === 'string' &&
  (builtinKeys.has(key) ||
    /^windows:[a-z0-9-]+$/.test(key) ||
    /^app:v1:[a-f0-9]{64}$/.test(key) ||
    /^file:v1:[a-f0-9]{32}$/.test(key));
const aliasIdentity = (alias: string) => alias.normalize('NFC').toLowerCase();
const validAlias = (alias: unknown): alias is string =>
  typeof alias === 'string' &&
  [...alias].length > 0 &&
  [...alias].length <= 32 &&
  /\p{L}/u.test(alias) &&
  !/[\s\p{C}]/u.test(alias);
const empty = (): Personalization => ({
  version: 1,
  favorites: [],
  recents: [],
  aliases: {},
  usage: {},
  items: {},
});
const object = (value: unknown): value is Record<string, unknown> =>
  !!value && typeof value === 'object' && !Array.isArray(value);

/** Only durable item identities and curated metadata cross the storage boundary. */
export function validatePersonalization(value: unknown): Personalization {
  const result = empty();
  if (!object(value) || value.version !== 1) return result;
  const keys = (input: unknown, limit: number) =>
    Array.isArray(input)
      ? [...new Set(input.filter(validKey))].slice(0, limit)
      : [];
  result.favorites = keys(value.favorites, 10000);
  result.recents = keys(value.recents, 50);
  const taken = new Set<string>();
  if (object(value.aliases))
    for (const [key, aliases] of Object.entries(value.aliases).slice(
      0,
      10000,
    )) {
      if (!validKey(key) || !Array.isArray(aliases)) continue;
      const validated: string[] = [];
      for (const alias of aliases) {
        if (!validAlias(alias)) continue;
        const identity = aliasIdentity(alias);
        if (taken.has(identity)) continue;
        validated.push(alias.normalize('NFC'));
        taken.add(identity);
        if (validated.length === 10) break;
      }
      if (validated.length) result.aliases[key] = validated;
    }
  if (object(value.usage))
    for (const [key, usage] of Object.entries(value.usage).slice(0, 10000)) {
      if (
        validKey(key) &&
        object(usage) &&
        typeof usage.count === 'number' &&
        Number.isSafeInteger(usage.count) &&
        usage.count > 0 &&
        typeof usage.lastUsed === 'number' &&
        Number.isFinite(usage.lastUsed) &&
        usage.lastUsed > 0 &&
        usage.lastUsed <= Date.now() + 86400000
      ) {
        result.usage[key] = { count: usage.count, lastUsed: usage.lastUsed };
      }
    }
  result.recents = result.recents.filter((key) => !!result.usage[key]);
  const retained = new Set([
    ...result.favorites,
    ...result.recents,
    ...Object.keys(result.aliases),
    ...Object.keys(result.usage),
  ]);
  if (object(value.items))
    for (const key of retained) {
      const metadata = value.items[key];
      if (
        !object(metadata) ||
        typeof metadata.label !== 'string' ||
        !metadata.label.trim() ||
        typeof metadata.category !== 'string' ||
        !categories.has(metadata.category as CommandCategory)
      )
        continue;
      result.items[key] = {
        label: metadata.label.slice(0, 256).replace(/[\p{C}]/gu, ''),
        category: metadata.category as CommandCategory,
      };
    }
  return result;
}

function load(): Personalization {
  try {
    if (typeof localStorage !== 'undefined')
      return validatePersonalization(
        JSON.parse(localStorage.getItem(personalizationStorageKey) ?? 'null'),
      );
  } catch {
    /* Session personalization still works with unavailable storage. */
  }
  return empty();
}
const state = writable(load());
state.subscribe((value) => {
  try {
    if (typeof localStorage !== 'undefined')
      localStorage.setItem(personalizationStorageKey, JSON.stringify(value));
  } catch {
    /* Keep the session usable when storage is full or denied. */
  }
});
export const personalization = { subscribe: state.subscribe };

export function keyForCommand(command: Command): string | undefined {
  // Copy-result actions may contain expressions or clipboard values: never persist them.
  return command.action.type !== 'copy' && validKey(command.personalizationKey)
    ? command.personalizationKey
    : undefined;
}
function rememberItem(value: Personalization, command: Command | string): void {
  if (typeof command === 'string') return;
  if (command.action.type === 'file' && isNative())
    void rememberFile(command.action.fileId).catch(() => {});
  const key = keyForCommand(command);
  if (key)
    value.items = {
      ...value.items,
      [key]: { label: command.label, category: command.category },
    };
}
const resolveKey = (command: Command | string) =>
  typeof command === 'string'
    ? validKey(command)
      ? command
      : undefined
    : keyForCommand(command);

export function toggleFavorite(command: Command | string): void {
  const key = resolveKey(command);
  if (!key) return;
  state.update((value) => {
    rememberItem(value, command);
    return {
      ...value,
      favorites: value.favorites.includes(key)
        ? value.favorites.filter((item) => item !== key)
        : [...value.favorites, key],
    };
  });
}
export function moveFavorite(key: string, offset: number): void {
  state.update((value) => {
    const favorites = [...value.favorites];
    const from = favorites.indexOf(key);
    const to = Math.max(
      0,
      Math.min(favorites.length - 1, from + Math.trunc(offset)),
    );
    if (from < 0 || !Number.isFinite(to)) return value;
    favorites.splice(from, 1);
    favorites.splice(to, 0, key);
    return { ...value, favorites };
  });
}
export function setAliases(
  command: Command | string,
  aliases: string[],
): string | null {
  const key = resolveKey(command);
  if (!key) return 'This item cannot have aliases.';
  if (aliases.length > 10) return 'Use at most ten aliases per item.';
  if (aliases.some((alias) => !validAlias(alias)))
    return 'Aliases need a letter, at most 32 characters, and no spaces or control characters.';
  const normalized = aliases.map((alias) => alias.normalize('NFC'));
  const identities = normalized.map(aliasIdentity);
  if (new Set(identities).size !== identities.length)
    return 'Each alias must be unique (ignoring case).';
  const current = get(state);
  for (const [otherKey, existing] of Object.entries(current.aliases)) {
    if (
      otherKey !== key &&
      existing.some((alias) => identities.includes(aliasIdentity(alias)))
    )
      return 'An alias is already assigned to another item.';
  }
  state.update((value) => {
    rememberItem(value, command);
    const next = { ...value.aliases };
    if (normalized.length) next[key] = normalized;
    else delete next[key];
    return { ...value, aliases: next };
  });
  return null;
}
/** Call only after execution resolves successfully; no action result is accepted. */
export function recordSuccessfulUse(command: Command, now = Date.now()): void {
  const key = keyForCommand(command);
  if (!key || !Number.isFinite(now) || now <= 0) return;
  state.update((value) => {
    rememberItem(value, command);
    return {
      ...value,
      recents: [key, ...value.recents.filter((item) => item !== key)].slice(
        0,
        50,
      ),
      usage: {
        ...value.usage,
        [key]: {
          count: Math.min(
            Number.MAX_SAFE_INTEGER,
            (value.usage[key]?.count ?? 0) + 1,
          ),
          lastUsed: now,
        },
      },
    };
  });
}
export function clearHistory(): void {
  state.update((value) => ({
    ...value,
    recents: [],
    usage: {},
    items: Object.fromEntries(
      Object.entries(value.items).filter(
        ([key]) => value.favorites.includes(key) || value.aliases[key]?.length,
      ),
    ),
  }));
}
export function savedItems(
  catalogue: readonly Command[],
  value: Personalization,
): SavedItem[] {
  const available = new Map(
    catalogue.flatMap((command) => {
      const key = keyForCommand(command);
      return key ? [[key, command] as const] : [];
    }),
  );
  return [
    ...new Set([
      ...value.favorites,
      ...Object.keys(value.aliases),
      ...value.recents,
      ...Object.keys(value.usage),
      ...available.keys(),
    ]),
  ].map((key) => {
    const command = available.get(key);
    return {
      key,
      label: command?.label ?? value.items[key]?.label ?? 'Unavailable item',
      category:
        command?.category ?? value.items[key]?.category ?? 'Applications',
      command,
      available: !!command,
      favorite: value.favorites.includes(key),
      aliases: value.aliases[key] ?? [],
      usage: value.usage[key],
    };
  });
}
