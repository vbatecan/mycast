<script lang="ts">
  import { onDestroy, tick } from 'svelte';
  import {
    Check,
    Copy,
    Download,
    FileUp,
    Paintbrush,
    Pencil,
    Plus,
    Trash2,
  } from '@lucide/svelte';
  import {
    activeTheme,
    availableThemes,
    setThemeMode,
    setSystemTheme,
    builtInThemes,
    deleteTheme,
    exportTheme,
    importTheme,
    resetPreview,
    selectTheme,
    themes,
    themeNameLimit,
    type Theme,
    type ThemeDraft,
  } from '../stores/themes';
  import ThemeEditor from './ThemeEditor.svelte';

  let {
    oneditingchange = () => {},
  }: { oneditingchange?: (editing: boolean) => void } = $props();
  let draft = $state<ThemeDraft | null>(null);
  let pendingDelete = $state<string | null>(null);
  let notice = $state('');
  let isError = $state(false);
  let fileInput = $state<HTMLInputElement>();
  let createButton = $state<HTMLButtonElement>();

  $effect(() => oneditingchange(draft !== null));
  onDestroy(resetPreview);

  export function cancelEditing(): boolean {
    if (!draft) return false;
    resetPreview();
    draft = null;
    void tick().then(() => createButton?.focus());
    return true;
  }

  function edit(theme: Theme, duplicate = false) {
    notice = '';
    pendingDelete = null;
    draft = {
      ...(duplicate ? {} : { id: theme.id }),
      name: duplicate
        ? `${theme.name.slice(0, themeNameLimit - 5)} copy`
        : theme.name,
      scheme: theme.scheme,
      colors: { ...theme.colors },
    };
  }

  function create() {
    notice = '';
    pendingDelete = null;
    draft = {
      name: 'My theme',
      scheme: $activeTheme.scheme,
      colors: { ...$activeTheme.colors },
    };
  }

  function done() {
    draft = null;
    notice = 'Theme saved. Make yourself at home.';
    isError = false;
    void tick().then(() => createButton?.focus());
  }

  async function readTheme(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    notice = '';
    isError = false;
    try {
      if (file.size > 8 * 1024)
        throw new Error('Choose a theme JSON file smaller than 8 KB.');
      const result = importTheme(await file.text());
      if (!result.ok) throw new Error(result.error);
      selectTheme(result.theme.id);
      notice = `Imported ${result.theme.name}.`;
    } catch (reason) {
      isError = true;
      notice = reason instanceof Error ? reason.message : String(reason);
    } finally {
      input.value = '';
    }
  }

  function download(theme: Theme) {
    try {
      const url = URL.createObjectURL(
        new Blob([exportTheme(theme)], { type: 'application/json' }),
      );
      const anchor = document.createElement('a');
      anchor.href = url;
      anchor.download = `${
        theme.name
          .toLowerCase()
          .replace(/[^a-z0-9]+/g, '-')
          .replace(/^-|-$/g, '') || 'mycast-theme'
      }.json`;
      document.body.appendChild(anchor);
      anchor.click();
      anchor.remove();
      setTimeout(() => URL.revokeObjectURL(url), 1000);
      notice = 'Theme download requested.';
      isError = false;
    } catch (reason) {
      isError = true;
      notice = reason instanceof Error ? reason.message : String(reason);
    }
  }

  function remove(id: string) {
    deleteTheme(id);
    pendingDelete = null;
    notice = 'Custom theme deleted.';
    isError = false;
  }
</script>

