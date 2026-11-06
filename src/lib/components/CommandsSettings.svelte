<script lang="ts">
  import { Search, Pin } from '@lucide/svelte';
  import { managementCatalogue as commandCatalogue } from '../stores/files';
  import { personalization, savedItems } from '../stores/personalization';
  import CommandIcon from './CommandIcon.svelte';
  import CommandDetails from './CommandDetails.svelte';

  let query = $state('');
  let savedOnly = $state(false);
  let selectedKey = $state('');
  const entries = $derived(
    savedItems($commandCatalogue, $personalization).filter(
      (item) =>
        (!savedOnly || item.favorite || item.aliases.length > 0) &&
        `${item.label} ${item.aliases.join(' ')}`
          .toLocaleLowerCase()
          .includes(query.toLocaleLowerCase()),
    ),
  );
  const selected = $derived(
    entries.find((item) => item.key === selectedKey) ?? entries[0],
  );
  $effect(() => {
    if (selected && selectedKey !== selected.key) selectedKey = selected.key;
  });

  function navigate(event: KeyboardEvent, index: number) {
    if (!['ArrowUp', 'ArrowDown', 'Home', 'End'].includes(event.key)) return;
    event.preventDefault();
    event.stopPropagation();
    const next =
      event.key === 'Home'
        ? 0
        : event.key === 'End'
          ? entries.length - 1
          : (index + (event.key === 'ArrowDown' ? 1 : -1) + entries.length) %
            entries.length;
    selectedKey = entries[next].key;
    const list = (event.currentTarget as HTMLButtonElement).parentElement;
    const button =
      list?.querySelectorAll<HTMLButtonElement>('[role="option"]')[next];
    button?.focus({ preventScroll: true });
    button?.scrollIntoView({ block: 'nearest' });
  }
</script>

<div class="commands-settings">
  <aside class="command-sidebar" aria-label="Command selection">
    <div class="sidebar-header">
      <h2>Commands</h2>
      <p>Select an item to customize it.</p>
      <div class="search-field">
        <Search size={14} aria-hidden="true" /><input
          aria-label="Search saved commands"
          placeholder="Search commands…"
          bind:value={query}
        />
      </div>
      <label class="saved-only"
        ><input type="checkbox" bind:checked={savedOnly} /> Favorites and aliases
        only</label
      >
    </div>
    <div class="managed-items" role="listbox" aria-label="Commands">
      {#each entries as item, index (item.key)}
        <button
          class="command-item"
          class:selected={selected?.key === item.key}
          role="option"
          aria-selected={selected?.key === item.key}
          tabindex={selected?.key === item.key ? 0 : -1}
          title={item.label}
          onclick={() => (selectedKey = item.key)}
          onkeydown={(event) => navigate(event, index)}
        >
          <span class="item-icon"
            ><CommandIcon
              name={item.command?.icon ??
                (item.category === 'Files' ? 'file-text' : 'app-window')}
              applicationId={item.command?.action.type === 'application'
                ? item.command.action.applicationId
                : undefined}
            /></span
          >
          <span class="item-text"
            ><strong>{item.label}</strong><small
              >{item.available ? item.category : 'Unavailable'}</small
            ></span
          >
          {#if item.favorite}<Pin
              size={11}
              class="pin-indicator"
              aria-label="Pinned"
            />{/if}
        </button>
      {:else}<p class="empty-list">No matching commands.</p>{/each}
    </div>
    <div class="list-count">
      {entries.length}
      {entries.length === 1 ? 'item' : 'items'}
    </div>
  </aside>
  <div class="command-controls">
    {#if selected}
      {#key selected.key}<CommandDetails item={selected} />{/key}
    {:else}
      <div class="empty-detail">
        <Search size={25} strokeWidth={1.3} />
        <h2>No matching commands</h2>
        <p>Try another search or turn off the favorites filter.</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .commands-settings {
    display: grid;
    grid-template-columns: 200px minmax(0, 1fr);
    width: 100%;
    min-width: 0;
    min-height: 0;
  }
  .command-sidebar {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    border-right: 1px solid var(--border);
    padding-right: 12px;
  }
  .sidebar-header {
    flex-shrink: 0;
  }
  h2 {
    margin: 0;
    font-size: var(--text-16);
    font-weight: 600;
  }
  p {
    color: var(--muted);
    font-size: var(--text-10);
    line-height: 1.5;
    margin: 5px 0 12px;
  }
  .search-field {
    display: flex;
    align-items: center;
    gap: 6px;
    border: 1px solid var(--border);
    background: var(--surface);
    border-radius: 6px;
    padding: 7px 8px;
    color: var(--muted);
  }
  .search-field:focus-within {
    border-color: var(--accent);
  }
  .search-field input {
    width: 100%;
    min-width: 0;
    border: 0;
    outline: none;
    color: var(--text);
    background: transparent;
    font-size: var(--text-11);
  }
  .saved-only {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--secondary);
    font-size: var(--text-10);
    margin: 10px 0;
    line-height: 1.5;
  }
  .saved-only input {
    flex-shrink: 0;
  }
  .managed-items {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding-bottom: 8px;
  }
  .command-item {
    display: flex;
    align-items: center;
    gap: 8px;
    text-align: left;
    width: 100%;
    min-width: 0;
    padding: 8px 6px;
    margin-bottom: 3px;
    border: 1px solid transparent;
    border-radius: 6px;
    background: transparent;
  }
  .command-item:hover {
    background: var(--hover);
  }
  .command-item.selected {
    background: color-mix(in srgb, var(--accent) 10%, var(--surface));
    border-color: color-mix(in srgb, var(--accent) 30%, var(--border));
  }
  .item-icon {
    display: block;
    width: 29px;
    height: 29px;
    flex-shrink: 0;
    color: var(--secondary);
  }
  .item-text {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
    flex: 1;
  }
  strong {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
    font-size: var(--text-11);
    font-weight: 500;
  }
  small {
    color: var(--muted);
    font-size: var(--text-9);
  }
  :global(.pin-indicator) {
    color: var(--accent);
    flex-shrink: 0;
  }
  .list-count {
    flex-shrink: 0;
    padding: 8px 6px;
    font-size: var(--text-9);
    color: var(--muted);
    border-top: 1px solid var(--border);
  }
  .command-controls {
    min-width: 0;
    overflow-y: auto;
    padding: 0 0 20px 22px;
  }
  .empty-detail {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    text-align: center;
    color: var(--muted);
    height: 100%;
    padding: 20px;
    gap: 8px;
  }
  .empty-detail h2 {
    font-size: var(--text-14);
    color: var(--secondary);
  }
  .empty-list {
    padding: 12px 6px;
  }
  @media (max-width: 550px) {
    .commands-settings {
      grid-template-columns: 155px minmax(0, 1fr);
    }
    .command-controls {
      padding-left: 12px;
    }
  }
</style>
