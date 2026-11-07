<script lang="ts">
  import {
    SearchX,
    Star,
    Clock,
    FolderSearch,
    CircleAlert,
    Search,
  } from '@lucide/svelte';
  import type { CommandScope } from '../stores/applications';
  import { preferences } from '../stores/preferences';
  import { shortcutLabel } from '../shortcuts';
  import KeyboardHint from './KeyboardHint.svelte';
  let {
    query,
    scope = 'all',
    state = 'empty',
    onsearchall,
    onmanagefiles,
    onretry,
  }: {
    query: string;
    scope?: CommandScope;
    state?:
      | 'empty'
      | 'searching'
      | 'indexing'
      | 'no-folders'
      | 'file-error'
      | 'application-error'
      | 'applications-loading'
      | 'desktop';
    onsearchall?: () => void;
    onmanagefiles?: () => void;
    onretry?: () => void;
  } = $props();
  const text = $derived(
    query.trim().length > 45 ? query.trim().slice(0, 45) + '…' : query.trim(),
  );
  const content = $derived.by(() => {
    if (state === 'applications-loading')
      return {
        icon: Search,
        title: 'Finding your applications',
        body: 'Results will appear here when your applications are ready.',
      };
    if (state === 'file-error')
      return {
        icon: CircleAlert,
        title: 'File search couldn’t finish',
        body: 'Try again or check your search folders in Settings.',
      };
    if (state === 'application-error')
      return {
        icon: CircleAlert,
        title: 'Applications couldn’t refresh',
        body: 'Try refreshing your installed applications again.',
      };
    if (state === 'no-folders')
      return {
        icon: FolderSearch,
        title: 'Choose where to search',
        body: 'Add search folders in Settings to find your files here.',
      };
    if (state === 'indexing')
      return {
        icon: FolderSearch,
        title: 'Preparing file search',
        body: 'Results will appear as your folders are indexed.',
      };
    if (state === 'searching')
      return {
        icon: Search,
        title: 'Finding your files',
        body: 'Results will appear here when the search finishes.',
      };
    if (state === 'desktop')
      return {
        icon: FolderSearch,
        title: 'Find files in the desktop app',
        body: 'Open Mycast on your computer to search your folders.',
      };
    if (!text && scope === 'favorites')
      return {
        icon: Star,
        title: 'Your favorites go here',
        body: 'Pin a result to keep it close.',
      };
    if (!text && scope === 'recents')
      return {
        icon: Clock,
        title: 'Your recent items go here',
        body: 'Apps and files you open will appear here.',
      };
    if (scope === 'files')
      return {
        icon: FolderSearch,
        title: text ? 'No matching files' : 'No files to show',
        body: text
          ? `Nothing matches “${text}” in your search folders.`
          : 'Try typing part of a filename or check your search folders.',
      };
    const names = {
      applications: 'applications',
      commands: 'commands',
      favorites: 'favorites',
      recents: 'recent items',
      all: 'results',
    };
    if (text)
      return {
        icon: SearchX,
        title:
          scope === 'all'
            ? `No matches for “${text}”`
            : `No matching ${names[scope]}`,
        body:
          scope === 'all'
            ? 'Try a shorter name or different spelling.'
            : `Nothing matches “${text}” in this filter. Try a shorter name or search All.`,
      };
    return {
      icon: Search,
      title:
        scope === 'all' ? 'Ready when you are' : `No ${names[scope]} to show`,
      body: 'Search by name or try another filter.',
    };
  });
  const Icon = $derived(content.icon);
  const showFolders = $derived(
    !!onmanagefiles &&
      (state === 'no-folders' ||
        state === 'file-error' ||
        state === 'indexing' ||
        (state === 'empty' && scope === 'files')),
  );
</script>

<div class="empty" role="status">
  <span class="empty-icon" aria-hidden="true"
    ><Icon size={25} strokeWidth={1.4} /></span
  >
  <h2>{content.title}</h2>
  <p>{content.body}</p>
  {#if state === 'empty' && !text && scope === 'favorites'}
    <span class="pin-hint"
      ><KeyboardHint
        keys={shortcutLabel($preferences.shortcuts.pin).split('+')}
      /> Pin selected result</span
    >
  {/if}
  {#if (onretry && ['file-error', 'application-error'].includes(state)) || showFolders || (state === 'empty' && scope !== 'all' && text)}
    <div class="actions">
      {#if onretry && ['file-error', 'application-error'].includes(state)}<button
          onclick={onretry}>Retry</button
        >{/if}
      {#if showFolders}<button onclick={onmanagefiles}>Manage folders</button
        >{/if}
      {#if state === 'empty' && scope !== 'all' && text && onsearchall}<button
          onclick={onsearchall}>Search All</button
        >{/if}
    </div>
  {/if}
</div>

<style>
  .empty {
    flex: 1;
    min-height: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-direction: column;
    padding: 25px;
    overflow-y: auto;
  }
  .empty-icon {
    display: grid;
    place-items: center;
    width: 50px;
    height: 50px;
    border: 1px solid var(--border);
    background: var(--surface);
    border-radius: 13px;
    color: var(--secondary);
    margin-bottom: 17px;
  }
  h2 {
    font-size: var(--text-14);
    font-weight: 500;
    margin: 0 0 7px;
  }
  p {
    font-size: var(--text-12);
    color: var(--secondary);
    margin: 0 0 7px;
    overflow-wrap: anywhere;
    text-align: center;
  }
  .pin-hint {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: var(--text-11);
    color: var(--muted);
    text-align: center;
  }
  .actions {
    display: flex;
    gap: 8px;
    margin-top: 14px;
  }
  button {
    padding: 7px 12px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface);
    color: var(--text);
    font-size: var(--text-11);
  }
  button:hover {
    background: var(--hover);
  }
</style>
