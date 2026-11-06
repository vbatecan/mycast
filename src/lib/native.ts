import { invoke, isTauri } from '@tauri-apps/api/core';
import type { NativeAction } from './types/command';
import type { WindowMaterial } from './stores/preferences';

export interface LauncherStatus {
  platform: string;
  shortcut: string | null;
  shortcutError: string | null;
  version: string;
  startupAvailable: boolean;
  launchAtStartup: boolean;
  startupError: string | null;
  visible: boolean;
  shortcutCapture: boolean;
}

export interface InstalledApplication {
  canRunAsAdmin?: boolean;
  id: string;
  persistentKey: string;
  name: string;
  description: string;
  source: 'start-menu' | 'windows-app';
  keywords: string[];
}

export interface ApplicationCatalogue {
  applications: InstalledApplication[];
  warnings: string[];
}

export const listWindowsCommands = (): Promise<
  import('./types/command').Command[]
> => nativeInvoke('list_windows_commands');
export const openWindowsCommand = (id: string): Promise<string> =>
  nativeInvoke('open_windows_command', { id });
export const openSearchUrl = (url: string): Promise<string> =>
  nativeInvoke('open_search_url', { url });
export const isNative = (): boolean => isTauri();

async function nativeInvoke<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!isNative())
    throw new Error(
      'Available in the desktop app. Run npm run tauri dev to use native actions.',
    );
  return invoke<T>(command, args);
}

export const getLauncherStatus = (): Promise<LauncherStatus> =>
  nativeInvoke('get_launcher_status');
export const setLauncherShortcut = (
  shortcut: string | null,
): Promise<LauncherStatus> =>
  nativeInvoke('set_launcher_shortcut', { shortcut });
export const setShortcutCapture = (active: boolean): Promise<void> =>
  nativeInvoke('set_shortcut_capture', { active });
export const setLaunchAtStartup = (enabled: boolean): Promise<LauncherStatus> =>
  nativeInvoke('set_launch_at_startup', { enabled });
export const executeNative = (
  action: NativeAction,
  mode: LaunchMode = 'normal',
): Promise<{ message: string; cancelled?: boolean }> =>
  nativeInvoke('execute_native', { action, mode });
export const copyText = (text: string): Promise<void> =>
  nativeInvoke('copy_text', { text });
export const hideLauncher = (): Promise<void> => nativeInvoke('hide_launcher');
export const quitLauncher = (): Promise<void> => nativeInvoke('quit_launcher');
export const setWindowMaterial = (
  material: WindowMaterial,
  dark: boolean,
): Promise<void> => nativeInvoke('set_window_material', { material, dark });
export const listApplications = (
  refresh = false,
): Promise<ApplicationCatalogue> =>
  nativeInvoke('list_applications', { refresh });
export type LaunchMode = 'normal' | 'administrator';
export const launchApplication = (
  id: string,
  mode: LaunchMode = 'normal',
): Promise<{ message: string; cancelled?: boolean }> =>
  nativeInvoke('launch_application', { id, mode });
export const getApplicationIcon = (id: string): Promise<string | null> =>
  nativeInvoke('get_application_icon', { id });

export interface FilePreferences {
  roots: string[];
  exclusions: string[];
  includeHidden: boolean;
  terminalDirectory: string;
}
export interface FileIndexStatus {
  state: 'idle' | 'indexing' | 'reconciling' | 'ready';
  indexed: number;
  revision: number;
  warnings: string[];
  preferences: FilePreferences | null;
  shell: string;
}
export interface FileItem {
  id: string;
  persistentKey: string;
  name: string;
  parentPath: string;
  kind: 'file' | 'folder';
  relevance: number;
}
export interface FileSearchOptions {
  query: string;
  requestId: number;
  favorites: string[];
  recents: string[];
  aliases: Record<string, string[]>;
  usage: Record<string, { count: number; lastUsed: number }>;
  collection: string;
  browse: boolean;
  now: number;
}
export const getFileStatus = (): Promise<FileIndexStatus> =>
  nativeInvoke('get_file_status');
export const searchFiles = (
  options: FileSearchOptions,
): Promise<{ items: FileItem[]; requestId: number; elapsedMs: number }> =>
  nativeInvoke('search_files', { options });
export const resolveFiles = (keys: string[]): Promise<FileItem[]> =>
  nativeInvoke('resolve_files', { keys });
export const rememberFile = (id: string): Promise<void> =>
  nativeInvoke('remember_file', { id });
export const fileAction = (
  id: string,
  action: 'open' | 'reveal' | 'copy-path',
): Promise<string> => nativeInvoke('file_action', { id, action });
export const rebuildFileIndex = (): Promise<void> =>
  nativeInvoke('rebuild_file_index');
export const setFilePreferences = (
  preferences: FilePreferences,
): Promise<FileIndexStatus> =>
  nativeInvoke('set_file_preferences', { preferences });
export const chooseFileFolder = (): Promise<string | null> =>
  nativeInvoke('choose_file_folder');
export const runShellCommand = (command: string): Promise<string> =>
  nativeInvoke('run_shell_command', { command });
