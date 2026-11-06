<script lang="ts">
  import {
    Pin,
    ArrowUp,
    ArrowDown,
    Plus,
    X,
    Tag,
    ShieldCheck,
  } from '@lucide/svelte';
  import CommandIcon from './CommandIcon.svelte';
  import {
    personalization,
    toggleFavorite,
    moveFavorite,
    setAliases,
    type SavedItem,
  } from '../stores/personalization';

  import { fileAction, getLauncherStatus, type LaunchMode } from '../native';
  import { onMount, tick } from 'svelte';
  let {
    item,
    onexecute,
    draft,
    ondraftchange,
  }: {
    item: SavedItem;
    onexecute: (
      command: import('../types/command').Command,
      mode?: LaunchMode,
    ) => Promise<void>;
    draft: string;
    ondraftchange: (value: string) => void;
  } = $props();
  let alias = $derived(draft);
  let windows = $state(false);
  let launching = $state(false);
  onMount(() => {
    void getLauncherStatus()
      .then((status) => (windows = status.platform === 'windows'))
      .catch(() => {});
  });
  const canAdmin = $derived(
    windows &&
      !!item.command &&
      (item.command.canRunAsAdmin ||
        (item.command.action.type === 'native' &&
          item.command.action.name === 'notepad')),
  );
  async function launch(mode: LaunchMode = 'normal') {
    if (!item.command || launching) return;
    launching = true;
    try {
      await onexecute(item.command, mode);
    } finally {
      launching = false;
    }
  }
  async function fileControl(action: 'reveal' | 'copy-path') {
    if (item.command?.action.type !== 'file' || launching) return;
    launching = true;
    try {
      notice = await fileAction(item.command.action.fileId, action);
      error = '';
    } catch (reason) {
      error = String(reason);
    } finally {
      launching = false;
    }
  }
  let error = $state('');
  let notice = $state('');
  const position = $derived($personalization.favorites.indexOf(item.key));

  function updateAliases(aliases: string[]) {
    error = setAliases(item.command ?? item.key, aliases) ?? '';
    if (error) {
      notice = '';
      return false;
    }
    notice = 'Aliases saved.';
    return true;
  }
  async function removeAlias(value: string) {
    if (updateAliases(item.aliases.filter((entry) => entry !== value))) {
      await tick();
      document.getElementById('new-command-alias')?.focus();
    }
  }
  function addAlias(event: SubmitEvent) {
    event.preventDefault();
    if (!alias.trim()) return;
    if (updateAliases([...item.aliases, alias.trim()])) ondraftchange('');
  }
</script>

