import { derived, get, writable } from 'svelte/store';
import { builtInThemes, defaultThemeId } from '../themes/presets';
import {
  customThemeLimit,
  parseThemeJson,
  validateThemeDraft,
} from '../themes/validation';
import type {
  Theme,
  ThemeDraft,
  ThemeResult,
  ThemesState,
} from '../themes/types';

export { builtInThemes, defaultThemeId } from '../themes/presets';
export {
  applyTheme,
  themeStyle,
  exportTheme,
  parseThemeJson,
  validateThemeDraft,
  contrastRatio,
  customThemeLimit,
  themeNameLimit,
} from '../themes/validation';
export type {
  Theme,
  ThemeDraft,
  ThemeResult,
  ThemesState,
  ThemeColors,
  ThemeColorKey,
  ThemeScheme,
} from '../themes/types';

const storageKey = 'mycast.themes.v1';
const builtinIds = new Set(builtInThemes.map((theme) => theme.id));

function load(): ThemesState {
  const fallback: ThemesState = {
    activeId: defaultThemeId,
    mode: 'manual',
    darkId: defaultThemeId,
    lightId: 'paper',
    customThemes: [],
    preview: null,
  };
  try {
    if (typeof localStorage === 'undefined') return fallback;
    const source = localStorage.getItem(storageKey);
    if (!source || source.length > 131072) return fallback;
    const value: unknown = JSON.parse(source);
    if (!value || typeof value !== 'object') return fallback;
    const stored = value as Record<string, unknown>;
    if (
      ![1, 2].includes(stored.version as number) ||
      !Array.isArray(stored.customThemes)
    )
      return fallback;
    const customThemes: Theme[] = [];
    const knownIds = new Set(builtinIds);
    for (const candidate of stored.customThemes.slice(0, customThemeLimit)) {
      const result = validateThemeDraft(candidate);
      if (!result.ok || !result.draft.id || knownIds.has(result.draft.id))
        continue;
      const theme: Theme = { ...result.draft, id: result.draft.id };
      customThemes.push(theme);
      knownIds.add(theme.id);
    }
    const activeId =
      typeof stored.activeId === 'string' && knownIds.has(stored.activeId)
        ? stored.activeId
        : defaultThemeId;
    const catalogue = [...builtInThemes, ...customThemes];
    const pairId = (
      id: unknown,
      scheme: 'dark' | 'light',
      fallbackId: string,
    ) =>
      catalogue.some((theme) => theme.id === id && theme.scheme === scheme)
        ? (id as string)
        : fallbackId;
    return {
      activeId,
      customThemes,
      preview: null,
      mode: stored.mode === 'system' ? 'system' : 'manual',
      darkId: pairId(stored.darkId, 'dark', defaultThemeId),
      lightId: pairId(stored.lightId, 'light', 'paper'),
    };
  } catch {
    return fallback;
  }
}

const state = writable<ThemesState>(load());
export const themes = { subscribe: state.subscribe };
export const availableThemes = derived(state, ($state) => [
  ...builtInThemes,
  ...$state.customThemes,
]);
const systemScheme = writable<'dark' | 'light'>('dark');
/** Subscribe once at the app boundary; Windows changes arrive as events, never polling. */
export function observeSystemTheme(): () => void {
  if (typeof window === 'undefined' || !window.matchMedia) return () => {};
  const media = window.matchMedia('(prefers-color-scheme: dark)');
  const update = () => systemScheme.set(media.matches ? 'dark' : 'light');
  update();
  media.addEventListener('change', update);
  return () => media.removeEventListener('change', update);
}
export const activeTheme = derived(
  [state, systemScheme],
  ([$state, $scheme]): Theme =>
    $state.preview ??
    [...builtInThemes, ...$state.customThemes].find(
      (theme) =>
        theme.id ===
        ($state.mode === 'system'
          ? $scheme === 'dark'
            ? $state.darkId
            : $state.lightId
          : $state.activeId),
    ) ??
    builtInThemes[0]!,
);

function persist(value: ThemesState): void {
  try {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem(
        storageKey,
        JSON.stringify({
          version: 2,
          activeId: value.activeId,
          mode: value.mode,
          darkId: value.darkId,
          lightId: value.lightId,
          customThemes: value.customThemes,
        }),
      );
    }
  } catch {
    // Theme changes remain usable for the session when storage is full or unavailable.
  }
}

