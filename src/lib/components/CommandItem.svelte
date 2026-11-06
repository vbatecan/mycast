<script lang="ts">
  import { ArrowUpRight } from '@lucide/svelte';
  import CommandIcon from './CommandIcon.svelte';
  import type { Command } from '../types/command';
  let {
    command,
    selected,
    showDescription,
    onselect,
    onexecute,
  }: {
    command: Command;
    selected: boolean;
    showDescription: boolean;
    onselect: () => void;
    onexecute: () => void;
  } = $props();
  let tone = $derived(
    command.category === 'Applications'
      ? 'blue'
      : command.category === 'Quick Actions'
        ? 'amber'
        : 'violet',
  );
</script>

<button
  type="button"
  tabindex="-1"
  id={'command-' + command.id}
  role="option"
  aria-selected={selected}
  class="command-row"
  class:selected
  class:calculation={command.id === 'calculation'}
  onpointermove={onselect}
  onmousedown={(e) => e.preventDefault()}
  onclick={onexecute}
  title={command.category === 'Files'
    ? command.description + '\\' + command.label
    : undefined}
>
  <span class="command-icon {tone}"
    ><CommandIcon
      name={command.icon}
      applicationId={command.action.type === 'application'
        ? command.action.applicationId
        : undefined}
    /></span
  >
  <span class="command-copy"
    ><span class="command-label">{command.label}</span>
    {#if showDescription}<span class="command-description"
        >{command.description}</span
      >{/if}
  </span>
  <span class="command-kind"
    >{command.action.type === 'copy' || command.id.startsWith('copy-')
      ? 'Copy'
      : command.action.type === 'run'
        ? 'Run'
        : command.action.type === 'file'
          ? command.action.kind === 'folder'
            ? 'Folder'
            : 'File'
          : command.category === 'Applications'
            ? 'Application'
            : command.category === 'Preferences'
              ? 'Mycast'
              : 'Command'}</span
  >
  {#if selected}<span class="row-enter" aria-hidden="true">↵</span
    >{:else}<ArrowUpRight class="row-arrow" size={13} aria-hidden="true" />{/if}
</button>

<style>
  .command-row {
    display: flex;
    align-items: center;
    text-align: left;
    width: 100%;
    background: transparent;
    gap: 12px;
    min-height: var(--row-min-height);
    padding: var(--row-padding) 12px;
    border: 1px solid transparent;
    border-radius: 8px;
    cursor: pointer;
    transition:
      background calc(120ms * var(--motion-scale, 1)),
      border-color calc(120ms * var(--motion-scale, 1));
  }
  .command-row:hover {
    background: var(--hover);
  }
  .command-row.selected {
    background: var(--selected);
    border-color: color-mix(in srgb, var(--border) 35%, transparent);
  }
  .command-icon {
    display: grid;
    place-items: center;
    height: var(--row-icon);
    width: var(--row-icon);
    flex-shrink: 0;
    border-radius: 8px;
    border: 1px solid color-mix(in srgb, var(--text) 5%, transparent);
  }
  .blue {
    background: color-mix(in srgb, var(--accent) 11%, var(--surface));
    color: var(--accent);
  }
  .violet {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .amber {
    background: color-mix(in srgb, #c49d60 10%, var(--surface));
    color: color-mix(in srgb, #c49d60 65%, var(--text));
  }
  .command-copy {
    display: flex;
    flex: 1;
    min-width: 0;
    flex-direction: column;
    gap: 2px;
  }
  .command-label {
    font-size: var(--text-13);
    font-weight: 500;
    line-height: var(--text-18);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: normal;
    overflow-wrap: anywhere;
  }
  .command-description {
    color: var(--secondary);
    font-size: var(--text-11);
    line-height: var(--text-15);
    white-space: normal;
    overflow-wrap: anywhere;
    text-overflow: ellipsis;
    overflow: hidden;
  }
  .command-kind {
    font-size: var(--text-10);
    color: var(--muted);
    flex-shrink: 0;
  }
  .row-enter {
    font-size: var(--text-14);
    width: 21px;
    min-height: 20px;
    border-radius: 4px;
    border: 1px solid var(--border);
    text-align: center;
    color: var(--secondary);
    line-height: var(--text-17);
    background: var(--surface);
  }
  :global(.row-arrow) {
    width: 21px;
    color: var(--muted);
  }
  .calculation {
    margin-top: 8px;
    min-height: 112px;
    gap: 18px;
    padding: 20px;
  }
  .calculation .command-icon {
    height: 43px;
    width: 43px;
    border-radius: 11px;
  }
  .calculation .command-label {
    font-size: var(--text-32);
    line-height: var(--text-42);
    font-weight: 450;
    letter-spacing: -1px;
  }
  .calculation .command-description {
    margin-top: 4px;
  }
</style>
