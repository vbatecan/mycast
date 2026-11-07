import { writable, get, derived } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import {
  isNative,
  getFileStatus,
  resolveFiles,
  type FileItem,
  type FileIndexStatus,
} from '../native';
import { modules } from './modules';
import { personalization } from './personalization';
import { commandCatalogue } from './applications';
import type { Command } from '../types/command';

export const fileStatus = writable<FileIndexStatus>({
  state: 'idle',
  indexed: 0,
  revision: 0,
  warnings: [],
  preferences: null,
  shell: 'PowerShell',
});
export const savedFileCommands = writable<Command[]>([]);
export const managementCatalogue = derived(
  [commandCatalogue, savedFileCommands],
  ([commands, files]) => [...commands, ...files],
);
export function fileCommand(item: FileItem, query?: string): Command {
  return {
    id: `file-${item.id}`,
    personalizationKey: item.persistentKey,
    label: item.name,
    description: item.parentPath,
    category: 'Files',
    icon: item.kind === 'folder' ? 'folder-open' : 'file-text',
    keywords: [],
    action: { type: 'file', fileId: item.id, kind: item.kind },
    searchRelevance: item.relevance,
    searchQuery: query,
  };
}
let refreshGeneration = 0;
export async function refreshSavedFiles(): Promise<void> {
  if (!isNative() || !get(modules).files) return;
  const generation = ++refreshGeneration;
  const state = get(personalization);
  const keys = [
    ...new Set([
      ...state.favorites,
      ...state.recents,
      ...Object.keys(state.aliases),
    ]),
  ].filter((key) => key.startsWith('file:v1:'));
  const items = keys.length ? await resolveFiles(keys) : [];
  if (generation === refreshGeneration)
    savedFileCommands.set(items.map((item) => fileCommand(item)));
}
/** One listener at the app boundary. Startup in the tray never begins a file scan. */
export async function observeFileIndex(): Promise<() => void> {
  if (!isNative()) return () => {};
  const stop = await listen<FileIndexStatus>('file-index-changed', (event) => {
    fileStatus.update((current) => ({
      ...current,
      ...event.payload,
      preferences: event.payload.preferences ?? current.preferences,
      shell: event.payload.shell || current.shell,
    }));
    void refreshSavedFiles().catch(() => {});
  });
  try {
    fileStatus.set(await getFileStatus());
  } catch (reason) {
    stop();
    throw reason;
  }
  const unsubscribe = personalization.subscribe(() => {
    void refreshSavedFiles().catch(() => {});
  });
  return () => {
    stop();
    unsubscribe();
  };
}