{#if draft}
  <ThemeEditor {draft} ondone={done} oncancel={cancelEditing} />
{:else}
  <div class="picker-heading">
    <div>
      <p class="eyebrow">YOUR SPACE, YOUR PALETTE</p>
      <h2>Make it feel like you.</h2>
    </div>
    <div class="heading-actions">
      <button class="text-button" onclick={() => fileInput?.click()}
        ><FileUp size={12} /> Import</button
      ><button class="create-button" bind:this={createButton} onclick={create}
        ><Plus size={12} /> Create theme</button
      >
    </div>
  </div>
  <input
    bind:this={fileInput}
    type="file"
    accept=".json,application/json"
    onchange={readTheme}
    class="file-input"
    tabindex="-1"
    aria-label="Import a theme JSON file"
  />

  <div class="theme-mode-controls">
    <label
      >Theme mode
      <select
        value={$themes.mode}
        onchange={(event) =>
          setThemeMode(event.currentTarget.value as 'manual' | 'system')}
      >
        <option value="manual">Manual</option><option value="system"
          >Follow Windows</option
        >
      </select>
    </label>
    {#if $themes.mode === 'system'}
      <div class="system-pair">
        {#each ['dark', 'light'] as scheme}
          <label
            >{scheme === 'dark' ? 'Dark theme' : 'Light theme'}
            <select
              value={scheme === 'dark' ? $themes.darkId : $themes.lightId}
              onchange={(event) =>
                setSystemTheme(
                  scheme as 'dark' | 'light',
                  event.currentTarget.value,
                )}
            >
              {#each $availableThemes.filter((theme) => theme.scheme === scheme) as theme}
                <option value={theme.id}>{theme.name}</option>
              {/each}
            </select>
          </label>
        {/each}
      </div>
    {/if}
    <p>Choosing a theme below selects Manual mode.</p>
  </div>

  <div class="preset-grid" aria-label="Built-in themes">
    {#each builtInThemes as theme}
      <div class="theme-card" class:selected={$themes.activeId === theme.id}>
        <button
          class="preset-button"
          aria-pressed={$themes.activeId === theme.id}
          onclick={() => {
            selectTheme(theme.id);
            notice = '';
          }}
          aria-label={`Use ${theme.name} theme`}
        >
          <span
            class="miniature"
            style={`--preview-bg:${theme.colors.background};--preview-surface:${theme.colors.surface};--preview-selected:${theme.colors.selected};--preview-text:${theme.colors.text};--preview-muted:${theme.colors.muted};--preview-accent:${theme.colors.accent};--preview-border:${theme.colors.border}`}
            aria-hidden="true"
          >
            <span class="mini-search"
              ><span class="mini-search-icon"></span><span
                class="mini-placeholder"
              ></span></span
            >
            <span class="mini-command active"
              ><span class="mini-app"></span><span class="mini-line"
              ></span><span class="mini-key"></span></span
            >
            <span class="mini-command"
              ><span class="mini-app"></span><span class="mini-line short"
              ></span></span
            >
          </span>
          <span class="theme-name"
            >{theme.name}{#if $themes.activeId === theme.id}<Check
                size={12}
              />{/if}</span
          >
        </button>
        <div class="preset-actions">
          <span>{theme.scheme === 'light' ? 'Light' : 'Dark'}</span><button
            title={`Customize ${theme.name}`}
            aria-label={`Duplicate ${theme.name}`}
            onclick={() => edit(theme, true)}><Copy size={11} /></button
          ><button
            title={`Export ${theme.name}`}
            aria-label={`Export ${theme.name}`}
            onclick={() => download(theme)}><Download size={11} /></button
          >
        </div>
      </div>
    {/each}
  </div>

  <div class="custom-heading">
    <span>Custom themes</span><span>{$themes.customThemes.length}</span>
  </div>
  {#if $themes.customThemes.length}
    <div class="custom-list">
      {#each $themes.customThemes as theme (theme.id)}
        <div class="custom-row" class:active={$themes.activeId === theme.id}>
          <button
            class="custom-select"
            aria-pressed={$themes.activeId === theme.id}
            onclick={() => selectTheme(theme.id)}
            ><span class="swatches" aria-hidden="true"
              ><i
                style={`background:${theme.colors.background};border-color:${theme.colors.border}`}
              ></i><i
                style={`background:${theme.colors.selected};border-color:${theme.colors.border}`}
              ></i><i
                style={`background:${theme.colors.accent};border-color:${theme.colors.border}`}
              ></i></span
            ><span class="custom-name">{theme.name}</span
            >{#if $themes.activeId === theme.id}<Check size={12} />{/if}</button
          >
          {#if pendingDelete === theme.id}<span class="delete-confirm"
              >Delete theme?</span
            ><button class="confirm-button" onclick={() => remove(theme.id)}
              >Delete</button
            ><button
              class="cancel-delete"
              onclick={() => (pendingDelete = null)}>Cancel</button
            >
          {:else}<div class="row-actions">
              <button
                title="Edit theme"
                aria-label={`Edit ${theme.name}`}
                onclick={() => edit(theme)}><Pencil size={12} /></button
              ><button
                title="Duplicate theme"
                aria-label={`Duplicate ${theme.name}`}
                onclick={() => edit(theme, true)}><Copy size={12} /></button
              ><button
                title="Export theme"
                aria-label={`Export ${theme.name}`}
                onclick={() => download(theme)}><Download size={12} /></button
              ><button
                title="Delete theme"
                aria-label={`Delete ${theme.name}`}
                onclick={() => (pendingDelete = theme.id)}
                ><Trash2 size={12} /></button
              >
            </div>{/if}
        </div>
      {/each}
    </div>
  {:else}<div class="custom-empty">
      <Paintbrush size={14} /><span
        >A favorite color? Start with a theme and make it yours.</span
      >
    </div>{/if}
  {#if notice}<p
      class="notice"
      class:error={isError}
      role={isError ? 'alert' : 'status'}
    >
      {notice}
    </p>{/if}
{/if}

<style>
  .theme-mode-controls {
    margin-bottom: 16px;
  }
  .theme-mode-controls label {
    display: flex;
    gap: 8px;
    align-items: center;
    justify-content: space-between;
    font-size: var(--text-11);
    color: var(--secondary);
  }
  .theme-mode-controls select {
    max-width: 220px;
    background: var(--surface);
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: 5px;
    padding: 5px 7px;
    font-size: var(--text-11);
  }
  .theme-mode-controls p {
    font-size: var(--text-10);
    color: var(--muted);
    margin: 8px 0 0;
  }
  .system-pair {
    display: grid;
    gap: 8px;
    margin-top: 10px;
  }

  .picker-heading {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 10px;
    margin-bottom: 15px;
  }
  .eyebrow {
    color: var(--muted);
    font-size: var(--text-8);
    font-weight: 600;
    letter-spacing: 0.9px;
    margin: 0 0 6px;
  }
  h2 {
    margin: 0;
    font-size: var(--text-16);
    font-weight: 550;
    letter-spacing: -0.3px;
  }
  .heading-actions {
    display: flex;
    gap: 6px;
  }
  .heading-actions button {
    display: flex;
    align-items: center;
    gap: 5px;
    height: 27px;
    border: 1px solid var(--border);
    border-radius: 5px;
    padding: 0 8px;
    font: inherit;
    font-size: var(--text-10);
    white-space: nowrap;
  }
  .text-button {
    background: transparent;
    color: var(--secondary);
  }
  .create-button {
    background: var(--surface);
    color: var(--text);
  }
  button:hover {
    background: var(--hover);
  }
  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  .file-input {
    display: none;
  }
  .preset-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 9px;
  }
  .theme-card {
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
    background: var(--surface);
  }
  .theme-card.selected {
    border-color: var(--accent);
  }
  .preset-button {
    display: block;
    width: 100%;
    padding: 0;
    background: none;
    border: 0;
    text-align: left;
  }
  .miniature {
    display: flex;
    flex-direction: column;
    gap: 3px;
    margin: 7px 7px 0;
    padding: 0 7px 4px;
    border: 1px solid var(--preview-border);
    background: var(--preview-bg);
    border-radius: 5px;
    height: 60px;
    overflow: hidden;
  }
  .mini-search {
    height: 20px;
    display: flex;
    align-items: center;
    gap: 5px;
    border-bottom: 1px solid var(--preview-border);
    margin: 0 -7px 1px;
    padding: 0 8px;
  }
  .mini-search-icon {
    width: 5px;
    height: 5px;
    border: 1px solid var(--preview-muted);
    border-radius: 50%;
  }
  .mini-placeholder {
    height: 2px;
    width: 50%;
    background: var(--preview-muted);
    opacity: 0.6;
    border-radius: 2px;
  }
  .mini-command {
    height: 13px;
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 0 3px;
    border-radius: 3px;
  }
  .mini-command.active {
    background: var(--preview-selected);
  }
  .mini-app {
    width: 6px;
    height: 6px;
    background: var(--preview-accent);
    border-radius: 2px;
    opacity: 0.8;
  }
  .mini-command:not(.active) .mini-app {
    background: var(--preview-muted);
  }
  .mini-line {
    width: 48%;
    height: 2px;
    background: var(--preview-text);
    border-radius: 2px;
    opacity: 0.65;
  }
  .mini-line.short {
    width: 34%;
    opacity: 0.45;
  }
  .mini-key {
    width: 7px;
    height: 5px;
    margin-left: auto;
    border: 1px solid var(--preview-muted);
    border-radius: 1px;
    opacity: 0.45;
  }
  .theme-name {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 7px 9px 1px;
    color: var(--text);
    font-size: var(--text-10);
    font-weight: 500;
  }
  .theme-name :global(svg) {
    color: var(--accent);
  }
  .preset-actions {
    display: flex;
    align-items: center;
    gap: 1px;
    padding: 0 6px 3px 9px;
  }
  .preset-actions > span {
    flex: 1;
    color: var(--muted);
    font-size: var(--text-8);
  }
  .preset-actions button,
  .row-actions button {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    padding: 0;
    background: transparent;
    border: 0;
    border-radius: 4px;
    color: var(--muted);
  }
  .preset-actions button:hover,
  .row-actions button:hover {
    background: var(--hover);
    color: var(--text);
  }
  .custom-heading {
    margin-top: 20px;
    margin-bottom: 8px;
    display: flex;
    gap: 8px;
    align-items: center;
    color: var(--secondary);
    font-size: var(--text-10);
  }
  .custom-heading > span:last-child {
    color: var(--muted);
    font-size: var(--text-9);
  }
  .custom-row {
    display: flex;
    align-items: center;
    gap: 4px;
    min-height: 40px;
    border-bottom: 1px solid var(--border);
  }
  .custom-select {
    display: flex;
    align-items: center;
    gap: 10px;
    flex: 1;
    min-width: 0;
    height: 33px;
    padding: 0 5px 0 0;
    background: transparent;
    border: 0;
    text-align: left;
    color: var(--secondary);
    border-radius: 4px;
  }
  .custom-row.active .custom-select {
    color: var(--text);
  }
  .custom-select :global(svg) {
    color: var(--accent);
    flex-shrink: 0;
  }
  .custom-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: var(--text-11);
  }
  .swatches {
    display: flex;
    padding-left: 1px;
    flex-shrink: 0;
  }
  .swatches i {
    width: 14px;
    height: 20px;
    border: 1px solid;
    border-radius: 4px;
    margin-left: -1px;
  }
  .row-actions {
    display: flex;
    gap: 3px;
  }
  .custom-empty {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--muted);
    font-size: var(--text-10);
    padding: 10px 0 12px;
  }
  .delete-confirm {
    font-size: var(--text-9);
    color: var(--secondary);
    white-space: nowrap;
    margin: 0 3px;
  }
  .confirm-button,
  .cancel-delete {
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--surface);
    padding: 4px 6px;
    color: var(--text);
    font: inherit;
    font-size: var(--text-9);
  }
  .cancel-delete {
    background: transparent;
    color: var(--muted);
  }
  .notice {
    font-size: var(--text-10);
    color: var(--secondary);
    line-height: 1.5;
    padding: 7px 0 0;
    margin: 0;
  }
  .notice.error {
    color: var(--accent);
  }
  @media (max-width: 520px) {
    .preset-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
    .picker-heading {
      align-items: flex-start;
    }
    .heading-actions {
      flex-direction: column;
    }
  }
</style>
