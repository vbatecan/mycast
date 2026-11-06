<script lang="ts">
  import {
    aiProvider,
    providers,
    getAiConfig,
    saveAiConfig,
    type AiConfig,
    type Provider,
  } from '../stores/ai';
  let config = $state<AiConfig>();
  let apiKey = $state('');
  let removeKey = $state(false);
  let error = $state('');
  let message = $state('');
  let busy = $state(false);
  let sequence = 0;
  $effect(() => {
    const provider = $aiProvider;
    const request = ++sequence;
    config = undefined;
    apiKey = '';
    removeKey = false;
    error = '';
    message = '';
    void getAiConfig(provider)
      .then((value) => {
        if (request === sequence) config = value;
      })
      .catch((reason) => {
        if (request === sequence) error = String(reason);
      });
  });
  async function save() {
    if (!config) return;
    busy = true;
    error = '';
    message = '';
    try {
      config = await saveAiConfig(config, apiKey, removeKey);
      apiKey = '';
      removeKey = false;
      message = 'AI settings saved.';
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }
</script>

<h2>AI assistant</h2>
<p class="section-description">
  Type ?? and a prompt, then press Enter. Only submitted prompts go to the
  selected provider. Responses stay in this session; keys use Windows Credential
  Manager.
</p>
<div class="setting-row">
  <label for="ai-provider">Provider</label><select
    id="ai-provider"
    disabled={busy}
    value={$aiProvider}
    onchange={(e) => aiProvider.set(e.currentTarget.value as Provider)}
    >{#each Object.entries(providers) as [id, name]}<option value={id}
        >{name}</option
      >{/each}</select
  >
</div>
{#if config}
  <div class="setting-row">
    <label for="ai-model">Model ID</label><input
      id="ai-model"
      bind:value={config.model}
      placeholder="Enter the exact model ID"
    />
  </div>
  <div class="setting-row">
    <label for="ai-url">API base URL</label><input
      id="ai-url"
      bind:value={config.baseUrl}
      readonly={['openai', 'claude', 'gemini'].includes(config.provider)}
    />
  </div>
  <p class="quiet-note">
    Custom, Ollama and LM Studio use /chat/completions beneath this base URL.
    Include /v1 where your server requires it. Local defaults: Ollama :11434/v1,
    LM Studio :1234/v1.
  </p>
  <div class="setting-row">
    <label for="ai-key"
      >API key {config.hasKey ? '(saved)' : '(not saved)'}</label
    ><input
      id="ai-key"
      type="password"
      bind:value={apiKey}
      autocomplete="new-password"
      placeholder="Leave blank to keep saved key"
      disabled={removeKey}
    />
  </div>
  <label
    ><input type="checkbox" bind:checked={removeKey} /> Delete saved key for this
    endpoint</label
  >
  <p class="quiet-note">
    Keys are optional for local servers. Endpoint changes use a separate key.
    Provider usage may incur charges.
  </p>
  <button class="small-button" disabled={busy} onclick={save}
    >{busy ? 'Saving…' : 'Save AI settings'}</button
  >
{/if}
{#if error}<p role="alert">{error}</p>{/if}
{#if message}<p role="status">{message}</p>{/if}

<style>
  h2 {
    font-size: var(--text-18);
    margin: 0 0 10px;
  }
  .section-description,
  .quiet-note {
    font-size: var(--text-12);
    color: var(--secondary);
    line-height: 1.6;
  }
  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 15px;
    padding: 12px 0;
    border-bottom: 1px solid var(--border);
    font-size: var(--text-12);
  }
  input:not([type='checkbox']),
  select {
    background: var(--surface);
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 7px 9px;
    width: 60%;
    min-width: 0;
  }
  input[readonly] {
    color: var(--secondary);
  }
  label {
    font-size: var(--text-12);
  }
  .small-button {
    background: var(--surface);
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 7px 12px;
  }
  p[role='alert'] {
    font-size: var(--text-12);
    color: var(--danger);
  }
  p[role='status'] {
    font-size: var(--text-12);
    color: var(--success);
  }
</style>
