export const shortcutActions = [
  ['all', 'All', 'Alt+Shift+1'],
  ['applications', 'Applications', 'Alt+Shift+2'],
  ['commands', 'Commands', 'Alt+Shift+3'],
  ['files', 'Files', 'Alt+Shift+4'],
  ['favorites', 'Favorites', 'Alt+Shift+5'],
  ['recents', 'Recents', 'Alt+Shift+6'],
  ['pin', 'Pin selected item', 'Ctrl+D'],
  ['nextFilter', 'Next filter', 'Alt+Shift+ArrowRight'],
  ['previousFilter', 'Previous filter', 'Alt+Shift+ArrowLeft'],
  ['nextItem', 'Next result', 'ArrowDown'],
  ['previousItem', 'Previous result', 'ArrowUp'],
  ['open', 'Open selected item', 'Enter'],
  ['openAdmin', 'Open as administrator', 'Shift+Enter'],
  ['close', 'Close / Back', 'Escape'],
] as const;
export type ShortcutAction = (typeof shortcutActions)[number][0];
export type ShortcutBindings = Record<ShortcutAction, string>;
export const defaultShortcuts = Object.fromEntries(
  shortcutActions.map(([id, , binding]) => [id, binding]),
) as ShortcutBindings;

export function eventShortcut(
  event: Pick<
    KeyboardEvent,
    'code' | 'key' | 'ctrlKey' | 'altKey' | 'shiftKey' | 'metaKey'
  >,
): string | null {
  if (['Control', 'Alt', 'Shift', 'Meta'].includes(event.key)) return null;
  const key = event.code.startsWith('Key')
    ? event.code.slice(3)
    : event.code.startsWith('Digit')
      ? event.code.slice(5)
      : event.code || event.key;
  return [
    event.ctrlKey && 'Ctrl',
    event.altKey && 'Alt',
    event.shiftKey && 'Shift',
    event.metaKey && 'Super',
    key,
  ]
    .filter(Boolean)
    .join('+');
}

export function validShortcut(value: unknown): value is string {
  if (typeof value !== 'string') return false;
  if (
    !/^(Ctrl\+)?(Alt\+)?(Shift\+)?(Super\+)?([A-Z0-9]|F(?:[1-9]|1[0-9]|2[0-4])|ArrowUp|ArrowDown|ArrowLeft|ArrowRight|Enter|Escape|Tab|Space|Home|End|PageUp|PageDown|Delete|Backspace)$/.test(
      value,
    )
  )
    return false;
  const parts = value.split('+');
  const key = parts.at(-1)!;
  // Keep plain typing and text-editing shortcuts available in the search field.
  if (
    /^[A-Z0-9]$/.test(key) &&
    !parts.some((part) => ['Ctrl', 'Alt', 'Super'].includes(part))
  )
    return false;
  return ![
    'Ctrl+A',
    'Ctrl+C',
    'Ctrl+V',
    'Ctrl+X',
    'Ctrl+Z',
    'Ctrl+Y',
    'Alt+F4',
    'Alt+Tab',
    'Tab',
    'Shift+Tab',
  ].includes(value);
}

export function validateShortcuts(value: unknown): ShortcutBindings {
  const result = { ...defaultShortcuts };
  if (!value || typeof value !== 'object') return result;
  const candidate = value as Partial<ShortcutBindings>;
  for (const [id] of shortcutActions)
    if (validShortcut(candidate[id])) result[id] = candidate[id];
  // Restore the full default set for corrupted duplicate bindings.
  return new Set(Object.values(result)).size === shortcutActions.length
    ? result
    : { ...defaultShortcuts };
}

export function shortcutLabel(binding: string): string {
  return binding
    .replace('ArrowUp', '↑')
    .replace('ArrowDown', '↓')
    .replace('ArrowLeft', '←')
    .replace('ArrowRight', '→')
    .replace('Escape', 'Esc');
}

/** Upgrade only legacy defaults, keeping custom bindings and avoiding conflicts. */
export function migrateFilterShortcuts(value: unknown): ShortcutBindings {
  const result = validateShortcuts(value);
  const legacy: Partial<ShortcutBindings> = {
    all: 'Ctrl+Alt+1',
    applications: 'Ctrl+Alt+2',
    commands: 'Ctrl+Alt+3',
    files: 'Ctrl+Alt+4',
    favorites: 'Ctrl+Alt+5',
    recents: 'Ctrl+Alt+6',
    nextFilter: 'Ctrl+Tab',
    previousFilter: 'Ctrl+Shift+Tab',
  };
  for (const id of Object.keys(legacy) as ShortcutAction[]) {
    if (result[id] !== legacy[id]) continue;
    const target = defaultShortcuts[id];
    if (
      Object.entries(result).some(
        ([other, binding]) => other !== id && binding === target,
      )
    )
      continue;
    result[id] = target;
  }
  return result;
}
