<script lang="ts">
  import ModuleSettings from './ModuleSettings.svelte';
  import UpdateSettings from './UpdateSettings.svelte';
  import WindowEffects from './WindowEffects.svelte';
  import ShortcutSettings from './ShortcutSettings.svelte';
  import { shortcutLabel } from '../shortcuts';
  import { onMount, untrack } from 'svelte';
  import {
    ArrowLeft,
    ChevronRight,
    Command,
    Power,
    Settings2,
    Sparkles,
  } from '@lucide/svelte';
  import { preferences } from '../stores/preferences';
  import { quitLauncher, type LauncherStatus } from '../native';
  import { clearHistory } from '../stores/personalization';
  import CommandsSettings from './CommandsSettings.svelte';
  import LauncherSettings from './LauncherSettings.svelte';
  import KeyboardHint from './KeyboardHint.svelte';
  import ThemePicker from './ThemePicker.svelte';
  import FileSettings from './FileSettings.svelte';
  import { applications, loadApplications } from '../stores/applications';

  let {
    onback,
    onexecute,
    initialSection = 'settings',
    shortcut,
    shortcutError,
    version,
    onstatuschange = () => {},
  }: {
    onexecute: (
      command: import('../types/command').Command,
      mode?: import('../native').LaunchMode,
    ) => Promise<void>;
    onback: () => void;
    initialSection?: 'settings' | 'about' | 'appearance' | 'updates';
    shortcut: string | null;
    shortcutError: string | null;
    version: string;
    onstatuschange?: (status: LauncherStatus) => void;
  } = $props();

  type Tab =
    | 'General'
    | 'Commands'
    | 'Files & Run'
    | 'Appearance'
    | 'Shortcuts'
    | 'Modules'
    | 'Updates'
    | 'About';
  const tabId = (tab: Tab) => tab.toLowerCase().replace(/[^a-z0-9]+/g, '-');
  const tabs: Tab[] = [
    'General',
    'Commands',
    'Files & Run',
    'Appearance',
    'Shortcuts',
    'Modules',
    'Updates',
    'About',
  ];
  let active: Tab = $state(
    untrack(() =>
      initialSection === 'updates'
        ? 'Updates'
        : initialSection === 'about'
          ? 'About'
          : initialSection === 'appearance'
            ? 'Appearance'
            : 'General',
    ),
  );
  let backButton: HTMLButtonElement;
  let error = $state('');
  let themePicker = $state<{ cancelEditing: () => boolean }>();
  let editingTheme = $state(false);
  let launcherSettings = $state<{ cancelRecording: () => boolean }>();
  let historyCleared = $state(false);
  let shortcutSettings = $state<{ cancelRecording: () => boolean }>();

  onMount(() => backButton?.focus({ preventScroll: true }));

  function back() {
    if (
      shortcutSettings?.cancelRecording() ||
      launcherSettings?.cancelRecording() ||
      themePicker?.cancelEditing()
    )
      return;
    onback();
  }

  function editorEscape(event: KeyboardEvent) {
    if (
      event.key !== 'Escape' ||
      !(
        shortcutSettings?.cancelRecording() ||
        launcherSettings?.cancelRecording() ||
        themePicker?.cancelEditing()
      )
    )
      return;
    event.preventDefault();
    event.stopPropagation();
  }

  function selectTab(tab: Tab) {
    shortcutSettings?.cancelRecording();
    themePicker?.cancelEditing();
    launcherSettings?.cancelRecording();
    editingTheme = false;
    active = tab;
  }

  function toggle(key: 'animations' | 'showDescriptions') {
    preferences.update((value) => ({ ...value, [key]: !value[key] }));
  }

  function navigateTabs(event: KeyboardEvent, index: number) {
    if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
    event.preventDefault();
    const next =
      event.key === 'Home'
        ? 0
        : event.key === 'End'
          ? tabs.length - 1
          : (index + (event.key === 'ArrowRight' ? 1 : -1) + tabs.length) %
            tabs.length;
    selectTab(tabs[next]);
    const tab = (event.currentTarget as HTMLButtonElement).parentElement
      ?.children[next] as HTMLButtonElement | undefined;
    tab?.focus();
  }

  async function quit() {
    error = '';
    try {
      await quitLauncher();
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
    }
  }
</script>

<section
  class="settings-view"
  aria-label="Mycast preferences"
  onkeydowncapture={editorEscape}
