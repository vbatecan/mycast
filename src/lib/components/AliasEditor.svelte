<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { get } from 'svelte/store';
  import type { Command } from '../types/command';
  import { personalization, setAliases } from '../stores/personalization';
  let {
    itemKey,
    label,
    command,
    onclose,
  }: {
    itemKey: string;
    label: string;
    command?: Command;
    onclose: () => void;
  } = $props();
  let draft = $state(
    untrack(() => get(personalization).aliases[itemKey]?.join('\n') ?? ''),
  );
  let error = $state('');
  let field: HTMLTextAreaElement;
  let dialog: HTMLDivElement;
  onMount(() => {
    const previous = document.activeElement as HTMLElement | null;
    field?.focus();
    return () => previous?.focus();
  });
  function save() {
    error =
      setAliases(
        command ?? itemKey,
        draft.split(/\r?\n/).filter((alias) => alias.length > 0),
      ) ?? '';
    if (!error) onclose();
  }
  function keyboard(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      onclose();
    }
    if (event.key === 'Enter' && event.ctrlKey) {
      event.preventDefault();
      save();
    }
    if (event.key === 'Tab') {
      const controls = [
        ...dialog.querySelectorAll<HTMLElement>('textarea, button'),
      ];
      const index = controls.indexOf(document.activeElement as HTMLElement);
      if (event.shiftKey && index === 0) {
        event.preventDefault();
        controls.at(-1)?.focus();
      } else if (!event.shiftKey && index === controls.length - 1) {
        event.preventDefault();
        controls[0]?.focus();
      }
    }
  }
</script>

<div class="alias-overlay">
  <div
    class="alias-dialog"
    bind:this={dialog}
    role="dialog"
    aria-modal="true"
    aria-labelledby="alias-title"
    tabindex="-1"
    onkeydown={keyboard}
  >
    <h2 id="alias-title">Aliases for {label}</h2>
    <label for="alias-lines">One alias per line · up to 10</label>
    <textarea
      id="alias-lines"
      bind:this={field}
      bind:value={draft}
      rows="5"
      spellcheck="false"
      aria-describedby="alias-help"></textarea>
    <p id="alias-help">
      Each alias needs a letter, has no spaces, and can contain up to 32
      characters. Aliases are unique across commands.
    </p>
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    <div class="actions">
      <button onclick={onclose}>Cancel</button><button
        class="primary"
        onclick={save}>Save aliases</button
      >
    </div>
  </div>
</div>

<style>
  .alias-overlay {
    position: absolute;
    inset: 0;
    z-index: 20;
    background: #0006;
    display: grid;
    place-items: center;
    padding: 18px;
  }
  .alias-dialog {
    width: min(440px, 100%);
    max-height: 100%;
    overflow-y: auto;
    padding: 20px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--bg);
    box-shadow: 0 12px 40px #0006;
  }
  h2 {
    margin: 0 0 15px;
    font-size: var(--text-16);
    font-weight: 600;
  }
  label,
  p {
    font-size: var(--text-11);
    color: var(--secondary);
    line-height: 1.5;
  }
  textarea {
    display: block;
    width: 100%;
    resize: vertical;
    min-height: 85px;
    max-height: 180px;
    margin: 8px 0;
    padding: 9px;
    font-family: inherit;
    font-size: var(--text-13);
    color: var(--text);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
  }
  .actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    margin-top: 15px;
  }
  button {
    padding: 7px 12px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface);
    font-size: var(--text-12);
  }
  .primary {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .error {
    color: var(--danger);
  }
</style>
