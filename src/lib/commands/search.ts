import type { Command } from '../types/command';
import { commands } from './registry';
import type { Usage } from '../stores/personalization';

export interface SearchOptions {
  favorites?: readonly string[];
  recents?: readonly string[];
  recentLimit?: number;
  aliases?: Readonly<Record<string, readonly string[]>>;
  usage?: Readonly<Record<string, Usage>>;
  /** One timestamp per result computation keeps all comparisons consistent. */
  now?: number;
  collection?: 'all' | 'favorites' | 'recents';
}

interface SearchEntry {
  label: string;
  description: string;
  keywords: string[];
  words: string[];
}

const metadata = new WeakMap<Command, SearchEntry>();
const normalize = (text: string) =>
  text
    .normalize('NFKD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase()
    .trim();

function entryFor(command: Command): SearchEntry {
  let entry = metadata.get(command);
  if (!entry) {
    const label = normalize(command.label);
    const description = normalize(command.description);
    const keywords = command.keywords.map(normalize);
    entry = {
      label,
      description,
      keywords,
      words: `${label} ${keywords.join(' ')}`.split(/\s+/),
    };
    metadata.set(command, entry);
  }
  return entry;
}

/** Small subsequence matcher; reward contiguous characters and word starts. */
function fuzzyScore(query: string, candidate: string): number {
  if (query.length < 2 || query.length > candidate.length) return 0;
  let cursor = 0;
  let previous = -2;
  let score = 0;
  for (const character of query) {
    const position = candidate.indexOf(character, cursor);
    if (position === -1) return 0;
    score +=
      2 +
      (position === previous + 1 ? 3 : 0) +
      (position === 0 || candidate[position - 1] === ' ' ? 3 : 0);
    previous = position;
    cursor = position + 1;
  }
  return Math.min(99, score);
}

function scoreEntry(query: string, entry: SearchEntry): number {
  if (entry.label === query) return 1000;
  if (entry.label.startsWith(query)) return 900;
  if (entry.keywords.includes(query)) return 850;
  if (entry.words.some((word) => word.startsWith(query))) return 800;
  if (entry.label.includes(query)) return 700;
  if (entry.keywords.some((keyword) => keyword.includes(query))) return 600;
  if (entry.description.includes(query)) return 500;

  const tokens = query.split(/\s+/);
  if (tokens.length > 1) {
    const haystack = `${entry.label} ${entry.description} ${entry.keywords.join(' ')}`;
    if (tokens.every((token) => haystack.includes(token))) return 400;
  }
  return Math.max(
    fuzzyScore(query, entry.label),
    ...entry.keywords.map((keyword) => fuzzyScore(query, keyword)),
  ) > 0
    ? 100
    : 0;
}

export function learnedScore(usage: Usage | undefined, now: number): number {
  if (
    !usage ||
    !Number.isFinite(usage.count) ||
    usage.count <= 0 ||
    !Number.isFinite(usage.lastUsed)
  )
    return 0;
  const daysSinceLastUse = Math.max(0, (now - usage.lastUsed) / 86400000);
  return Math.log2(1 + usage.count) + 4 / (1 + daysSinceLastUse);
}

const identity = (command: Command) => command.personalizationKey ?? command.id;
function collectionItems(
  registry: readonly Command[],
  keys: readonly string[],
): Command[] {
  const byKey = new Map(
    registry.map((command) => [identity(command), command]),
  );
  return [...new Set(keys)].flatMap((key) => {
    const command = byKey.get(key);
    return command ? [command] : [];
  });
}

function recentItems(
  registry: readonly Command[],
  options: SearchOptions,
): Command[] {
  const limit =
    typeof options.recentLimit === 'number' &&
    Number.isFinite(options.recentLimit)
      ? Math.min(20, Math.max(1, Math.round(options.recentLimit)))
      : 5;
  return collectionItems(registry, options.recents ?? [])
    .filter(
      (command) =>
        command.action.type === 'application' ||
        command.action.type === 'file' ||
        (command.action.type === 'native' &&
          !command.action.name.startsWith('copy-')),
    )
    .slice(0, limit);
}

/** Recents lead the home view; each item belongs to exactly one section. */
export function homeSection(
  command: Command,
  registry: readonly Command[],
  options: SearchOptions,
): 'Favorites' | 'Recents' | 'Suggested' {
  const recents = recentItems(registry, options);
  if (recents.some((item) => identity(item) === identity(command)))
    return 'Recents';
  const recentKeys = new Set(recents.map(identity));
  const favorites = collectionItems(registry, options.favorites ?? [])
    .filter((item) => !recentKeys.has(identity(item)))
    .slice(0, 8);
  const favoriteKeys = new Set(favorites.map(identity));
  if (favoriteKeys.has(identity(command))) return 'Favorites';
  return 'Suggested';
}

/** Bound DOM work even when callers supply a much larger development catalogue. */
export function searchCommands(
  query: string,
  registry: readonly Command[] = commands,
  limit = 40,
  options: SearchOptions = {},
): Command[] {
  const maximum = Number.isFinite(limit)
    ? Math.max(0, Math.min(40, Math.floor(limit)))
    : 40;
  const normalized = normalize(query).slice(0, 256);
  const now = options.now ?? Date.now();
  const favorites = new Set(options.favorites ?? []);
  const collection = options.collection ?? 'all';
  const candidates =
    collection === 'favorites'
      ? collectionItems(registry, options.favorites ?? [])
      : collection === 'recents'
        ? recentItems(registry, options)
        : [...registry];
  if (!normalized) {
    if (collection !== 'all') return candidates.slice(0, maximum);
    const recent = recentItems(registry, options);
    const seen = new Set(recent.map(identity));
    const pinned = collectionItems(registry, options.favorites ?? [])
      .filter((command) => !seen.has(identity(command)))
      .slice(0, 8);
    pinned.forEach((command) => seen.add(identity(command)));
    const suggested = candidates
      .map((command, index) => ({
        command,
        index,
        learned: learnedScore(options.usage?.[identity(command)], now),
      }))
      .filter(({ command }) => !seen.has(identity(command)))
      .sort(
        (a, b) =>
          Number(favorites.has(identity(b.command))) -
            Number(favorites.has(identity(a.command))) ||
          b.learned - a.learned ||
          a.index - b.index,
      )
      .map(({ command }) => command);
    return [...recent, ...pinned, ...suggested].slice(0, maximum);
  }
  const exactAlias = query.trim().normalize('NFC').toLowerCase();
  return candidates
    .map((command, index) => ({
      command,
      index,
      score: options.aliases?.[identity(command)]?.some(
        (alias) => alias.normalize('NFC').toLowerCase() === exactAlias,
      )
        ? 1100
        : command.action.type === 'file' && command.searchQuery === query
          ? (command.searchRelevance ?? 0)
          : scoreEntry(normalized, {
              ...entryFor(command),
              keywords: [
                ...entryFor(command).keywords,
                ...(options.aliases?.[identity(command)] ?? []).map(normalize),
              ],
            }),
      pinned: favorites.has(identity(command)),
      learned: learnedScore(options.usage?.[identity(command)], now),
    }))
    .filter(({ score }) => score > 0)
    .sort(
      (a, b) =>
        b.score - a.score ||
        Number(b.pinned) - Number(a.pinned) ||
        b.learned - a.learned ||
        a.index - b.index,
    )
    .slice(0, maximum)
    .map(({ command }) => command);
}