>
  <header class="settings-header">
    <button
      class="back-button"
      bind:this={backButton}
      onclick={back}
      aria-label={editingTheme ? 'Cancel theme editing' : 'Back to commands'}
      title={editingTheme ? 'Cancel theme editing' : 'Back to commands'}
    >
      <ArrowLeft size={17} strokeWidth={1.7} />
    </button>
    <div class="heading">
      <span>Preferences</span><span class="breadcrumb"
        ><ChevronRight size={12} /> {active}</span
      >
    </div>
    <Settings2 size={17} strokeWidth={1.5} class="header-icon" />
  </header>

  <div class="tabs" role="tablist" aria-label="Preference sections">
    {#each tabs as tab, index}
      <button
        role="tab"
        id={`tab-${tabId(tab)}`}
        aria-controls={`panel-${tabId(tab)}`}
        aria-selected={active === tab}
        tabindex={active === tab ? 0 : -1}
        class:active={active === tab}
        onclick={() => selectTab(tab)}
        onkeydown={(event) => navigateTabs(event, index)}>{tab}</button
      >
    {/each}
  </div>

  <div
    class="settings-content"
    class:commands-panel={active === 'Commands'}
    role="tabpanel"
    id={`panel-${tabId(active)}`}
    aria-labelledby={`tab-${tabId(active)}`}
    tabindex="0"
  >
    {#if active === 'General'}
      <p class="section-eyebrow">MAKE IT YOURS</p>
      <h2>A little closer to your workflow.</h2>
      <p class="section-description">Your preferences stay on this device.</p>

      <LauncherSettings
        bind:this={launcherSettings}
        {shortcut}
        {shortcutError}
        {onstatuschange}
      />
      <div class="setting-row">
        <div class="setting-label">
          <span>Installed applications</span><small
            >Refresh after installing or removing a program.</small
          >
        </div>
        <button
          class="small-button"
          disabled={$applications.status === 'loading'}
          onclick={() => void loadApplications(true)}
          >Refresh applications</button
        >
      </div>
      <div class="setting-row last">
        <div class="setting-label">
          <span>Show descriptions</span><small
            >A little more context beneath each command.</small
          >
        </div>
        <button
          class="switch"
          class:enabled={$preferences.showDescriptions}
          role="switch"
          aria-checked={$preferences.showDescriptions}
          aria-label="Show descriptions"
          onclick={() => toggle('showDescriptions')}><span></span></button
        >
      </div>
      <div class="setting-row last">
        <div class="setting-label">
          <span>Recents and learned ranking</span><small
            >Clear successful-use history. Favorites and aliases stay saved.</small
          >
        </div>
        <button
          class="small-button"
          onclick={() => {
            clearHistory();
            historyCleared = true;
          }}>Clear recents and learned ranking</button
        >
      </div>
      {#if historyCleared}<p class="quiet-note" role="status">
          Recents and learned ranking cleared.
        </p>{/if}
      <div class="setting-row last">
        <label class="setting-label" for="recent-limit"
          ><span>Recent items</span><small
            >Show 1–20 recently opened programs, files, and folders first.</small
          ></label
        >
        <input
          class="small-button"
          id="recent-limit"
          type="number"
          min="1"
          max="20"
          step="1"
          value={$preferences.recentLimit}
          onchange={(event) =>
            preferences.update((value) => ({
              ...value,
              recentLimit: event.currentTarget.valueAsNumber,
            }))}
        />
      </div>
    {:else if active === 'Modules'}
      <ModuleSettings />
    {:else if active === 'Updates'}
      <UpdateSettings />
    {:else if active === 'Shortcuts'}
      <LauncherSettings
        bind:this={launcherSettings}
        {shortcut}
        {shortcutError}
        {onstatuschange}
      />
      <ShortcutSettings
        bind:this={shortcutSettings}
        launcherShortcut={shortcut}
      />
    {:else if active === 'Commands'}
      <CommandsSettings {onexecute} />
    {:else if active === 'Files & Run'}
      <FileSettings />
    {:else if active === 'Appearance'}
      {#if !editingTheme}<WindowEffects />{/if}
      <ThemePicker
        bind:this={themePicker}
        oneditingchange={(editing) => (editingTheme = editing)}
      />
      {#if !editingTheme}
        <div class="setting-row">
          <label class="setting-label" for="density"
            ><span>Density</span><small
              >Choose the space around each command.</small
            ></label
          >
          <select
            id="density"
            value={$preferences.density}
            onchange={(event) =>
              preferences.update((value) => ({
                ...value,
                density: event.currentTarget.value as 'comfortable' | 'compact',
              }))}
            ><option value="comfortable">Comfortable</option><option
              value="compact">Compact</option
            ></select
          >
        </div>
        <div class="setting-row">
          <label class="setting-label" for="text-size"
            ><span>Text size</span><small>Scale text throughout Mycast.</small
            ></label
          >
          <select
            id="text-size"
            value={$preferences.textSize}
            onchange={(event) =>
              preferences.update((value) => ({
                ...value,
                textSize: Number(event.currentTarget.value) as
                  90 | 100 | 110 | 125,
              }))}
            >{#each [90, 100, 110, 125] as size}<option value={size}
                >{size}%</option
              >{/each}</select
          >
        </div>
        <div class="setting-row last">
          <div class="setting-label">
            <span>Animations</span><small
              >Subtle transitions as you move around.</small
            >
          </div>
          <button
            class="switch"
            class:enabled={$preferences.animations}
            role="switch"
            aria-checked={$preferences.animations}
            aria-label="Animations"
            onclick={() => toggle('animations')}><span></span></button
          >
        </div>
        <p class="quiet-note">
          <Sparkles size={13} strokeWidth={1.5} /> System reduced motion preferences
          are always respected.
        </p>
      {/if}
    {:else}
      <div class="about-brand">
        <div class="brand-mark" aria-hidden="true">
          <Command size={25} strokeWidth={1.55} />
        </div>
        <div>
          <h2>Mycast <span class="version">v{version}</span></h2>
          <p>Your next action, a keystroke away.</p>
        </div>
      </div>
      <div class="about-details">
        <div><span>Made for</span><span>Windows 11</span></div>
        <div>
          <span>Under the surface</span><span>Built with Tauri and Rust</span>
        </div>
        <div>
          <span>Connection</span><span class="offline"
            ><span></span> Works offline</span
          >
        </div>
      </div>
      <div class="about-bottom">
        <p>Small by design. Ready when you are.</p>
        <button class="quit-button" onclick={quit}
          ><Power size={13} /> Quit Mycast</button
        >
      </div>
    {/if}
    {#if error}<p class="action-error" role="alert">{error}</p>{/if}
  </div>

  <footer class="settings-footer">
    <span class="footer-brand"
      ><Command size={13} strokeWidth={1.5} /> Mycast</span
    ><span class="back-hint"
      ><KeyboardHint
        keys={shortcutLabel($preferences.shortcuts.close).split('+')}
      />
      {editingTheme ? 'Cancel editing' : 'Back to commands'}</span
    >
  </footer>
</section>

<style>
  .small-button,
  select {
    max-width: 48%;
    padding: 6px 8px;
    background: var(--surface);
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: 5px;
    font-size: var(--text-10);
  }

  .settings-view {
    height: 100%;
    min-height: 0;
    display: flex;
    flex-direction: column;
    color: var(--text);
    animation: settings-enter 160ms ease-out;
  }
  @keyframes settings-enter {
    from {
      opacity: 0;
      transform: translateX(5px);
    }
    to {
      opacity: 1;
      transform: translateX(0);
    }
  }
  .settings-header {
    height: 66px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 22px;
    border-bottom: 1px solid var(--border);
  }
  .back-button {
    display: grid;
    place-items: center;
    width: 29px;
    height: 29px;
    padding: 0;
    border: 1px solid var(--border);
    border-radius: 7px;
    color: var(--secondary);
    background: transparent;
    cursor: pointer;
    transition:
      background 120ms,
      color 120ms;
  }
  .back-button:hover {
    background: var(--hover);
    color: var(--text);
  }
  .heading {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: var(--text-13);
    font-weight: 550;
  }
  .breadcrumb {
    display: inline-flex;
    align-items: center;
    gap: 9px;
    color: var(--muted);
    font-size: var(--text-12);
    font-weight: 400;
  }
  :global(.header-icon) {
    margin-left: auto;
    color: var(--muted);
  }
  .tabs {
    padding: 14px 26px 0;
    display: flex;
    gap: 17px;
    overflow-x: auto;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .tabs button {
    position: relative;
    background: transparent;
    border: 0;
    padding: 0 0 12px;
    white-space: nowrap;
    color: var(--muted);
    font: inherit;
    font-size: var(--text-12);
    cursor: pointer;
    transition: color 120ms;
  }
  .tabs button:hover {
    color: var(--text);
  }
  .tabs button.active {
    color: var(--text);
  }
  .tabs button.active::after {
    content: '';
    position: absolute;
    bottom: -1px;
    left: 0;
    right: 0;
    height: 2px;
    background: var(--accent);
    border-radius: 2px;
  }
  .settings-content {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 21px 28px 18px;
    outline: none;
  }
  .settings-content.commands-panel {
    display: flex;
    overflow: hidden;
    padding-bottom: 0;
  }
  .section-eyebrow {
    margin: 0 0 7px;
    color: var(--muted);
    font-size: var(--text-9);
    letter-spacing: 1.1px;
    font-weight: 600;
  }
  h2 {
    margin: 0;
    font-size: var(--text-17);
    font-weight: 550;
    letter-spacing: -0.35px;
    line-height: 1.4;
  }
  .section-description {
    margin: 5px 0 17px;
    color: var(--muted);
    font-size: var(--text-11);
  }
  .setting-row {
    min-height: 63px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 20px;
    border-bottom: 1px solid var(--border);
  }
  .setting-row.last {
    border-bottom: 0;
  }
  .setting-label {
    display: flex;
    flex-direction: column;
    gap: 5px;
    min-width: 0;
  }
  .setting-label > span {
    color: var(--text);
    font-size: var(--text-12);
  }
  .setting-label small {
    color: var(--muted);
    font-size: var(--text-10);
    line-height: 1.4;
  }
  .switch {
    display: inline-flex;
    align-items: center;
    flex-shrink: 0;
    height: 20px;
    width: 34px;
    padding: 3px;
    border-radius: 12px;
    background: var(--selected);
    border: 0;
    cursor: pointer;
    transition: background 130ms;
  }
  .switch span {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--secondary);
    box-shadow: 0 1px 2px #0003;
    transition:
      transform 130ms,
      background 130ms;
  }
  .switch.enabled {
    background: var(--accent);
  }
  .switch.enabled span {
    transform: translateX(14px);
    background: var(--bg);
  }
  .switch:disabled {
    cursor: not-allowed;
    opacity: 0.4;
  }
  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 4px;
  }
  .quiet-note {
    display: flex;
    align-items: center;
    gap: 7px;
    color: var(--muted);
    font-size: var(--text-10);
    margin-top: 23px;
  }
  .action-error {
    margin: 7px 0 3px;
    font-size: var(--text-10);
    color: var(--accent);
    line-height: 1.5;
  }
  .about-brand {
    display: flex;
    align-items: center;
    gap: 15px;
    margin: 12px 0 27px;
  }
  .brand-mark {
    width: 49px;
    height: 49px;
    background: color-mix(in srgb, var(--accent) 8%, transparent);
    border: 1px solid color-mix(in srgb, var(--accent) 15%, transparent);
    border-radius: 12px;
    display: grid;
    place-items: center;
    color: var(--accent);
  }
  .about-brand h2 {
    font-size: var(--text-21);
    font-weight: 600;
    letter-spacing: -0.5px;
  }
  .version {
    font-size: var(--text-10);
    color: var(--muted);
    font-weight: 400;
    letter-spacing: 0;
    margin-left: 5px;
    vertical-align: middle;
  }
  .about-brand p {
    font-size: var(--text-11);
    color: var(--muted);
    margin: 5px 0 0;
  }
  .about-details > div {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 38px;
    font-size: var(--text-11);
    border-bottom: 1px solid var(--border);
  }
  .about-details > div > span:first-child {
    color: var(--muted);
  }
  .about-details > div > span:last-child {
    color: var(--secondary);
  }
  .offline {
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .offline > span {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--accent);
  }
  .about-bottom {
    margin-top: 23px;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .about-bottom p {
    font-size: var(--text-10);
    color: var(--muted);
    margin: 0;
  }
  .quit-button {
    display: flex;
    align-items: center;
    gap: 6px;
    background: none;
    border: 1px solid var(--border);
    border-radius: 5px;
    color: var(--secondary);
    padding: 6px 9px;
    font: inherit;
    font-size: var(--text-10);
    cursor: pointer;
  }
  .quit-button:hover {
    background: var(--hover);
    color: var(--text);
  }
  .settings-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 40px;
    padding: 0 20px;
    border-top: 1px solid var(--border);
    flex-shrink: 0;
    background: color-mix(in srgb, var(--surface) 25%, var(--bg));
  }
  .footer-brand {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    font-size: var(--text-10);
    color: var(--muted);
    font-weight: 550;
  }
  .back-hint {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    font-size: var(--text-10);
    color: var(--muted);
  }
  @media (prefers-reduced-motion: reduce) {
    *,
    *::after {
      transition: none !important;
    }
  }
</style>
