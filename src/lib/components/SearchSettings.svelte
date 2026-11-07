<script lang="ts">
  import { preferences, updatePreference } from '../stores/preferences';
  import { validSearchTemplate, type SearchEngine } from '../commands/web';
  import { Globe, ArrowUpRight, Check } from '@lucide/svelte';
  import ChoicePicker from './ChoicePicker.svelte';
  import ConnectionField from './ConnectionField.svelte';
  let draft = $state($preferences.customSearchUrl);
  let error = $state('');
  let message = $state('');
  const dirty = $derived(draft !== $preferences.customSearchUrl);
  $effect(() => {
    if (dirty) message = '';
  });
  function save() {
    if (!validSearchTemplate(draft)) {
      error = 'Use a valid search URL with one {query} placeholder.';
      return;
    }
    updatePreference('customSearchUrl', draft);
    error = '';
    message = 'Saved';
  }
</script>

<form
  class="search-settings"
  onsubmit={(event) => {
    event.preventDefault();
    save();
  }}
>
  <div class="settings-heading">
    <span class="heading-mark"><Globe size={18} strokeWidth={1.6} /></span>
    <h2>Web search</h2>
  </div>
  <div class="settings-body">
    <div class="search-card">
      <ChoicePicker
        id="search-engine"
        label="Search engine"
        value={$preferences.searchEngine}
        choices={[
          { value: 'google', label: 'Google', mark: 'G' },
          { value: 'bing', label: 'Bing', mark: 'B' },
          { value: 'duckduckgo', label: 'DuckDuckGo', mark: 'D' },
          {
            value: 'custom',
            label: 'Custom',
            caption: 'Your search URL',
            mark: '</>',
          },
        ]}
        onchange={(value) => {
          updatePreference('searchEngine', value as SearchEngine);
          error = '';
          message = 'Saved';
        }}
      />
      {#if $preferences.searchEngine === 'custom'}<ConnectionField
          id="search-template"
          label="Search URL"
          kind="search"
          bind:value={draft}
          placeholder={'https://example.com/search?q={query}'}
          hint={'Use {query} where the search terms belong.'}
          oninput={() => (error = '')}
        />{/if}
    </div>
    {#if error}<p class="form-error" role="alert">{error}</p>{/if}
  </div>
  <div class="form-actions">
    <span class="save-status" role="status"
      >{#if message}<Check size={12} /> {message}{/if}</span
    >
    {#if $preferences.searchEngine === 'custom'}<button
        class="save-button"
        type="submit"
        disabled={!dirty}>Save changes<ArrowUpRight size={14} /></button
      >{/if}
  </div>
</form>

<style>
  .search-settings {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    color: var(--text);
    width: 100%;
  }
  .settings-heading {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-shrink: 0;
    padding-bottom: 16px;
  }
  .heading-mark {
    display: grid;
    place-items: center;
    width: 33px;
    height: 33px;
    border-radius: 10px;
    background: var(--accent-soft);
    color: var(--accent);
    border: 1px solid color-mix(in srgb, var(--accent) 15%, var(--border));
  }
  h2 {
    margin: 0;
    font-size: var(--text-16);
    font-weight: 550;
    letter-spacing: -0.3px;
  }
  .settings-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding-right: 2px;
  }
  .search-card {
    display: flex;
    flex-direction: column;
    gap: 17px;
    padding: 14px;
    border: 1px solid var(--border);
    border-radius: 13px;
    background: color-mix(in srgb, var(--surface) 24%, transparent);
  }
  .form-actions {
    flex-shrink: 0;
    display: flex;
    justify-content: space-between;
    align-items: center;
    min-height: 58px;
    padding: 11px 0;
  }
  .save-status {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    color: var(--secondary);
    font-size: var(--text-10);
  }
  .save-button {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    padding: 9px 13px;
    border-radius: 8px;
    background: var(--accent);
    color: var(--bg);
    border: 1px solid transparent;
    font-size: var(--text-11);
    font-weight: 500;
  }
  .save-button:disabled {
    color: var(--muted);
    background: var(--surface);
    border-color: var(--border);
  }
  .form-error {
    color: var(--danger);
    font-size: var(--text-11);
    line-height: 1.6;
    margin-top: 12px;
  }
</style>
