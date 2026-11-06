import { writable } from 'svelte/store';
import {
  defaultShortcuts,
  validateShortcuts,
  type ShortcutBindings,
} from '../shortcuts';

export interface Preferences {
  animations: boolean;
  showDescriptions: boolean;
  density: 'comfortable' | 'compact';
  textSize: 90 | 100 | 110 | 125;
  material: WindowMaterial;
  opacity: number;
  glow: boolean;
  highlightBorder: boolean;
  recentLimit: number;
  shortcuts: ShortcutBindings;
}

export type WindowMaterial =
  'solid' | 'transparent' | 'blur' | 'acrylic' | 'mica';
export const windowMaterials: WindowMaterial[] = [
  'solid',
  'transparent',
  'blur',
  'acrylic',
  'mica',
];

const storageKey = 'mycast.preferences.v1';
const defaults: Preferences = {
  animations: true,
  showDescriptions: true,
  density: 'comfortable',
  textSize: 100,
  material: 'acrylic',
  opacity: 78,
  glow: false,
  highlightBorder: true,
  recentLimit: 5,
  shortcuts: { ...defaultShortcuts },
};

function validated(value: unknown): Preferences {
  if (!value || typeof value !== 'object') return { ...defaults };
  const candidate = value as Partial<Preferences>;
  return {
    shortcuts: validateShortcuts(candidate.shortcuts),
    recentLimit:
      typeof candidate.recentLimit === 'number' &&
      Number.isFinite(candidate.recentLimit)
        ? Math.min(20, Math.max(1, Math.round(candidate.recentLimit)))
        : defaults.recentLimit,
    material: windowMaterials.includes(candidate.material as WindowMaterial)
      ? candidate.material!
      : defaults.material,
    opacity:
      typeof candidate.opacity === 'number' &&
      Number.isFinite(candidate.opacity)
        ? Math.min(100, Math.max(40, Math.round(candidate.opacity)))
        : defaults.opacity,
    glow: typeof candidate.glow === 'boolean' ? candidate.glow : defaults.glow,
    highlightBorder:
      typeof candidate.highlightBorder === 'boolean'
        ? candidate.highlightBorder
        : defaults.highlightBorder,
    density: candidate.density === 'compact' ? 'compact' : 'comfortable',
    textSize: [90, 100, 110, 125].includes(candidate.textSize as number)
      ? candidate.textSize!
      : 100,
    animations:
      typeof candidate.animations === 'boolean'
        ? candidate.animations
        : defaults.animations,
    showDescriptions:
      typeof candidate.showDescriptions === 'boolean'
        ? candidate.showDescriptions
        : defaults.showDescriptions,
  };
}

function load(): Preferences {
  try {
    if (typeof localStorage !== 'undefined')
      return validated(JSON.parse(localStorage.getItem(storageKey) ?? 'null'));
  } catch {
    // Storage can be unavailable or left with an invalid value; defaults stay usable.
  }
  return { ...defaults };
}

const store = writable<Preferences>(load());

store.subscribe((value) => {
  try {
    if (typeof localStorage !== 'undefined')
      localStorage.setItem(storageKey, JSON.stringify(value));
  } catch {
    // Preferences remain functional for this session if persistence is unavailable.
  }
});

export const preferences = {
  subscribe: store.subscribe,
  set: (value: Preferences) => store.set(validated(value)),
  update: (update: (value: Preferences) => Preferences) =>
    store.update((value) => validated(update(value))),
  reset: () => store.set({ ...defaults }),
};

export function updatePreference<K extends keyof Preferences>(
  key: K,
  value: Preferences[K],
): void {
  preferences.update((current) => ({ ...current, [key]: value }));
}
