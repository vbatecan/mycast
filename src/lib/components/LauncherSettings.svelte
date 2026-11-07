<script lang="ts">
  import { get } from 'svelte/store';
  import { preferences } from '../stores/preferences';
  import { onMount, onDestroy } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import {
    getLauncherStatus,
    isNative,
    setLauncherShortcut,
    setShortcutCapture,
    setLaunchAtStartup,
    type LauncherStatus,
  } from '../native';
  import KeyboardHint from './KeyboardHint.svelte';
  let {
    shortcut,
    shortcutError,
    onstatuschange = () => {},
  }: {
    shortcut: string | null;
    shortcutError: string | null;
    onstatuschange?: (status: LauncherStatus) => void;
  } = $props();
  let status = $state<LauncherStatus | null>(null);
  let recording = $state(false);
  let listenersReady = $state(false);
  let candidate = $state('');
  let error = $state('');
  let busy = $state(false);
  let recorder = $state<HTMLButtonElement>();
  let disposed = false;
  const unlisteners: UnlistenFn[] = [];
  const message = (reason: unknown) =>
    reason instanceof Error ? reason.message : String(reason);
  function update(next: LauncherStatus) {
    status = next;
    onstatuschange(next);
  }
  async function cancel() {
    recording = false;
    candidate = '';
    if (isNative()) await setShortcutCapture(false).catch(() => {});
  }
  export function cancelRecording(): boolean {
    if (!recording && !candidate) return false;
    void cancel();
    return true;
  }
  onMount(() => {
    if (!isNative()) return;
    void getLauncherStatus()
      .then((value) => {
        if (!disposed) update(value);
      })
      .catch((reason) => {
        error = message(reason);
      });
    void (async () => {
      for (const [name, callback] of [
        [
          'launcher-shortcut-captured',
          (event: { payload: string }) => {
            if (recording) {
              candidate = event.payload;
              error = '';
              recording = false;
              void setShortcutCapture(false).catch((reason) => {
                error = message(reason);
              });
            }
          },
        ],
        [
          'launcher-shortcut-capture-error',
          (event: { payload: string }) => {
            if (recording) error = event.payload;
          },
        ],
        [
          'launcher-shortcut-capture-ended',
          (event: { payload: string }) => {
            recording = false;
            if (event.payload !== 'completed') candidate = '';
          },
        ],
      ] as const) {
        const unlisten = await listen(
          name,
          callback as Parameters<typeof listen>[1],
        );
        if (disposed) unlisten();
        else unlisteners.push(unlisten);
      }
    })()
      .then(() => {
        if (!disposed) listenersReady = true;
      })
      .catch((reason) => {
        if (!disposed) error = message(reason);
      });
  });
  onDestroy(() => {
    disposed = true;
    unlisteners.forEach((fn) => fn());
    void cancel();
  });
  async function record() {
    error = '';
    candidate = '';
    recording = true;
    try {
      await setShortcutCapture(true);
      if (disposed || !recording) {
        await setShortcutCapture(false);
        return;
      }
      recorder?.focus();
    } catch (reason) {
      recording = false;
      error = message(reason);
    }
  }
  function capture(event: KeyboardEvent) {
    if (!recording) return;
    event.preventDefault();
    event.stopPropagation();
    // Windows input is captured before the OS handles Alt/Windows shortcuts.
    if (status?.platform === 'windows') return;
    if (event.key === 'Escape') {
      void cancel();
      return;
    }
    if (['Control', 'Alt', 'Shift', 'Meta'].includes(event.key)) return;
    if (!(event.ctrlKey || event.altKey || event.metaKey)) {
      error = 'Include Ctrl, Alt, or Windows with a key.';
      return;
    }
    const key = event.code.startsWith('Key')
      ? event.code.slice(3)
      : event.code.startsWith('Digit')
        ? event.code.slice(5)
        : event.code;
    if (!key || ['Tab', 'Escape'].includes(key)) {
      error = 'Choose a different key.';
      return;
    }
    candidate = [
      event.ctrlKey && 'Ctrl',
      event.altKey && 'Alt',
      event.shiftKey && 'Shift',
      event.metaKey && 'Super',
      key,
    ]
      .filter(Boolean)
      .join('+');
    recording = false;
    void setShortcutCapture(false).catch((reason) => {
      error = message(reason);
    });
  }
  async function save(value: string | null) {
    if (value && Object.values(get(preferences).shortcuts).includes(value)) {
      error =
        'This shortcut is already used by a launcher control. Change it in Shortcuts first.';
      return;
    }
    busy = true;
    error = '';
    try {
      await setShortcutCapture(false);
      update(await setLauncherShortcut(value));
      candidate = '';
      recording = false;
    } catch (reason) {
      error = message(reason);
    } finally {
      busy = false;
    }
  }
  async function toggleStartup() {
    if (!status || busy) return;
    busy = true;
    error = '';
    try {
      update(await setLaunchAtStartup(!status.launchAtStartup));
    } catch (reason) {
      error = message(reason);
      try {
        update(await getLauncherStatus());
      } catch {
        /* Preserve the last confirmed state if read-back is unavailable. */
      }
    } finally {
      busy = false;
    }
  }