<section aria-labelledby="command-detail-title">
  <header class="detail-header">
    <div class="detail-icon">
      <CommandIcon
        name={item.command?.icon ??
          (item.category === 'Files' ? 'file-text' : 'app-window')}
        applicationId={item.command?.action.type === 'application'
          ? item.command.action.applicationId
          : undefined}
      />
    </div>
    <div class="detail-title">
      <span class="eyebrow">{item.category}</span>
      <h2 id="command-detail-title">{item.label}</h2>
    </div>
  </header>
  {#if item.command?.description}<p class="description">
      {item.command.description}
    </p>{/if}
  {#if !item.available}<p class="unavailable">
      Currently unavailable. Your favorites and aliases stay saved.
    </p>{/if}

  <div class="launch-controls">
    <button
      disabled={!item.available || launching || !windows}
      onclick={() => launch()}>Open</button
    >
    {#if item.category === 'Applications'}<button
        disabled={!item.available || launching || !canAdmin}
        title={canAdmin
          ? 'Request administrator privileges through Windows'
          : 'Available for supported Windows desktop applications only'}
        onclick={() => launch('administrator')}
        ><ShieldCheck size={13} />Run as administrator</button
      >{/if}
    {#if item.command?.action.type === 'file'}<button
        disabled={launching || !windows}
        onclick={() => fileControl('reveal')}>Reveal</button
      ><button
        disabled={launching || !windows}
        onclick={() => fileControl('copy-path')}>Copy path</button
      >{/if}
  </div>
  <div class="control-section">
    <div class="favorite-row">
      <div>
        <h3>Favorite</h3>
        <p>Keep this item close at hand.</p>
      </div>
      <button
        class:active={item.favorite}
        aria-pressed={item.favorite}
        onclick={() => toggleFavorite(item.command ?? item.key)}
        ><Pin size={13} />{item.favorite ? 'Pinned' : 'Pin'}</button
      >
    </div>
    {#if item.favorite}
      <div class="favorite-order">
        <span
          >Position {position + 1} of {$personalization.favorites.length}</span
        >
        <div class="order-buttons">
          <button
            aria-label={'Move ' + item.label + ' up'}
            title="Move up"
            disabled={position <= 0}
            onclick={() => moveFavorite(item.key, -1)}
            ><ArrowUp size={13} /></button
          >
          <button
            aria-label={'Move ' + item.label + ' down'}
            title="Move down"
            disabled={position === $personalization.favorites.length - 1}
            onclick={() => moveFavorite(item.key, 1)}
            ><ArrowDown size={13} /></button
          >
        </div>
      </div>
    {/if}
  </div>

  <div class="control-section aliases-section">
    <div class="section-heading">
      <h3>Aliases</h3>
      <span>{item.aliases.length} / 10</span>
    </div>
    <p>Find this item by a name that makes sense to you.</p>
    <div class="alias-list">
      {#each item.aliases as value}
        <span class="alias-chip"
          ><Tag size={11} /><span>{value}</span><button
            aria-label={'Remove alias ' + value}
            title="Remove alias"
            onclick={() => removeAlias(value)}><X size={12} /></button
          ></span
        >
      {:else}<span class="no-aliases">No aliases yet.</span>{/each}
    </div>
    <form onsubmit={addAlias}>
      <label for="new-command-alias">New alias</label>
      <div class="alias-entry">
        <input
          id="new-command-alias"
          value={alias}
          oninput={(event) => {
            ondraftchange(event.currentTarget.value);
            error = '';
            notice = '';
          }}
          placeholder="e.g. worknotes"
          autocomplete="off"
          spellcheck="false"
          aria-describedby="alias-help"
          disabled={item.aliases.length >= 10}
        />
        <button
          class="add-alias"
          type="submit"
          disabled={!alias.trim() || item.aliases.length >= 10}
          ><Plus size={13} />Add</button
        >
      </div>
      <p id="alias-help" class="help">
        Up to 32 characters, including a letter. No spaces. Each alias must be
        unique.
      </p>
    </form>
    {#if error}<p class="error" role="alert">{error}</p>{/if}
    <p class="notice" role="status">{notice}</p>
  </div>
</section>

<style>
  .launch-controls {
    display: flex;
    flex-wrap: wrap;
    gap: 7px;
    margin-top: 12px;
  }
  .detail-header {
    display: flex;
    align-items: center;
    gap: 11px;
    margin-bottom: 10px;
  }
  .detail-icon {
    width: 42px;
    height: 42px;
    flex-shrink: 0;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--surface);
    color: var(--secondary);
  }
  .detail-title {
    min-width: 0;
  }
  .eyebrow {
    display: block;
    color: var(--muted);
    font-size: var(--text-9);
    margin-bottom: 3px;
  }
  h2 {
    margin: 0;
    font-size: var(--text-17);
    font-weight: 600;
    line-height: 1.35;
    overflow-wrap: anywhere;
  }
  h3 {
    margin: 0;
    font-size: var(--text-12);
    font-weight: 550;
  }
  p {
    margin: 4px 0 0;
    color: var(--muted);
    font-size: var(--text-10);
    line-height: 1.5;
  }
  .description {
    overflow-wrap: anywhere;
    color: var(--secondary);
  }
  .unavailable {
    margin-top: 8px;
    color: var(--secondary);
  }
  .control-section {
    padding: 17px 0;
    border-bottom: 1px solid var(--border);
  }
  .favorite-row,
  .section-heading,
  .favorite-order {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 6px 10px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    color: var(--secondary);
    font-size: var(--text-11);
    flex-shrink: 0;
  }
  button:hover:not(:disabled) {
    background: var(--hover);
    color: var(--text);
  }
  button.active,
  .add-alias {
    background: var(--accent-soft);
    color: var(--accent);
    border-color: color-mix(in srgb, var(--accent) 30%, var(--border));
  }
  button:disabled {
    opacity: 0.4;
  }
  .favorite-order {
    margin-top: 11px;
    padding: 8px 10px;
    border-radius: 6px;
    background: color-mix(in srgb, var(--surface) 65%, transparent);
    color: var(--secondary);
    font-size: var(--text-10);
  }
  .order-buttons {
    display: flex;
    gap: 5px;
  }
  .order-buttons button {
    padding: 5px;
  }
  .section-heading > span {
    color: var(--muted);
    font-size: var(--text-10);
    font-variant-numeric: tabular-nums;
  }
  .alias-list {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin: 12px 0;
  }
  .alias-chip {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    max-width: 100%;
    padding: 3px 4px 3px 7px;
    border: 1px solid var(--border);
    background: var(--surface);
    border-radius: 5px;
    color: var(--secondary);
    font-size: var(--text-11);
  }
  .alias-chip > span {
    overflow-wrap: anywhere;
    min-width: 0;
  }
  .alias-chip button {
    padding: 2px;
    border: 0;
    background: transparent;
  }
  .no-aliases {
    color: var(--muted);
    font-size: var(--text-11);
  }
  label {
    display: block;
    margin-bottom: 6px;
    color: var(--secondary);
    font-size: var(--text-10);
  }
  .alias-entry {
    display: flex;
    gap: 7px;
  }
  input {
    flex: 1;
    width: 100%;
    min-width: 0;
    padding: 8px 10px;
    background: var(--surface);
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: 6px;
    font-size: var(--text-11);
  }
  input:focus-visible {
    outline: 1px solid var(--accent);
  }
  .help {
    margin-top: 7px;
    font-size: var(--text-9);
  }
  .error {
    color: var(--danger);
    margin-top: 8px;
  }
  .notice {
    color: var(--secondary);
    min-height: 15px;
  }
  .aliases-section {
    border-bottom: 0;
  }
</style>
