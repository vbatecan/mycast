<script lang="ts">
  import { onDestroy } from 'svelte';
  import {
    Sparkles,
    ShieldCheck,
    ChevronDown,
    ArrowUpRight,
    Check,
    Plug,
    ChevronRight,
  } from '@lucide/svelte';
  import ChoicePicker from './ChoicePicker.svelte';
  import ConnectionField from './ConnectionField.svelte';
  let { onmcp = () => {} }: { onmcp?: () => void } = $props();
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
  let reload = $state(0);
  let saved = $state('');
  let savedEndpoint = $state('');
  let sequence = 0;
  const endpoint = (url: string) => url.trim().replace(/\/+$/, '');
  const snapshot = (value: AiConfig) =>
    JSON.stringify([value.provider, value.model, value.baseUrl]);
  const local = $derived(
    config?.provider === 'ollama' || config?.provider === 'lmstudio',
  );
  const editableEndpoint = $derived(
    config && ['custom', 'ollama', 'lmstudio'].includes(config.provider),
  );
  const hasSavedKey = $derived(
    !!config?.hasKey && endpoint(config.baseUrl) === savedEndpoint,
  );
  const dirty = $derived(
    !!config && (snapshot(config) !== saved || !!apiKey || removeKey),
  );
  $effect(() => {
    if (dirty) message = '';
  });
  $effect(() => {
    const provider = $aiProvider;
    reload;
    const request = ++sequence;
    config = undefined;
    apiKey = '';
    removeKey = false;
    error = '';
    message = '';
    void getAiConfig(provider)
      .then((value) => {
        if (request === sequence) {
          config = value;
          saved = snapshot(value);
          savedEndpoint = endpoint(value.baseUrl);
        }
      })
      .catch((reason) => {
        if (request === sequence) error = String(reason);
      });
  });
  onDestroy(() => {
    sequence += 1;
  });
  async function save() {
    if (!config || busy) return;
    busy = true;
    error = '';
    message = '';
    try {
      config = await saveAiConfig(config, apiKey, removeKey);
      saved = snapshot(config);
      savedEndpoint = endpoint(config.baseUrl);
      apiKey = '';
      removeKey = false;
      message = 'Saved';
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }
</script>

<form
  class="ai-connection"
  onsubmit={(event) => {
    event.preventDefault();
    void save();
  }}
>
  <div class="connection-heading">
    <span class="heading-mark"><Sparkles size={18} strokeWidth={1.6} /></span>
    <h2>AI connection</h2>
    <span class="connection-state" class:pending={dirty}
      ><span></span>{dirty
        ? 'Unsaved changes'
        : config?.model
          ? 'Saved connection'
          : 'Not configured'}</span
    >
  </div>
  <fieldset class="connection-body" disabled={busy}>
    <div class="connection-card">
      <div class="connection-grid">
        <ChoicePicker
          id="ai-provider"
          label="Provider"
          value={$aiProvider}
          disabled={busy}
          choices={Object.entries(providers).map(([value, label]) => ({
            value,
            label: value === 'custom' ? 'Custom endpoint' : label,
            caption: ['ollama', 'lmstudio'].includes(value)
              ? 'Local'
              : value === 'custom'
                ? 'OpenAI-compatible'
                : 'Cloud',
            mark: (
              {
                openai: 'O',
                claude: 'C',
                gemini: 'G',
                custom: '</>',
                ollama: 'Ol',
                lmstudio: 'LM',
              } as Record<string, string>
            )[value],
          }))}
          onchange={(value) => aiProvider.set(value as Provider)}
        />
        {#if config}<ConnectionField
            id="ai-model"
            label="Model"
            kind="model"
            bind:value={config.model}
            placeholder="Model ID"
            required
          />{/if}
      </div>
      {#if config}
        {#snippet keyField()}
          <div class="key-control">
            <ConnectionField
              id="ai-key"
              label="API key"
              kind="key"
              bind:value={apiKey}
              saved={hasSavedKey && !removeKey}
              disabled={removeKey}
              placeholder={removeKey
                ? 'Key will be removed'
                : hasSavedKey
                  ? 'Leave blank to keep saved key'
                  : local
                    ? 'Optional'
                    : 'Enter API key'}
            />
            {#if hasSavedKey}<button
                class="remove-key"
                type="button"
                aria-label={removeKey ? 'Undo key removal' : 'Remove saved key'}
                aria-pressed={removeKey}
                onclick={() => (removeKey = !removeKey)}
                >{removeKey ? 'Undo' : 'Remove'}</button
              >{/if}
          </div>
        {/snippet}
        <div class="connection-grid" class:single={!editableEndpoint || local}>
          {#if editableEndpoint}<ConnectionField
              id="ai-url"
              label="Base URL"
              kind="endpoint"
              bind:value={config.baseUrl}
              required
              oninput={() => (removeKey = false)}
            />{/if}
          {#if !local}{@render keyField()}{/if}
        </div>
        {#if local}<details class="optional-auth">
            <summary
              >Authentication <span>Optional</span><ChevronDown
                size={12}
              /></summary
            >
            <div class="auth-content">{@render keyField()}</div>
          </details>{/if}
      {:else if !error}<p class="loading" role="status">
          Loading connection…
        </p>{/if}
    </div>
    <button class="mcp-link" type="button" onclick={onmcp}
      ><Plug size={15} /><span>MCP servers</span><ChevronRight
        size={13}
      /></button
    >
    {#if config}<details class="connection-privacy">
        <summary
          ><ShieldCheck size={14} /><span>Connection details</span><ChevronDown
            size={12}
          /></summary
        >
        {#if !editableEndpoint}<p>{config.baseUrl}</p>{/if}
        <p>
          Keys stay in Windows Credential Manager. Submitted prompts, recent
          context, and connected tool definitions and results go to your
          provider. History stays in memory until you start a new conversation,
          switch providers, or quit Mycast.
        </p>
        {#if !local}<p>Your provider may charge for requests.</p>{/if}
      </details>{/if}
    {#if error}<div class="connection-error" role="alert">{error}</div>
      {#if !config}<button
          class="retry"
          type="button"
          onclick={() => (reload += 1)}>Retry</button
        >{/if}{/if}
  </fieldset>
  {#if config}<div class="connection-actions">
      <span class="save-status" role="status"
        >{#if message}<Check size={12} /> {message}{/if}</span
      >
      <button
        class="save-connection"
        type="submit"
        disabled={busy || !dirty || !config.model.trim()}
        >{busy ? 'Saving…' : 'Save changes'}<ArrowUpRight size={14} /></button
      >
    </div>{/if}
</form>

<style>
  .ai-connection {
    flex: 1;
    min-height: 0;
    width: 100%;
    display: flex;
    flex-direction: column;
    color: var(--text);
  }
  .connection-heading {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 0 16px;
    flex-shrink: 0;
  }
  .heading-mark {
    width: 33px;
    height: 33px;
    display: grid;
    place-items: center;
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
  .connection-state {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-left: auto;
    font-size: var(--text-10);
    color: var(--muted);
  }
  .connection-state > span {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--secondary);
  }
  .connection-state.pending > span {
    background: var(--accent);
  }
  fieldset {
    margin: 0;
    padding: 0;
    border: 0;
    min-width: 0;
  }
  .connection-body {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding-right: 2px;
  }
  .connection-card {
    padding: 14px;
    border: 1px solid var(--border);
    border-radius: 13px;
    background: color-mix(in srgb, var(--surface) 24%, transparent);
    display: flex;
    flex-direction: column;
    gap: 15px;
  }
  .connection-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px;
    align-items: start;
  }
  .connection-grid.single {
    grid-template-columns: 1fr;
  }
  .key-control {
    min-width: 0;
    position: relative;
  }
  button {
    cursor: pointer;
  }
  .remove-key {
    position: absolute;
    top: -2px;
    right: 0;
    border: 0;
    background: transparent;
    padding: 2px 3px;
    color: var(--muted);
    font-size: var(--text-10);
    border-radius: 4px;
  }
  .remove-key:hover {
    color: var(--danger);
    background: var(--hover);
  }
  .remove-key[aria-pressed='true'] {
    color: var(--accent);
  }
  .optional-auth {
    padding-top: 1px;
  }
  summary {
    display: flex;
    align-items: center;
    gap: 7px;
    list-style: none;
    cursor: pointer;
    color: var(--secondary);
    font-size: var(--text-11);
  }
  summary::-webkit-details-marker {
    display: none;
  }
  summary :global(svg:last-child) {
    margin-left: auto;
    transition: transform calc(150ms * var(--motion-scale, 1));
  }
  details[open] > summary :global(svg:last-child) {
    transform: rotate(180deg);
  }
  .optional-auth summary > span {
    color: var(--muted);
    font-size: var(--text-10);
  }
  .auth-content {
    margin-top: 14px;
  }
  .connection-privacy {
    margin: 13px 4px 3px;
  }
  .connection-privacy summary {
    font-size: var(--text-10);
    color: var(--muted);
  }
  .connection-privacy p {
    margin: 10px 0 0;
    color: var(--muted);
    font-size: var(--text-11);
    line-height: 1.6;
    overflow-wrap: anywhere;
  }
  .connection-actions {
    min-height: 58px;
    padding: 11px 0;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }
  .save-connection {
    display: flex;
    align-items: center;
    gap: 10px;
    border: 1px solid transparent;
    padding: 9px 13px;
    border-radius: 8px;
    color: var(--bg);
    background: var(--accent);
    font-size: var(--text-11);
    font-weight: 500;
    box-shadow: inset 0 1px 0 #ffffff15;
  }
  .save-connection:hover:not(:disabled) {
    filter: brightness(1.07);
  }
  .save-connection:disabled {
    background: var(--surface);
    border-color: var(--border);
    color: var(--muted);
    cursor: default;
    box-shadow: none;
  }
  .save-status {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: var(--text-10);
    color: var(--secondary);
  }
  .loading {
    font-size: var(--text-11);
    color: var(--muted);
  }
  .connection-error {
    font-size: var(--text-11);
    color: var(--danger);
    line-height: 1.6;
    margin-top: 12px;
  }
  .retry {
    margin-top: 8px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text);
    padding: 7px 12px;
    border-radius: 7px;
  }
  @media (max-width: 540px) {
    .connection-grid {
      grid-template-columns: 1fr;
    }
  }
  .mcp-link {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    margin-top: 11px;
    padding: 10px 5px;
    border: 0;
    background: transparent;
    color: var(--secondary);
    font-size: var(--text-11);
    text-align: left;
  }
  .mcp-link > span {
    flex: 1;
  }
  .mcp-link:hover {
    color: var(--text);
  }
</style>