</script>

<svelte:window
  onblur={() => {
    if (recording || candidate) void cancel();
  }}
/>
<div class="setting-row">
  <div class="setting-label">
    <span>Global shortcut</span><small
      >Toggle Mycast from anywhere. Manage in-app controls in Shortcuts.</small
    >
  </div>
  <div class="shortcut-controls" data-shortcut-recorder>
    <button
      class="recorder"
      bind:this={recorder}
      onclick={record}
      onkeydowncapture={capture}
      onblur={() => {
        if (recording) void cancel();
      }}
      disabled={!isNative() || !status || !listenersReady || busy}
      aria-label={recording
        ? 'Press a launcher shortcut, Escape to cancel'
        : 'Record launcher shortcut'}
    >
      {#if recording}Press shortcut…{:else if candidate}{candidate}{:else if status?.shortcut ?? shortcut}<KeyboardHint
          keys={(status?.shortcut ?? shortcut ?? '').split('+')}
        />{:else}Record shortcut{/if}
    </button>
    {#if recording || candidate}<button
        onclick={() => void cancel()}
        disabled={busy}>Cancel</button
      >{/if}
    {#if candidate}<button onclick={() => save(candidate)} disabled={busy}
        >Save</button
      >{/if}
    <button
      onclick={() => save(null)}
      disabled={!isNative() || busy || recording}>Reset</button
    >
  </div>
</div>
{#if error || status?.shortcutError || shortcutError}<p
    class="error"
    role="alert"
  >
    {error || status?.shortcutError || shortcutError}
  </p>{/if}
<div class="setting-row">
  <div class="setting-label">
    <span>Launch at startup</span><small
      >{status?.startupAvailable
        ? 'Start directly in the tray at sign-in. Manual launches show Mycast.'
        : status?.startupError ||
          'Available in the installed release. Development depends on Vite.'}</small
    >
  </div>
  <button
    class="switch"
    class:enabled={status?.launchAtStartup}
    role="switch"
    aria-checked={status?.launchAtStartup ?? false}
    aria-label="Launch at startup"
    disabled={!status?.startupAvailable || busy}
    onclick={toggleStartup}><span></span></button
  >
</div>
{#if status?.startupAvailable && status.startupError}<p
    class="error"
    role="alert"
  >
    {status.startupError}
  </p>{/if}

<style>
  .setting-row {
    min-height: 72px;
    padding: 10px 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    border-bottom: 1px solid var(--border);
  }
  .setting-label {
    display: flex;
    flex-direction: column;
    gap: 5px;
    min-width: 0;
  }
  .setting-label > span {
    font-size: var(--text-12);
  }
  small {
    font-size: var(--text-10);
    color: var(--muted);
    line-height: 1.4;
  }
  .shortcut-controls {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: 5px;
    flex-wrap: wrap;
    max-width: 55%;
  }
  .shortcut-controls button {
    min-height: 28px;
    padding: 4px 7px;
    border: 1px solid var(--border);
    border-radius: 5px;
    background: var(--surface);
    font-size: var(--text-10);
  }
  .shortcut-controls .recorder {
    min-width: 100px;
  }
  .switch {
    display: inline-flex;
    align-items: center;
    flex-shrink: 0;
    height: 20px;
    width: 34px;
    padding: 3px;
    border-radius: 12px;
    background: var(--selected);
    border: 0;
  }
  .switch span {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--secondary);
  }
  .switch.enabled {
    background: var(--accent);
  }
  .switch.enabled span {
    transform: translateX(14px);
    background: var(--bg);
  }
  .switch:disabled {
    opacity: 0.4;
  }
  .error {
    margin: 7px 0 3px;
    font-size: var(--text-10);
    color: var(--accent);
    line-height: 1.5;
  }
</style>
