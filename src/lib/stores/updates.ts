import { get, writable } from 'svelte/store';
import { invoke, isTauri } from '@tauri-apps/api/core';
import type { Update } from '@tauri-apps/plugin-updater';
export const updateState = writable<{
  status: string;
  version?: string;
  notes?: string;
  error?: string;
  downloaded?: number;
  total?: number;
}>({ status: 'idle' });
let initial = true;
try {
  initial = localStorage.getItem('mycast.autoUpdates') !== 'false';
} catch {}
export const autoUpdates = writable(initial);
autoUpdates.subscribe((value) => {
  try {
    localStorage.setItem('mycast.autoUpdates', String(value));
  } catch {}
});
let update: Update | null = null;
let checking = false;
export async function checkUpdates() {
  if (
    checking ||
    ['downloading', 'installed'].includes(get(updateState).status)
  )
    return;
  checking = true;
  try {
    if (!isTauri())
      throw new Error('Updates are available in the desktop app.');
    if (!(await invoke<boolean>('updater_supported')))
      throw new Error(
        'Automatic installation is supported on Windows and Linux AppImage. Update Debian packages using your package manager or the GitHub download.',
      );
    updateState.set({ status: 'checking' });
    await update?.close();
    update = null;
    const provider = await import('@tauri-apps/plugin-updater');
    update = await provider.check({ timeout: 15000 });
    updateState.set(
      update
        ? { status: 'available', version: update.version, notes: update.body }
        : { status: 'current' },
    );
  } catch (reason) {
    updateState.set({ status: 'error', error: String(reason) });
  } finally {
    checking = false;
  }
}
export async function installUpdate() {
  if (!update || get(updateState).status !== 'available') return;
  const selected = update;
  let downloaded = 0;
  let total: number | undefined;
  updateState.set({ status: 'downloading', version: selected.version });
  try {
    await selected.downloadAndInstall((event) => {
      if (event.event === 'Started') total = event.data.contentLength;
      if (event.event === 'Progress') downloaded += event.data.chunkLength;
      updateState.set({
        status: 'downloading',
        version: selected.version,
        downloaded,
        total,
      });
    });
    updateState.set({ status: 'installed', version: selected.version });
  } catch (reason) {
    updateState.set({
      status: 'available',
      version: selected.version,
      notes: selected.body,
      error: String(reason),
    });
  }
}
export async function restartUpdatedApp() {
  try {
    const { relaunch } = await import('@tauri-apps/plugin-process');
    await relaunch();
  } catch (reason) {
    updateState.update((state) => ({ ...state, error: String(reason) }));
  }
}
export function watchUpdates() {
  if (!isTauri()) return;
  const timer = setTimeout(() => {
    if (get(autoUpdates)) void checkUpdates();
  }, 15000);
  const interval = setInterval(
    () => {
      if (get(autoUpdates)) void checkUpdates();
    },
    6 * 60 * 60 * 1000,
  );
  return () => {
    clearTimeout(timer);
    clearInterval(interval);
  };
}