function commit(value: ThemesState): void {
  state.set(value);
  persist(value);
}

function findTheme(id: string): Theme | undefined {
  return [...builtInThemes, ...get(state).customThemes].find(
    (theme) => theme.id === id,
  );
}

function newId(): string {
  if (
    typeof crypto !== 'undefined' &&
    typeof crypto.randomUUID === 'function'
  ) {
    return `custom-${crypto.randomUUID()}`;
  }
  return `custom-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 12)}`;
}

export function selectTheme(id: string): boolean {
  if (!findTheme(id)) return false;
  commit({ ...get(state), mode: 'manual', activeId: id, preview: null });
  return true;
}

export function setThemeMode(mode: 'manual' | 'system'): void {
  if (mode === 'manual' || mode === 'system')
    commit({ ...get(state), mode, preview: null });
}
export function setSystemTheme(scheme: 'dark' | 'light', id: string): boolean {
  if (findTheme(id)?.scheme !== scheme) return false;
  commit({
    ...get(state),
    [scheme === 'dark' ? 'darkId' : 'lightId']: id,
    preview: null,
  });
  return true;
}

/** Save selects the new theme. Built-in presets are immutable; duplicate one first. */
export function saveTheme(draft: ThemeDraft): ThemeResult {
  const result = validateThemeDraft(draft);
  if (!result.ok) return result;
  const current = get(state);
  const validated = result.draft;
  if (validated.id && builtinIds.has(validated.id)) {
    return {
      ok: false,
      error: 'Duplicate a built-in theme before editing it.',
    };
  }
  const existing = validated.id
    ? current.customThemes.findIndex((theme) => theme.id === validated.id)
    : -1;
  if (validated.id && existing === -1) {
    return {
      ok: false,
      error: 'This custom theme no longer exists. Create a new theme instead.',
    };
  }
  if (existing === -1 && current.customThemes.length >= customThemeLimit) {
    return {
      ok: false,
      error:
        'You can save up to 24 custom themes. Remove one before adding another.',
    };
  }
  const theme: Theme = { ...validated, id: validated.id ?? newId() };
  const customThemes = [...current.customThemes];
  if (existing >= 0) customThemes[existing] = theme;
  else customThemes.push(theme);
  commit({
    ...current,
    mode: 'manual',
    activeId: theme.id,
    customThemes,
    preview: null,
    darkId:
      current.darkId === theme.id && theme.scheme !== 'dark'
        ? defaultThemeId
        : current.darkId,
    lightId:
      current.lightId === theme.id && theme.scheme !== 'light'
        ? 'paper'
        : current.lightId,
  });
  return { ok: true, theme };
}

export function duplicateTheme(
  source: Theme | string,
  name?: string,
): ThemeResult {
  const theme = typeof source === 'string' ? findTheme(source) : source;
  if (!theme)
    return { ok: false, error: 'The theme to duplicate could not be found.' };
  return saveTheme({
    name: name ?? `${theme.name.slice(0, 35)} copy`,
    scheme: theme.scheme,
    colors: { ...theme.colors },
  });
}

export function deleteTheme(id: string): boolean {
  const current = get(state);
  if (!current.customThemes.some((theme) => theme.id === id)) return false;
  commit({
    ...current,
    darkId: current.darkId === id ? defaultThemeId : current.darkId,
    lightId: current.lightId === id ? 'paper' : current.lightId,
    activeId: current.activeId === id ? defaultThemeId : current.activeId,
    customThemes: current.customThemes.filter((theme) => theme.id !== id),
    preview: null,
  });
  return true;
}

export function importTheme(source: string): ThemeResult {
  const result = parseThemeJson(source);
  return result.ok ? saveTheme(result.draft) : result;
}

export function previewTheme(draft: ThemeDraft): ThemeResult {
  const result = validateThemeDraft(draft);
  if (!result.ok) return result;
  const theme: Theme = {
    ...result.draft,
    id: result.draft.id ?? 'custom-preview',
  };
  state.update((current) => ({ ...current, preview: theme }));
  return { ok: true, theme };
}

export function resetPreview(): void {
  state.update((current) =>
    current.preview ? { ...current, preview: null } : current,
  );
}
