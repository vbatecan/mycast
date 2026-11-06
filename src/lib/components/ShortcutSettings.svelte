<script lang="ts">
  import { preferences, updatePreference } from '../stores/preferences';
  import {
    shortcutActions,
    defaultShortcuts,
    eventShortcut,
    validShortcut,
    shortcutLabel,
    type ShortcutAction,
  } from '../shortcuts';
  let { launcherShortcut }: { launcherShortcut: string | null } = $props();
  let recording = $state<ShortcutAction | null>(null);
  let error = $state('');
  export function cancelRecording(): boolean {
    if (!recording) return false;
    recording = null;
    error = '';
    return true;
  }
  function capture(event: KeyboardEvent, id: ShortcutAction) {
    if (recording !== id) return;
    event.preventDefault();
    event.stopPropagation();
    if (event.key === 'Escape') {
      cancelRecording();
      return;
    }
    const binding = eventShortcut(event);
    if (!binding) return;
    if (!validShortcut(binding)) {
      error =
        'Use a modified key or a navigation key. Keep typing, text editing, and system shortcuts available.';
      return;
    }
    if (binding === launcherShortcut) {
      error = 'This shortcut already opens Mycast.';
      return;
    }
    const duplicate = shortcutActions.find(
      ([other]) => other !== id && $preferences.shortcuts[other] === binding,
    );
    if (duplicate) {
      error = `Already used for ${duplicate[1]}. Choose a different shortcut.`;
      return;
    }
    updatePreference('shortcuts', { ...$preferences.shortcuts, [id]: binding });
    recording = null;
    error = '';
  }
</script>

<section aria-labelledby="shortcut-title">
  <div class="title">
    <h2 id="shortcut-title">Launcher controls</h2>
    <button
      onclick={() => {
        updatePreference('shortcuts', { ...defaultShortcuts });
        cancelRecording();
      }}>Reset controls</button
    >
  </div>
  <p>
    These shortcuts work while Mycast is open. Select a shortcut, then press a
    new combination. Esc cancels recording.
  </p>
  {#if error}<p class="error" role="alert">{error}</p>{/if}
  {#each shortcutActions as [id, label]}
    <div class="shortcut-row">
      <span>{label}</span>
      <button
        class:recording={recording === id}
        aria-label={'Change shortcut for ' + label}
        onclick={() => {
          recording = id;
          error = '';
        }}
        onkeydown={(event) => capture(event, id)}
        onblur={() => {
          if (recording === id) cancelRecording();
        }}
        >{recording === id
          ? 'Press shortcut…'
          : shortcutLabel($preferences.shortcuts[id])}</button
      >
    </div>
  {/each}
</section>

<style>
  section {
    margin-top: 20px;
  }
  .title {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  h2 {
    font-size: var(--text-16);
    font-weight: 600;
    margin: 0;
  }
  p {
    font-size: var(--text-11);
    line-height: 1.5;
    color: var(--secondary);
    margin: 10px 0;
  }
  .error {
    color: var(--danger);
  }
  .shortcut-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 15px;
    min-height: 45px;
    border-bottom: 1px solid var(--border);
    font-size: var(--text-12);
  }
  button {
    padding: 6px 10px;
    border: 1px solid var(--border);
    border-radius: 5px;
    background: var(--surface);
    color: var(--secondary);
    font-size: var(--text-11);
  }
  .shortcut-row button {
    min-width: 115px;
  }
  button:hover {
    background: var(--hover);
    color: var(--text);
  }
  button.recording {
    border-color: var(--accent);
    color: var(--accent);
  }
</style>
