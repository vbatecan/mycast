<script lang="ts">
  import { preferences, updatePreference } from '../stores/preferences';
  import { validSearchTemplate, type SearchEngine } from '../commands/web';
  let draft = $state($preferences.customSearchUrl);
  let error = $state('');
  function save() {
    if (!validSearchTemplate(draft)) {
      error =
        'Enter an HTTP or HTTPS URL containing exactly one {query} placeholder, without credentials.';
      return;
    }
    updatePreference('customSearchUrl', draft);
    error = '';
  }
</script>

<h2>Web search</h2>
<p class="section-description">
  Type ? followed by search terms. Enter opens your default browser. Nothing is
  sent while typing.
</p>
<div class="setting-row">
  <label for="search-engine">Search engine</label>
  <select
    id="search-engine"
    value={$preferences.searchEngine}
    onchange={(event) =>
      updatePreference(
        'searchEngine',
        event.currentTarget.value as SearchEngine,
      )}
  >
    <option value="google">Google</option><option value="bing">Bing</option
    ><option value="duckduckgo">DuckDuckGo</option><option value="custom"
      >Custom</option
    >
  </select>
</div>
{#if $preferences.searchEngine === 'custom'}
  <label for="search-template">Search URL template</label>
  <input
    id="search-template"
    style="width:100%"
    bind:value={draft}
    placeholder={'https://example.com/search?q={query}'}
  />
  <button class="small-button" onclick={save}>Save URL</button>
  {#if error}<p role="alert">{error}</p>{/if}
{/if}
