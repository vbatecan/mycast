<script lang="ts">
  import CommandItem from './CommandItem.svelte';
  import type { Command } from '../types/command';
  let {
    commands,
    selectedIndex,
    showDescriptions,
    query,
    sectionFor,
    onselect,
    onexecute,
  }: {
    commands: Command[];
    selectedIndex: number;
    showDescriptions: boolean;
    query: string;
    sectionFor?: (command: Command) => string;
    onselect: (index: number) => void;
    onexecute: (command: Command) => void;
  } = $props();
  let list: HTMLDivElement;
  $effect(() => {
    const id = commands[selectedIndex]?.id;
    if (id)
      requestAnimationFrame(() =>
        document
          .getElementById('command-' + id)
          ?.scrollIntoView({ block: 'nearest' }),
      );
  });
</script>

<div
  class="results"
  bind:this={list}
  id="command-results"
  role="listbox"
  aria-label="Commands"
>
  {#each commands as command, index (command.id)}
    {#if !query.trim() && (index === 0 || (sectionFor?.(commands[index - 1]) ?? commands[index - 1].category) !== (sectionFor?.(command) ?? command.category))}
      <div class="section-label" role="presentation">
        {sectionFor?.(command) ?? command.category}
      </div>
    {/if}
    <CommandItem
      {command}
      selected={index === selectedIndex}
      showDescription={showDescriptions}
      onselect={() => onselect(index)}
      onexecute={() => onexecute(command)}
    />
  {/each}
</div>

<style>
  .results {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 0 11px 12px;
    scroll-padding: 8px;
  }
  .section-label {
    padding: 13px 13px 7px;
    font-size: var(--text-10);
    font-weight: 600;
    color: var(--muted);
    letter-spacing: 0.3px;
  }
</style>
