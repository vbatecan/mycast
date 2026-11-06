<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getFileStatus,
    setFilePreferences,
    chooseFileFolder,
    rebuildFileIndex,
    isNative,
    type FilePreferences,
  } from '../native';
  import { fileStatus } from '../stores/files';
  let draft = $state<FilePreferences | null>(null);
  let exclusions = $state('');
  let busy = $state(false);
  let error = $state('');
  let message = $state('');
  function load(value: FilePreferences | null) {
    draft = value
      ? { ...value, roots: [...value.roots], exclusions: [...value.exclusions] }
      : null;
    exclusions = value?.exclusions.join('\n') ?? '';
  }
  onMount(() => {
    if (isNative())
      void getFileStatus()
        .then((status) => {
          fileStatus.set(status);
          load(status.preferences);
        })
        .catch((reason) => (error = String(reason)));
  });
  async function choose(terminal = false) {
    busy = true;
    error = '';
    message = '';
    try {
      const path = await chooseFileFolder();
      if (path && draft) {
        if (terminal) draft.terminalDirectory = path;
        else if (
          !draft.roots.some((root) => root.toLowerCase() === path.toLowerCase())
        )
          draft.roots = [...draft.roots, path];
      }
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }
  async function save() {
    if (!draft) return;
    busy = true;
    error = '';
    message = '';
    try {
      const status = await setFilePreferences({
        ...draft,
        exclusions: exclusions
          .split(/\r?\n/)
          .map((value) => value.trim())
          .filter(Boolean),
      });
      fileStatus.set(status);
      load(status.preferences);
      message = 'Settings saved.';
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }
  async function rebuild() {
    busy = true;
    error = '';
    try {
      await rebuildFileIndex();
      message = 'Rebuilding file index…';
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }
</script>

<h2>Files &amp; Run</h2>
<p>
  Search file and folder names in your chosen folders. Use <strong>&gt;</strong> to
  run a command in a visible PowerShell terminal.
</p>
{#if draft}
  <div class="heading">
    <h3>Search folders</h3>
    <button disabled={busy} onclick={() => choose()}>Add folder</button>
  </div>
  {#each draft.roots as root, index}
    <div class="folder-row">
      <span title={root}>{root}</span><button
        aria-label={'Remove search folder ' + root}
        disabled={busy || draft.roots.length === 1}
        onclick={() => {
          if (draft)
            draft.roots = draft.roots.filter(
              (_, position) => position !== index,
            );
        }}>Remove</button
      >
    </div>
  {/each}
  <label class="check"
    ><input type="checkbox" bind:checked={draft.includeHidden} /> Include hidden and
    system files</label
  >
  <label class="field"
    >Excluded folder names, one per line<textarea
      rows="3"
      bind:value={exclusions}
      spellcheck="false"></textarea></label
  >
  <h3>PowerShell working folder</h3>
  <div class="folder-row">
    <span title={draft.terminalDirectory}>{draft.terminalDirectory}</span
    ><button disabled={busy} onclick={() => choose(true)}>Choose</button>
  </div>
  <p>
    Detected shell: {$fileStatus.shell || 'PowerShell'}. The terminal stays open
    after your command finishes. Mycast does not save raw Run commands.
  </p>
  <div class="actions">
    <button disabled={busy} onclick={save}>Save settings</button><button
      disabled={busy}
      onclick={() => load($fileStatus.preferences)}>Cancel edits</button
    ><button disabled={busy} onclick={rebuild}>Rebuild index</button>
  </div>
  <p class="index-status" role="status">
    {$fileStatus.state === 'indexing' || $fileStatus.state === 'reconciling'
      ? 'Indexing…'
      : $fileStatus.state === 'idle'
        ? 'Index starts when file search is first used.'
        : `${$fileStatus.indexed.toLocaleString()} files and folders indexed`}
  </p>
  {#if $fileStatus.warnings.length}<ul class="warnings">
      {#each $fileStatus.warnings as warning}<li>{warning}</li>{/each}
    </ul>{/if}
{:else}<p>
    File search and terminal execution are available in the desktop app.
  </p>{/if}
{#if error}<p class="error" role="alert">{error}</p>{/if}
{#if message}<p role="status">{message}</p>{/if}

<style>
  h2 {
    font-size: var(--text-16);
    font-weight: 600;
    margin: 0 0 6px;
  }
  h3 {
    font-size: var(--text-12);
    font-weight: 600;
    margin: 16px 0 8px;
  }
  p,
  .field,
  .check,
  .folder-row {
    font-size: var(--text-11);
    line-height: 1.5;
  }
  p {
    color: var(--secondary);
  }
  .heading,
  .folder-row,
  .actions {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .heading {
    justify-content: space-between;
  }
  .folder-row {
    padding: 7px 0;
    border-bottom: 1px solid var(--border);
  }
  .folder-row > span {
    flex: 1;
    min-width: 0;
    overflow-wrap: anywhere;
  }
  .check {
    display: flex;
    gap: 7px;
    align-items: center;
    margin: 12px 0;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 5px;
    color: var(--secondary);
  }
  textarea {
    width: 100%;
    font: inherit;
    resize: vertical;
    background: var(--surface);
    color: var(--text);
    padding: 7px;
    border: 1px solid var(--border);
    border-radius: 6px;
  }
  button {
    font-size: var(--text-10);
    padding: 6px 9px;
    border: 1px solid var(--border);
    border-radius: 5px;
    background: var(--surface);
    flex-shrink: 0;
  }
  .actions {
    flex-wrap: wrap;
    margin-top: 16px;
  }
  button:disabled {
    opacity: 0.45;
  }
  .error,
  .warnings {
    color: var(--danger);
    font-size: var(--text-10);
    overflow-wrap: anywhere;
  }
  .warnings {
    padding-left: 17px;
  }
</style>
