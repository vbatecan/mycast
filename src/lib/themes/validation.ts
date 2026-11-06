import { colorKeys } from './types';
import type {
  Theme,
  ThemeColors,
  ThemeDraftResult,
  ThemeColorKey,
} from './types';

export const themeNameLimit = 40;
export const customThemeLimit = 24;
const importSizeLimit = 8192;
const solidColor = /^#[0-9a-f]{6}$/i;
const themeId = /^[a-z0-9][a-z0-9-]{0,63}$/;

function record(value: unknown): value is Record<string, unknown> {
  return (
    value !== null &&
    typeof value === 'object' &&
    !Array.isArray(value) &&
    (Object.getPrototypeOf(value) === Object.prototype ||
      Object.getPrototypeOf(value) === null)
  );
}

function exactKeys(
  value: Record<string, unknown>,
  allowed: readonly string[],
): boolean {
  return Object.keys(value).every((key) => allowed.includes(key));
}

function luminance(color: string): number {
  const channels = [1, 3, 5].map((offset) => {
    const channel = Number.parseInt(color.slice(offset, offset + 2), 16) / 255;
    return channel <= 0.04045
      ? channel / 12.92
      : ((channel + 0.055) / 1.055) ** 2.4;
  });
  return channels[0]! * 0.2126 + channels[1]! * 0.7152 + channels[2]! * 0.0722;
}

export function contrastRatio(foreground: string, background: string): number {
  if (!solidColor.test(foreground) || !solidColor.test(background)) return 0;
  const first = luminance(foreground);
  const second = luminance(background);
  return (Math.max(first, second) + 0.05) / (Math.min(first, second) + 0.05);
}

/** All accepted values are copied into a small, fixed schema before use. */
export function validateThemeDraft(value: unknown): ThemeDraftResult {
  if (!record(value) || !exactKeys(value, ['id', 'name', 'scheme', 'colors'])) {
    return {
      ok: false,
      error: 'A theme can contain only a name, scheme, and colors.',
    };
  }
  if (
    typeof value.name !== 'string' ||
    !value.name.trim() ||
    value.name.trim().length > themeNameLimit ||
    /[\u0000-\u001f\u007f-\u009f\u202a-\u202e\u2066-\u2069]/.test(value.name)
  ) {
    return {
      ok: false,
      error: 'Give your theme a name between 1 and 40 characters.',
    };
  }
  if (value.scheme !== 'dark' && value.scheme !== 'light') {
    return { ok: false, error: 'Choose either a dark or light color scheme.' };
  }
  if (
    value.id !== undefined &&
    (typeof value.id !== 'string' || !themeId.test(value.id))
  ) {
    return { ok: false, error: 'The theme identifier is invalid.' };
  }
  if (!record(value.colors) || !exactKeys(value.colors, colorKeys)) {
    return {
      ok: false,
      error: 'A theme must contain exactly the nine supported colors.',
    };
  }
  const colors = {} as ThemeColors;
  for (const key of colorKeys) {
    const color = value.colors[key];
    if (typeof color !== 'string' || !solidColor.test(color)) {
      return {
        ok: false,
        error: `Use a six-digit hex color for ${key}, such as #17171B.`,
      };
    }
    colors[key] = color.toLowerCase();
  }
  if (contrastRatio(colors.text, colors.background) < 4.5) {
    return {
      ok: false,
      error:
        'Increase the text contrast against the background to at least 4.5:1.',
    };
  }
  if (contrastRatio(colors.text, colors.selected) < 4.5) {
    return {
      ok: false,
      error:
        'Increase the text contrast against the selection color to at least 4.5:1.',
    };
  }
  for (const key of ['surface', 'hover'] as const) {
    if (contrastRatio(colors.text, colors[key]) < 4.5) {
      return {
        ok: false,
        error: `Increase the primary text contrast against ${key} to at least 4.5:1.`,
      };
    }
  }
  if (contrastRatio(colors.accent, colors.background) < 4.5) {
    return {
      ok: false,
      error:
        'Increase the accent contrast against the background to at least 4.5:1 so buttons remain readable.',
    };
  }
  if (
    contrastRatio(colors.secondary, colors.background) < 3 ||
    contrastRatio(colors.secondary, colors.surface) < 3
  ) {
    return {
      ok: false,
      error:
        'Increase secondary text contrast against the background and surface to at least 3:1.',
    };
  }
  return {
    ok: true,
    draft: {
      ...(typeof value.id === 'string' ? { id: value.id } : {}),
      name: value.name.trim(),
      scheme: value.scheme,
      colors,
    },
  };
}

/** Import/export never accepts CSS, URLs, font definitions, assets, or identifiers. */
export function parseThemeJson(source: string): ThemeDraftResult {
  if (typeof source !== 'string' || source.length > importSizeLimit) {
    return { ok: false, error: 'Choose a theme JSON file smaller than 8 KB.' };
  }
  let value: unknown;
  try {
    value = JSON.parse(source);
  } catch {
    return { ok: false, error: 'This file is not valid theme JSON.' };
  }
  if (
    !record(value) ||
    !exactKeys(value, ['version', 'name', 'scheme', 'colors'])
  ) {
    return {
      ok: false,
      error: 'Theme files can contain only version, name, scheme, and colors.',
    };
  }
  if (value.version !== 1) {
    return {
      ok: false,
      error:
        'This theme format is not supported. Use a version 1 Mycast theme.',
    };
  }
  return validateThemeDraft({
    name: value.name,
    scheme: value.scheme,
    colors: value.colors,
  });
}

export function exportTheme(theme: Theme): string {
  const result = validateThemeDraft(theme);
  if (!result.ok) throw new Error(result.error);
  const { name, scheme, colors } = result.draft;
  return JSON.stringify({ version: 1, name, scheme, colors }, null, 2) + '\n';
}

const cssProperties: Record<ThemeColorKey, string> = {
  background: '--bg',
  surface: '--surface',
  hover: '--hover',
  selected: '--selected',
  text: '--text',
  secondary: '--secondary',
  muted: '--muted',
  border: '--border',
  accent: '--accent',
};

export function themeStyle(theme: Theme): string {
  const result = validateThemeDraft(theme);
  if (!result.ok) return '';
  return (
    colorKeys
      .map((key) => `${cssProperties[key]}:${result.draft.colors[key]}`)
      .join(';') + `;color-scheme:${result.draft.scheme}`
  );
}

export function applyTheme(theme: Theme, element?: HTMLElement): void {
  const result = validateThemeDraft(theme);
  if (!result.ok || (typeof document === 'undefined' && !element)) return;
  const target = element ?? document.documentElement;
  for (const key of colorKeys)
    target.style.setProperty(cssProperties[key], result.draft.colors[key]);
  target.style.colorScheme = result.draft.scheme;
}
