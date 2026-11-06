<script lang="ts">
  import { onMount, tick, untrack } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import {
    Command as CommandIcon,
    ArrowDown,
    ArrowUp,
    Check,
    CircleAlert,
    SlidersHorizontal,
    Sparkles,
  } from '@lucide/svelte';
  import SearchInput from './SearchInput.svelte';
  import CommandList from './CommandList.svelte';
  import EmptyState from './EmptyState.svelte';
  import SettingsView from './SettingsView.svelte';
  import KeyboardHint from './KeyboardHint.svelte';
  import CatalogueBar from './CatalogueBar.svelte';
  import {
    eventShortcut,
    shortcutLabel,
    type ShortcutAction,
  } from '../shortcuts';
  import { searchCommands, homeSection } from '../commands/search';
  import {
    personalization,
    keyForCommand,
    toggleFavorite,
    recordSuccessfulUse,
  } from '../stores/personalization';
  import { calculateExpression } from '../commands/calculator';
  import type { Calculation } from '../commands/calculator';
  import { executeCommand } from '../commands/execute';
  import {
    getLauncherStatus,
    hideLauncher,
    isNative,
    searchFiles,
  } from '../native';
  import {
    fileStatus,
    savedFileCommands,
    fileCommand,
    observeFileIndex,
  } from '../stores/files';
  import { preferences } from '../stores/preferences';
  import {
    applications,
    commandCatalogue,
    loadApplications,
  } from '../stores/applications';
  import type { CommandScope } from '../stores/applications';
  import type { Command } from '../types/command';

  let query = $state('');
  let selectedKey = $state('');
  let searchTimestamp = $state(Date.now());
  let view = $state<'palette' | 'settings' | 'about' | 'appearance'>('palette');
  let scope = $state<CommandScope>('all');
  let input = $state<HTMLInputElement>();
  let busy = $state(false);
  let message = $state('');
  let isError = $state(false);
  let shown = $state(true);
  let shortcut = $state<string | null>(null);
  let shortcutError = $state<string | null>(null);
  let version = $state('0.1.1');
  let platform = $state('');
  let filesReady = $state(false);
  let fileMatches = $state<Command[]>([]);
  let fileSearchError = $state('');
  let fileSearching = $state(false);
  let requestSequence = Date.now() * 1000;
  let runMode = $derived(query.trimStart().startsWith('>'));
  let runText = $derived(runMode ? query.trimStart().slice(1).trimStart() : '');
  let messageTimer: ReturnType<typeof setTimeout> | undefined;
  let hideTimer: ReturnType<typeof setTimeout> | undefined;
  let scopedCatalogue = $derived(
    [
      ...new Map(
        [...$commandCatalogue, ...$savedFileCommands, ...fileMatches].map(
          (command) => [command.personalizationKey ?? command.id, command],
        ),
      ).values(),
    ].filter(
      (command) =>
        !['applications', 'commands', 'files'].includes(scope) ||
        (scope === 'files'
          ? command.category === 'Files'
          : scope === 'applications'
            ? command.category === 'Applications'
            : command.category !== 'Applications' &&
              command.category !== 'Files'),
    ),
  );
  let calculation = $derived.by((): Calculation =>
    runMode || ['applications', 'favorites', 'recents', 'files'].includes(scope)
      ? { kind: 'none' }
      : calculateExpression(query),
  );
  let searchOptions = $derived({
    ...$personalization,
    recentLimit: $preferences.recentLimit,
    now: searchTimestamp,
    collection:
      scope === 'favorites' || scope === 'recents' ? scope : ('all' as const),
  });
  let results = $derived.by((): Command[] => {
    if (runMode)
      return [
        {
          id: 'run-command',
          label: runText ? 'Run ' + runText : 'Open PowerShell',
          description: `${$fileStatus.shell || 'PowerShell'} · ${$fileStatus.preferences?.terminalDirectory || 'Home folder'}`,
          category: 'Utilities',
          icon: 'terminal',
          keywords: [],
          action: { type: 'run', command: runText },
        },
      ];
    if (calculation.kind === 'success')
      return [
        {
          id: 'calculation',
          label: calculation.formatted,
          description: query + '  ·  Copy result to clipboard',
          category: 'Calculator',
          icon: 'result',
          keywords: [],
          action: { type: 'copy', text: calculation.formatted },
        },
      ];
    return searchCommands(query, scopedCatalogue, 40, searchOptions);
  });
  $effect(() => {
    const text = query;
    const selectedScope = scope;
    const state = $personalization;
    $fileStatus.revision;
    const active =
      filesReady &&
      shown &&
      view === 'palette' &&
      !runMode &&
      ['all', 'files', 'favorites', 'recents'].includes(selectedScope);
    const requestId = ++requestSequence;
    untrack(() => {
      if (
        !active ||
        fileMatches.some((command) => command.searchQuery !== text)
      )
        fileMatches = [];
    });
    fileSearchError = '';
    fileSearching = active;
    if (!active) return;
    let cancelled = false;
    const timer = setTimeout(
      () => {
        void searchFiles({
          query: text,
          requestId,
          favorites: state.favorites,
          recents: state.recents,
          aliases: state.aliases,
          usage: state.usage,
          now: Date.now(),
          collection:
            selectedScope === 'favorites' || selectedScope === 'recents'
              ? selectedScope
              : 'all',
          browse: selectedScope === 'files',
        })
          .then((response) => {
            if (!cancelled && requestId === requestSequence)
              fileMatches = response.items.map((item) =>
                fileCommand(item, text),
              );
          })
          .catch((error) => {
            if (!cancelled) fileSearchError = String(error);
          })
          .finally(() => {
            if (!cancelled) fileSearching = false;
          });
      },
      text ? 45 : 0,
    );
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  });
  const selectionKey = (command: Command) =>
    keyForCommand(command) ?? command.id;
  let selectedIndex = $derived(
    Math.max(
      0,
      results.findIndex((command) => selectionKey(command) === selectedKey),
    ),
  );
  let selectedCommand = $derived(results[selectedIndex]);
  let selectedPersonalizationKey = $derived(
    selectedCommand && keyForCommand(selectedCommand),
  );

  $effect(() => {
    query;
    scope;
    untrack(() => {
      selectedKey = '';
      message = '';
    });
  });
  $effect(() => {
    if (!results.some((command) => selectionKey(command) === selectedKey))
      selectedKey = results[0] ? selectionKey(results[0]) : '';
  });

  function notify(text: string, error = false) {
    clearTimeout(messageTimer);
    message = text;
    isError = error;
    messageTimer = setTimeout(
      () => {
        message = '';
      },
      error ? 6000 : 3000,
    );
  }
  async function focusSearch() {
    await tick();
    input?.focus();
  }
  async function back() {
    view = 'palette';
    await focusSearch();
  }
  function selectScope(next: CommandScope) {
    scope = next;
    selectedKey = '';
    void focusSearch();
  }
  async function hide() {
    if (!isNative()) {
      notify('Window hiding is available in the desktop app.', true);
      return;
    }
    if (!shown) return;
    shown = false;
    const reducedMotion = matchMedia(
      '(prefers-reduced-motion: reduce)',
    ).matches;
    hideTimer = setTimeout(
      async () => {
        try {
          await hideLauncher();
        } catch (error) {
          shown = true;
          notify(String(error), true);
        }
      },
      $preferences.animations && !reducedMotion ? 110 : 0,
    );
  }
  async function execute(
    command: Command,
    mode: import('../native').LaunchMode = 'normal',
  ) {
    if (busy) return;
    busy = true;
    try {
      const result = await executeCommand(command, mode);
      if (result.type === 'cancelled') {
        notify(result.message);
        return;
      }
      recordSuccessfulUse(command);
      searchTimestamp = Date.now();
      if (result.type === 'view') view = result.view;
      else if (result.type === 'calculator-mode') {
        query = '';
        await focusSearch();
        notify('Type a calculation, for example (50 + 20) / 2');
      } else {
        notify(result.message);
        if (
          command.action.type === 'application' ||
          command.action.type === 'file' ||
          command.action.type === 'run' ||
          (command.action.type === 'native' &&
            !command.action.name.startsWith('copy-'))
        )
          await hide();
      }
    } catch (error) {
      notify(error instanceof Error ? error.message : String(error), true);
    } finally {
      busy = false;
    }
  }
  function handleKeys(event: KeyboardEvent) {
    if (event.isComposing || event.defaultPrevented) return;
    const binding = eventShortcut(event);
    const action = (
      Object.keys($preferences.shortcuts) as ShortcutAction[]
    ).find((id) => $preferences.shortcuts[id] === binding);
    if (
      view === 'palette' &&
      action &&
      [
        'all',
        'applications',
        'commands',
        'files',
        'favorites',
        'recents',
      ].includes(action)
    ) {
      event.preventDefault();
      selectScope(action as CommandScope);
      return;
    }
    if (view === 'palette' && action === 'openAdmin') {
      event.preventDefault();
      if (selectedCommand && !event.repeat)
        void execute(selectedCommand, 'administrator');
      return;
    }
    if (view === 'palette' && action === 'pin') {
      event.preventDefault();
      if (selectedPersonalizationKey) toggleFavorite(selectedCommand);
      return;
    }
    if (
      view === 'palette' &&
      (action === 'nextFilter' || action === 'previousFilter')
    ) {
      event.preventDefault();
      const scopes: CommandScope[] = [
        'all',
        'applications',
        'commands',
        'files',
        'favorites',
        'recents',
      ];
      selectScope(
        scopes[
          (scopes.indexOf(scope) +
            (action === 'previousFilter' ? scopes.length - 1 : 1)) %
            scopes.length
        ],
      );
      return;
    }
    if (action === 'close' || (view !== 'palette' && event.key === 'Escape')) {
      event.preventDefault();
      if (view !== 'palette') void back();
      else void hide();
    } else if (
      view === 'palette' &&
      (action === 'nextItem' || action === 'previousItem')
    ) {
      event.preventDefault();
      if (results.length)
        selectedKey = selectionKey(
          results[
            (selectedIndex +
              (action === 'nextItem' ? 1 : -1) +
              results.length) %
              results.length
          ],
        );
      input?.focus();
    } else if (view === 'palette' && action === 'open' && !event.repeat) {
      event.preventDefault();
      if (results[selectedIndex]) void execute(results[selectedIndex]);
    }
  }
  onMount(() => {
    let disposed = false;
    const unlisteners: (() => void)[] = [];
    const rememberListener = (stop: () => void) => {
      if (disposed) stop();
      else unlisteners.push(stop);
    };
    let filesStarted = false;
    const startFiles = () => {
      if (filesStarted) return;
      filesStarted = true;
      void observeFileIndex()
        .then((stop) => {
          rememberListener(stop);
          if (!disposed) filesReady = true;
        })
        .catch((error) => notify(String(error), true));
    };
    if (isNative()) {
      void getLauncherStatus()
        .then((status) => {
          shortcut = status.shortcut;
          shortcutError = status.shortcutError;
          version = status.version;
          platform = status.platform;
          shown = status.visible;
          if (status.visible) {
            startFiles();
            void focusSearch();
            void loadApplications();
          }
          if (!status.shortcut && status.shortcutError)
            notify(status.shortcutError, true);
        })
        .catch((error) => notify(String(error), true));
      void listen('launcher-shown', () => {
        startFiles();
        clearTimeout(hideTimer);
        shown = false;
        searchTimestamp = Date.now();
        void loadApplications();
        view = 'palette';
        message = '';
        requestAnimationFrame(() => {
          shown = true;
          void focusSearch();
        });
      })
        .then(rememberListener)
        .catch((error) => notify(String(error), true));
      void listen('launcher-hidden', () => {
        shown = false;
      })
        .then(rememberListener)
        .catch((error) => notify(String(error), true));
    } else {
      void focusSearch();
      void loadApplications();
    }
    return () => {
      disposed = true;
      unlisteners.forEach((stop) => stop());
      clearTimeout(messageTimer);
      clearTimeout(hideTimer);
    };
  });
