<script lang="ts">
  import { Search, X } from '@lucide/svelte';
  let {
    value = $bindable(''),
    input = $bindable<HTMLInputElement | undefined>(),
    activeId,
    onkeydown,
  }: {
    value: string;
    input?: HTMLInputElement;
    activeId?: string;
    onkeydown: (event: KeyboardEvent) => void;
  } = $props();
</script>

<div class="search-header">
  <Search size={21} strokeWidth={1.7} aria-hidden="true" />
  <input
    bind:this={input}
    bind:value
    {onkeydown}
    aria-label="Search apps, commands, or files; use > to run PowerShell"
    role="combobox"
    aria-autocomplete="list"
    aria-expanded="true"
    aria-controls="command-results"
    aria-activedescendant={activeId}
    placeholder="Search apps, commands, files… or > to run"
    autocomplete="off"
    spellcheck="false"
    maxlength={value.trimStart().startsWith('>') ? 8193 : 512}
  />
  {#if value}
    <button
      class="clear"
      aria-label="Clear search"
      tabindex="-1"
      onclick={() => {
        value = '';
        input?.focus();
      }}><X size={15} /></button
    >
  {/if}
</div>

<style>
  .search-header {
    display: flex;
    align-items: center;
    gap: 14px;
    height: 73px;
    flex-shrink: 0;
    padding: 0 23px;
    border-bottom: 1px solid var(--border);
    color: var(--muted);
  }
  input {
    width: 100%;
    min-width: 0;
    outline: none;
    border: 0;
    background: transparent;
    color: var(--text);
    font-size: var(--text-17);
    font-weight: 400;
    letter-spacing: -0.25px;
  }
  input::placeholder {
    color: var(--muted);
  }
  .clear {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    padding: 0;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 5px;
    color: var(--secondary);
  }
</style>
