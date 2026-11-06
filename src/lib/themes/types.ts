export const colorKeys = [
  'background',
  'surface',
  'hover',
  'selected',
  'text',
  'secondary',
  'muted',
  'border',
  'accent',
] as const;

export type ThemeColorKey = (typeof colorKeys)[number];
export type ThemeColors = Record<ThemeColorKey, string>;
export type ThemeScheme = 'dark' | 'light';

export interface Theme {
  id: string;
  name: string;
  scheme: ThemeScheme;
  colors: ThemeColors;
}

export interface ThemeDraft {
  id?: string;
  name: string;
  scheme: ThemeScheme;
  colors: ThemeColors;
}

export type ThemeResult =
  { ok: true; theme: Theme } | { ok: false; error: string };
export type ThemeDraftResult =
  { ok: true; draft: ThemeDraft } | { ok: false; error: string };

export interface ThemesState {
  activeId: string;
  mode: 'manual' | 'system';
  darkId: string;
  lightId: string;
  customThemes: Theme[];
  /** Editor-only override. Never persisted or included in an export. */
  preview: Theme | null;
}