</script>

<svelte:window
  onkeydown={(event) => {
    if (
      (event.key === 'Escape' ||
        (event.ctrlKey && ['Tab', 'd', 'D'].includes(event.key))) &&
      event.target !== input
    )
      handleKeys(event);
  }}
/>

<section
  class="palette"
  class:closing={!shown}
  aria-label="Mycast command launcher"
  aria-busy={busy}
>
  {#if view !== 'palette'}
    <SettingsView
      onback={back}
      onexecute={execute}
      initialSection={view}
      {shortcut}
      {shortcutError}
      {version}
      onstatuschange={(status) => {
        shortcut = status.shortcut;
        shortcutError = status.shortcutError;
      }}
    />
  {:else}
    <SearchInput
      bind:value={query}
      bind:input
      activeId={results[selectedIndex]
        ? 'command-' + results[selectedIndex].id
        : undefined}
      onkeydown={handleKeys}
    />
    <CatalogueBar {scope} onselect={selectScope} />
    {#if runMode}<div class="file-search-status">
        Run in a visible PowerShell terminal · {shortcutLabel(
          $preferences.shortcuts.open,
        )} to execute
      </div>
    {:else if fileSearchError || (['all', 'files', 'favorites', 'recents'].includes(scope) && isNative() && (fileSearching || ['indexing', 'reconciling'].includes($fileStatus.state)))}
      <div
        class="file-search-status"
        role="status"
        title={$fileStatus.warnings.join('\n')}
      >
        {fileSearchError ||
          ($fileStatus.state === 'indexing' ||
          $fileStatus.state === 'reconciling'
            ? 'Indexing file names…'
            : fileSearching
              ? 'Searching files…'
              : '')}
        {#if $fileStatus.warnings.length}<span>
            · {$fileStatus.warnings.length} indexing notice(s)</span
          >{/if}
      </div>
    {/if}
    {#if $applications.error}
      <div class="discovery-error" role="status" title={$applications.error}>
        <CircleAlert size={12} /><span
          >Applications could not refresh. {$applications.error}</span
        >
      </div>
    {/if}
    {#if calculation.kind === 'error'}
      <div class="calculation-error" role="status">
        <CircleAlert size={16} />
        <div>
          <strong>Check your calculation</strong>
          <p>{calculation.message}</p>
        </div>
      </div>
      <div class="calculator-tip"><Sparkles size={14} /> Try (50 + 20) / 2</div>
    {:else if results.length}
      <CommandList
        commands={results}
        {selectedIndex}
        showDescriptions={$preferences.showDescriptions}
        {query}
        sectionFor={!query.trim()
          ? (command) =>
              scope === 'favorites'
                ? 'Favorites'
                : scope === 'files'
                  ? 'Files'
                  : scope === 'recents'
                    ? 'Recents'
                    : homeSection(command, scopedCatalogue, searchOptions)
          : undefined}
        onselect={(index) => (selectedKey = selectionKey(results[index]))}
        onexecute={execute}
      />
    {:else if scope === 'files' && isNative() && (fileSearching || $fileStatus.state !== 'ready')}
      <div class="files-loading" role="status">
        {$fileStatus.state === 'indexing' || $fileStatus.state === 'reconciling'
          ? 'Indexing your folders…'
          : 'Searching files…'}
      </div>
    {:else}<EmptyState {query} />{/if}
    <footer>
      <div class="footer-brand">
        <span class="brand-icon"><CommandIcon size={13} strokeWidth={2} /></span
        ><span>mycast</span>
      </div>
      <div
        class="footer-status"
        role="status"
        aria-live="polite"
        class:error={isError && !!message}
      >
        {#if message}{#if isError}<CircleAlert size={12} />{:else}<Check
              size={12}
            />{/if}<span>{message}</span>
        {:else}<div class="hint">
            <span class="arrow-pair"
              >{shortcutLabel($preferences.shortcuts.previousItem)}
              {shortcutLabel($preferences.shortcuts.nextItem)}</span
            > Navigate
          </div>
          <div class="hint">
            <KeyboardHint
              keys={shortcutLabel($preferences.shortcuts.open).split('+')}
            />
            {runMode ? 'Run' : calculation.kind === 'success' ? 'Copy' : 'Open'}
          </div>
          {#if platform === 'windows' && (selectedCommand?.canRunAsAdmin || (selectedCommand?.action.type === 'native' && selectedCommand.action.name === 'notepad'))}
            <div class="hint">
              <KeyboardHint
                keys={shortcutLabel($preferences.shortcuts.openAdmin).split(
                  '+',
                )}
              /> Admin
            </div>
          {/if}
          <div class="hint">
            <KeyboardHint
              keys={shortcutLabel($preferences.shortcuts.close).split('+')}
            /> Close
          </div>{/if}
      </div>
      <button
        class="preferences-button"
        title="Open Mycast settings"
        aria-label="Open Mycast settings"
        onclick={() => (view = 'settings')}
        ><SlidersHorizontal size={14} /></button
      >
    </footer>
  {/if}
</section>

<style>
  .palette {
    position: relative;
    height: 100%;
    width: 100%;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    background: var(--window-glow), var(--window-tint);
    backdrop-filter: var(--window-filter, none);
    box-shadow: var(--window-edge);
    border: 1px solid var(--border);
    border-radius: 12px;
    animation: enter 150ms ease-out;
    transition:
      opacity 110ms,
      transform 110ms;
  }
  .file-search-status {
    padding: 4px 23px;
    font-size: var(--text-10);
    color: var(--muted);
    flex-shrink: 0;
  }
  .files-loading {
    flex: 1;
    display: grid;
    place-items: center;
    color: var(--secondary);
    font-size: var(--text-13);
  }
  .palette.closing {
    opacity: 0;
    transform: translateY(-4px) scale(0.995);
  }
  .discovery-error {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 6px 23px;
    font-size: var(--text-10);
    color: var(--danger);
    flex-shrink: 0;
  }
  .discovery-error span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  footer {
    min-height: 43px;
    height: 43px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 15px;
    border-top: 1px solid var(--border);
    padding: 0 15px 0 20px;
    background: color-mix(in srgb, var(--bg) 60%, var(--surface));
  }
  .footer-brand {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: var(--text-11);
    font-weight: 600;
    letter-spacing: 0.1px;
    color: var(--secondary);
  }
  .brand-icon {
    width: 21px;
    height: 21px;
    display: grid;
    place-items: center;
    color: var(--accent);
    border-radius: 6px;
    background: var(--accent-soft);
    border: 1px solid color-mix(in srgb, var(--accent) 18%, transparent);
  }
  .footer-status {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 12px;
    color: var(--secondary);
    font-size: var(--text-10);
  }
  .footer-status > span {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .footer-status.error {
    color: var(--danger);
    gap: 6px;
  }
  .hint {
    display: flex;
    gap: 5px;
    align-items: center;
    white-space: nowrap;
  }
  .arrow-pair {
    display: flex;
    gap: 2px;
    padding: 3px 3px;
    border: 1px solid var(--border);
    border-radius: 4px;
    color: var(--secondary);
    background: var(--surface);
  }
  .preferences-button {
    height: 24px;
    width: 26px;
    border: 0;
    background: transparent;
    border-left: 1px solid var(--border);
    padding-left: 12px;
    box-sizing: content-box;
    display: grid;
    place-items: center;
    color: var(--muted);
    transition: color 120ms;
  }
  .preferences-button:hover {
    color: var(--text);
  }
  .calculation-error {
    padding: 22px 26px;
    display: flex;
    align-items: flex-start;
    gap: 12px;
    color: var(--danger);
  }
  .calculation-error strong {
    font-weight: 500;
    font-size: var(--text-13);
  }
  .calculation-error p {
    font-size: var(--text-12);
    color: var(--secondary);
    margin: 6px 0;
  }
  .calculator-tip {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 8px;
    font-size: var(--text-12);
    color: var(--muted);
  }
  @keyframes enter {
    from {
      opacity: 0;
      transform: translateY(-5px) scale(0.995);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }
</style>
