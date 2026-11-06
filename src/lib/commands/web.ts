import type { Command } from '../types/command';
export const engines = {
  google: ['Google', 'https://www.google.com/search?q={query}'],
  bing: ['Bing', 'https://www.bing.com/search?q={query}'],
  duckduckgo: ['DuckDuckGo', 'https://duckduckgo.com/?q={query}'],
} as const;
export type SearchEngine = keyof typeof engines | 'custom';
export function validSearchTemplate(template: string): boolean {
  if (template.split('{query}').length !== 2) return false;
  try {
    const url = new URL(template.replace('{query}', 'search'));
    return (
      ['https:', 'http:'].includes(url.protocol) &&
      !!url.hostname &&
      !url.username &&
      !url.password
    );
  } catch {
    return false;
  }
}
export function webResult(
  query: string,
  engine: SearchEngine,
  custom: string,
): Command[] {
  const text = query.trimStart().slice(1).trim();
  if (!text) return [];
  const [name, template] =
    engine === 'custom' ? ['Custom search', custom] : engines[engine];
  if (!validSearchTemplate(template)) return [];
  return [
    {
      id: 'web-search',
      module: 'web-search',
      label: `Search ${name} for “${text}”`,
      description: 'Open in your default browser',
      category: 'Utilities',
      icon: 'globe',
      keywords: [],
      action: {
        type: 'web-search',
        url: template.replace('{query}', encodeURIComponent(text)),
      },
    },
  ];
}
