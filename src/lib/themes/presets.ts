import type { Theme } from './types';

export const defaultThemeId = 'mycast-dark';

export const builtInThemes: readonly Theme[] = [
  {
    id: 'mycast-dark',
    name: 'Mycast Dark',
    scheme: 'dark',
    colors: {
      background: '#17171b',
      surface: '#202026',
      hover: '#292930',
      selected: '#303039',
      text: '#f5f5f7',
      secondary: '#a1a1aa',
      muted: '#777780',
      border: '#2a2a2e',
      accent: '#b5a2ff',
    },
  },
  {
    id: 'midnight',
    name: 'Midnight',
    scheme: 'dark',
    colors: {
      background: '#101823',
      surface: '#172231',
      hover: '#202f41',
      selected: '#293b50',
      text: '#f0f5fc',
      secondary: '#a2b4cb',
      muted: '#768ca7',
      border: '#2a394c',
      accent: '#a0c7ff',
    },
  },
  {
    id: 'graphite',
    name: 'Graphite',
    scheme: 'dark',
    colors: {
      background: '#1b1b1a',
      surface: '#242422',
      hover: '#2d2d2a',
      selected: '#383834',
      text: '#f6f5ef',
      secondary: '#b4b2a8',
      muted: '#838178',
      border: '#353531',
      accent: '#dcc7a1',
    },
  },
  {
    id: 'paper',
    name: 'Paper',
    scheme: 'light',
    colors: {
      background: '#f6f5f1',
      surface: '#eeede8',
      hover: '#e7e5df',
      selected: '#dfdce9',
      text: '#25242b',
      secondary: '#62606a',
      muted: '#7a7780',
      border: '#d9d7d1',
      accent: '#6750a4',
    },
  },
  {
    id: 'forest',
    name: 'Forest',
    scheme: 'dark',
    colors: {
      background: '#151d19',
      surface: '#1e2a23',
      hover: '#27362d',
      selected: '#304237',
      text: '#eff5ef',
      secondary: '#a5b8aa',
      muted: '#789181',
      border: '#2f3e34',
      accent: '#add5b6',
    },
  },
];

// Presets are shared metadata. Editors must work with a draft rather than mutate them.
for (const theme of builtInThemes) {
  Object.freeze(theme.colors);
  Object.freeze(theme);
}
Object.freeze(builtInThemes);
