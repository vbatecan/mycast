import { get, writable } from 'svelte/store';
import { invoke, isTauri } from '@tauri-apps/api/core';
import type { Command } from '../types/command';
export const moduleDefinitions = [
  [
    'ai',
    'AI assistant',
    'Ask with ?? using your own API key or a local model.',
  ],
  [
    'web-search',
    'Web search',
    'Search the web with ? in your default browser.',
  ],
  [
    'windows',
    'Windows settings and tools',
    'Find Windows settings and system tools.',
  ],
  [
    'applications',
    'Applications',
    'Installed application discovery and launching.',
  ],
  [
    'files',
    'Files and folders',
    'File search, indexing, and live folder updates.',
  ],
  ['run', 'Terminal / Run', 'Run commands in PowerShell.'],
  [
    'calculator',
    'Calculator · fend',
    'Scientific notation, advanced arithmetic, complex numbers, and units.',
  ],
  [
    'currency',
    'Currency converter',
    'Reference exchange rates with an offline cache.',
  ],
  [
    'utilities',
    'Utility commands',
    'Date, time, UUID, and built-in folder commands.',
  ],
] as const;
export type ModuleId = (typeof moduleDefinitions)[number][0];
export type ModuleState = Record<ModuleId, boolean>;
const defaults = Object.fromEntries(
  moduleDefinitions.map(([id]) => [id, true]),
) as ModuleState;
export const modules = writable<ModuleState>({ ...defaults });
export const modulesReady = writable(false);
let pending: Promise<void> | undefined;
export function initializeModules(): Promise<void> {
  return (pending ??= (async () => {
    if (isTauri()) modules.set(await invoke<ModuleState>('get_modules'));
    else {
      try {
        modules.set({
          ...defaults,
          ...JSON.parse(localStorage.getItem('mycast.modules') ?? '{}'),
        });
      } catch {}
    }
    modulesReady.set(true);
  })().catch((error) => {
    pending = undefined;
    throw error;
  }));
}
export async function setModule(id: ModuleId, enabled: boolean) {
  if (isTauri())
    modules.set(await invoke<ModuleState>('set_module', { id, enabled }));
  else {
    const next = { ...get(modules), [id]: enabled };
    localStorage.setItem('mycast.modules', JSON.stringify(next));
    modules.set(next);
  }
}
export function commandModule(command: Command): ModuleId | null {
  if (
    command.action.type === 'application' ||
    command.category === 'Applications'
  )
    return 'applications';
  if (command.module) return command.module;
  if (command.action.type === 'file') return 'files';
  if (command.action.type === 'run') return 'run';
  if (
    command.action.type === 'calculator-mode' ||
    command.category === 'Calculator'
  )
    return 'calculator';
  return command.category === 'Preferences' ? null : 'utilities';
}
export function commandEnabled(command: Command, state: ModuleState): boolean {
  const id = commandModule(command);
  return !id || state[id];
}
