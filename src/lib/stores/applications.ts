import { derived, get, writable } from 'svelte/store';
import { isNative, listApplications } from '../native';
import type { InstalledApplication } from '../native';
import type { Command } from '../types/command';
import { commands } from '../commands/registry';
import { invalidateApplicationIcons } from '../applications/icons';

export type CommandScope =
  'all' | 'applications' | 'commands' | 'favorites' | 'recents' | 'files';

interface ApplicationState {
  applications: InstalledApplication[];
  status: 'idle' | 'loading' | 'ready' | 'error' | 'preview';
  warnings: string[];
  error: string | null;
}

const state = writable<ApplicationState>({
  applications: [],
  status: 'idle',
  warnings: [],
  error: null,
});
export const applications = { subscribe: state.subscribe };
let pending: Promise<void> | undefined;

/** One native discovery per session, with explicit refresh and no watchers. */
export function loadApplications(refresh = false): Promise<void> {
  if (pending) return pending;
  if (!isNative()) {
    state.set({
      applications: [],
      status: 'preview',
      warnings: [],
      error: null,
    });
    return Promise.resolve();
  }
  if (!refresh && get(state).status === 'ready') return Promise.resolve();
  state.update((value) => ({ ...value, status: 'loading', error: null }));
  pending = listApplications(refresh)
    .then((catalogue) => {
      if (refresh) invalidateApplicationIcons();
      state.set({
        applications: catalogue.applications,
        warnings: catalogue.warnings,
        status: 'ready',
        error: null,
      });
    })
    .catch((error) => {
      state.update((value) => ({
        ...value,
        status: 'error',
        error:
          typeof error === 'string'
            ? error
            : error instanceof Error
              ? error.message
              : 'Could not discover installed applications.',
      }));
    })
    .finally(() => {
      pending = undefined;
    });
  return pending;
}

export const installedCommands = derived(state, (value) =>
  value.applications.map((application): Command => ({
    id: `app-${application.id}`,
    personalizationKey: application.persistentKey,
    label: application.name,
    description:
      application.description ||
      (application.source === 'start-menu'
        ? 'Start Menu application'
        : 'Windows application'),
    category: 'Applications',
    icon: 'app-window',
    keywords: [...application.keywords, application.name, 'app', 'application'],
    action: { type: 'application', applicationId: application.id },
  })),
);

/** A discovered counterpart inherits the fallback's stable personalization key. */
export function mergeCatalogue(installed: readonly Command[]): Command[] {
  const aliases: Record<string, string[]> = {
    notepad: ['notepad', 'windowsnotepad'],
    'windows-calculator': ['calculator', 'windowscalculator'],
    explorer: ['fileexplorer', 'windowsfileexplorer'],
    'windows-settings': ['settings', 'windowssettings'],
  };
  const replaced = new Set<string>();
  const discovered = installed.map((command) => {
    const name = command.label.toLowerCase().replace(/[^\p{L}\p{N}]+/gu, '');
    const fallback = Object.entries(aliases).find(
      ([id, names]) => !replaced.has(id) && names.includes(name),
    );
    if (!fallback) return command;
    replaced.add(fallback[0]);
    return { ...command, personalizationKey: `builtin:${fallback[0]}` };
  });
  const builtIns = commands.filter((command) => !replaced.has(command.id));
  return [
    ...discovered,
    ...builtIns.filter((command) => command.category === 'Applications'),
    ...builtIns.filter((command) => command.category !== 'Applications'),
  ];
}

export const commandCatalogue = derived(installedCommands, mergeCatalogue);
