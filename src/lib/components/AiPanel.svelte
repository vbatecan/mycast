<script lang="ts">
  import { onDestroy, tick } from 'svelte';
  import {
    Sparkles,
    Copy,
    RotateCcw,
    Settings2,
    ArrowLeft,
    Plug,
  } from '@lucide/svelte';
  import { marked } from 'marked';
  import DOMPurify from 'dompurify';
  import {
    aiResponse,
    aiPastTurns,
    aiProvider,
    providers,
    askAi,
    getAiConfig,
    clearAi,
    stopAi,
  } from '../stores/ai';
  import { copyText, isNative } from '../native';
  import { invoke } from '@tauri-apps/api/core';
  import { pendingToolApproval, approveTool, mcpServers } from '../stores/mcp';
  let {
    prompt,
    onsettings,
    onnew,
    onexit,
    onmcp,
  }: {
    prompt: string;
    onsettings: () => void;
    onnew: () => void;
    onexit: () => void;
    onmcp: () => void;
  } = $props();
  let configured = $state<boolean | null>(null);
  let model = $state('');
  const providerLabel = $derived(
    $aiProvider === 'custom' ? 'Custom endpoint' : providers[$aiProvider],
  );
  $effect(() => {
    const provider = $aiProvider;
    let cancelled = false;
    configured = null;
    model = '';
    if (!isNative()) return;
    void getAiConfig(provider)
      .then((config) => {
        if (!cancelled && config) {
          model = config.model;
          configured =
            !!config.model.trim() &&
            (!['openai', 'claude', 'gemini'].includes(provider) ||
              config.hasKey);
        }
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  });
  $effect(() => {
    $aiResponse.prompt;
    copyStatus = '';
  });
  let copyStatus = $state('');
  let panel = $state<HTMLElement>();
  $effect(() => {
    $aiResponse.text;
    $aiResponse.thinking;
    $pendingToolApproval;
    const element = panel;
    if (
      element &&
      element.scrollHeight - element.scrollTop - element.clientHeight < 90
    )
      void tick().then(() => {
        element.scrollTop = element.scrollHeight;
      });
  });
  function renderMarkdown(text: string) {
    return DOMPurify.sanitize(
      marked.parse(text, { async: false, gfm: true }) as string,
      {
        ALLOWED_TAGS: [
          'p',
          'br',
          'hr',
          'h1',
          'h2',
          'h3',
          'h4',
          'h5',
          'h6',
          'strong',
          'em',
          'del',
          's',
          'ul',
          'ol',
          'li',
          'blockquote',
          'pre',
          'code',
          'table',
          'thead',
          'tbody',
          'tr',
          'th',
          'td',
          'a',
        ],
        ALLOWED_ATTR: ['href', 'title', 'class'],
        FORBID_ATTR: ['style', 'target'],
      },
    );
  }
  const html = $derived(renderMarkdown($aiResponse.text));
  export function submit() {
    if ($aiResponse.running) return;
    if (configured === false) {
      onsettings();
      return;
    }
    const question = prompt.trim();
    if (!question) return;
    void askAi(question);
    onnew();
  }
  onDestroy(() => {
    void stopAi();
  });
  async function copy() {
    try {
      await copyText($aiResponse.text);
      copyStatus = 'Copied';
    } catch (error) {
      copyStatus = String(error);
    }
  }
  function openLink(event: MouseEvent) {
    const anchor = (event.target as HTMLElement).closest('a');
    if (!anchor) return;
    event.preventDefault();
    const href = anchor.getAttribute('href');
    if (href)
      void invoke('open_ai_link', { url: href }).catch((error) => {
        copyStatus = String(error);
      });
  }
</script>

<section class="ai-panel" aria-label="AI answer">
  <header class="ai-toolbar">
    <button
      class="back-to-search icon-button"
      onclick={onexit}
      aria-label="Back to search"
      title="Back to search"><ArrowLeft size={17} /></button
    >
    <span class="assistant-mark"><Sparkles size={14} /> AI</span>
    <span class="provider" title={providerLabel + (model ? ' / ' + model : '')}
      >{model || providerLabel}</span
    >
    <div class="toolbar-actions">
      {#if $mcpServers.some((server) => server.connected)}<button
          class="icon-button tools-connected"
          onclick={onmcp}
          aria-label="MCP servers"
          title="MCP servers"><Plug size={14} /></button
        >{/if}
      {#if $aiResponse.text && !$aiResponse.running}<button
          class="icon-button"
          onclick={copy}
          aria-label="Copy response"
          title="Copy response"><Copy size={14} /></button
        >{/if}
      {#if $aiResponse.prompt && !$aiResponse.running}<button
          class="icon-button"
          onclick={() => {
            void clearAi();
            onnew();
          }}
          aria-label="New conversation"
          title="New conversation"><RotateCcw size={14} /></button
        >{/if}
      <button
        class="icon-button"
        onclick={onsettings}
        aria-label="AI settings"
        title="AI settings"><Settings2 size={15} /></button
      >
    </div>
  </header>

  <!-- svelte-ignore a11y_no_noninteractive_element_interactions (Delegates sanitized Markdown link activation to the native browser command.) -->
  <div
    class="ai-content"
    bind:this={panel}
    role="log"
    aria-label="Conversation"
    aria-live="off"
    onclick={openLink}
    onkeydown={(event) => {
      if (event.key === 'Enter' && event.target instanceof HTMLAnchorElement) {
        event.preventDefault();
        void invoke('open_ai_link', {
          url: event.target.getAttribute('href'),
        }).catch((error) => (copyStatus = String(error)));
      }
    }}
  >
    {#each $aiPastTurns as turn}
      <article class="past-turn">
        <div class="submitted-prompt"><p>{turn.prompt}</p></div>
        {#if turn.text}<div class="markdown">
            {@html renderMarkdown(turn.text)}
          </div>{/if}
        {#if turn.error}<p class="past-status">
            This request failed.
          </p>{:else if turn.status === 'Stopped'}<p class="past-status">
            Stopped
          </p>{/if}
      </article>
    {/each}

    {#if !$aiResponse.prompt}
      <div class="ai-empty">
        <Sparkles size={24} strokeWidth={1.4} />
        <h2>
          {configured === false ? 'Connect an AI provider' : 'Ask anything'}
        </h2>
        {#if configured === false}<button
            class="setup-button"
            onclick={onsettings}>Set up AI</button
          >{/if}
      </div>
    {:else}
      <div class="submitted-prompt">
        <p>{$aiResponse.prompt}</p>
      </div>
      <div class="ai-status" role="status">
        {#if $aiResponse.running}<span class="status-dot"
          ></span>{$aiResponse.status}
        {:else if !$aiResponse.error && $aiResponse.status !== 'Complete'}{$aiResponse.status}{/if}
      </div>
      {#if $aiResponse.thinking}<details class="thinking">
          <summary>Thinking</summary>
          <pre>{$aiResponse.thinking}</pre>
        </details>{/if}
      {#if $aiResponse.text}
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions (Delegates sanitized Markdown links to the native browser command.) -->
        <div class="markdown" role="document" aria-label="AI response">
          {@html html}
        </div>
      {/if}
      {#if $aiResponse.error}<div class="ai-error" role="alert">
          <h3>Couldn’t get an answer</h3>
          <p>{$aiResponse.error}</p>
          <div class="error-actions">
            <button
              onclick={() => {
                void askAi($aiResponse.prompt, true);
                onnew();
              }}>Retry prompt</button
            ><button onclick={onsettings}>AI settings</button>
          </div>
        </div>{/if}
    {/if}
    {#if $pendingToolApproval}<section
        class="tool-approval"
        aria-label="Tool request"
      >
        <div class="tool-heading">
          <Plug size={15} /><strong>{$pendingToolApproval.tool}</strong><span
            >{$pendingToolApproval.server}</span
          >
        </div>
        <pre>{JSON.stringify($pendingToolApproval.arguments, null, 2)}</pre>
        <div class="tool-actions">
          <button
            onclick={() =>
              void approveTool(false).catch(
                (reason) => (copyStatus = String(reason)),
              )}>Deny</button
          ><button
            class="allow-tool"
            onclick={() =>
              void approveTool(true).catch(
                (reason) => (copyStatus = String(reason)),
              )}>Allow once</button
          >
        </div>
      </section>{/if}
  </div>
  <span class="copy-status" role="status">{copyStatus}</span>
</section>

<style>
  .ai-panel {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    color: var(--text);
  }
  .ai-toolbar {
    height: 56px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: flex-start;
    gap: 10px;
    padding: 0 18px;
    border-bottom: 1px solid var(--border);
  }
  .provider {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--secondary);
    font-size: var(--text-11);
  }
  .toolbar-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
    margin-left: auto;
  }
  button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    border: 0;
    border-radius: 5px;
    background: transparent;
    padding: 6px 8px;
    color: var(--secondary);
    font-size: var(--text-11);
  }
  button:hover {
    color: var(--text);
    background: var(--hover);
  }
  .icon-button {
    width: 28px;
    height: 28px;
    padding: 0;
  }
  .ai-content {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 18px 26px 20px;
    margin-bottom: 94px;
  }
  .ai-empty {
    height: 100%;
    min-height: 130px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-direction: column;
    gap: 14px;
    padding-bottom: 30px;
    color: var(--muted);
  }
  h2 {
    margin: 0;
    font-size: var(--text-18);
    font-weight: 500;
    color: var(--secondary);
    letter-spacing: -0.3px;
  }
  .setup-button {
    border: 1px solid var(--border);
    background: var(--surface);
    margin-top: 1px;
    padding: 7px 12px;
    color: var(--text);
  }
  .submitted-prompt {
    margin: 0 0 8px auto;
    max-width: 88%;
    width: fit-content;
    padding: 10px 14px;
    border: 1px solid color-mix(in srgb, var(--accent) 12%, var(--border));
    border-radius: 13px 13px 4px 13px;
    background: color-mix(in srgb, var(--accent) 6%, var(--surface));
    color: var(--secondary);
    font-size: var(--text-12);
  }
  .submitted-prompt p {
    margin: 0;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    max-height: 96px;
    overflow: auto;
    line-height: 1.6;
  }
  .ai-status {
    min-height: 26px;
    display: flex;
    align-items: center;
    gap: 7px;
    color: var(--muted);
    font-size: var(--text-11);
  }
  .status-dot {
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--accent);
  }
  .thinking {
    font-size: var(--text-11);
    color: var(--muted);
    margin: 8px 0 14px;
  }
  .thinking summary {
    cursor: pointer;
  }
  .thinking pre {
    font: inherit;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    max-height: 130px;
    overflow: auto;
    line-height: 1.6;
  }
  .copy-status {
    min-height: 0;
    color: var(--secondary);
    font-size: var(--text-10);
    padding: 0 26px;
  }
  .copy-status:not(:empty) {
    padding-bottom: 10px;
  }
  .markdown {
    font-size: var(--text-13);
    line-height: 1.65;
    overflow-wrap: anywhere;
    user-select: text;
  }
  .markdown :global(h1) {
    font-size: 1.5em;
    font-weight: 600;
    margin: 18px 0 10px;
  }
  .markdown :global(h2) {
    font-size: 1.3em;
    font-weight: 600;
    margin: 16px 0 8px;
  }
  .markdown :global(h3),
  .markdown :global(h4),
  .markdown :global(h5),
  .markdown :global(h6) {
    font-size: 1.1em;
    font-weight: 600;
    margin: 14px 0 8px;
  }
  .markdown :global(p) {
    margin: 10px 0;
  }
  .markdown :global(ul) {
    list-style: disc;
    padding-left: 24px;
    margin: 10px 0;
  }
  .markdown :global(ol) {
    list-style: decimal;
    padding-left: 24px;
    margin: 10px 0;
  }
  .markdown :global(hr) {
    border-color: var(--border);
    margin: 16px 0;
  }
  .markdown :global(pre) {
    background: var(--surface);
    padding: 12px;
    overflow: auto;
    white-space: pre;
    max-height: none;
    border-radius: 8px;
  }
  .markdown :global(code) {
    font-size: 0.9em;
    background: var(--surface);
  }
  .markdown :global(table) {
    border-collapse: collapse;
    display: block;
    overflow: auto;
  }
  .markdown :global(td),
  .markdown :global(th) {
    border: 1px solid var(--border);
    padding: 6px;
  }
  .markdown :global(a) {
    color: var(--accent);
  }
  .markdown :global(blockquote) {
    border-left: 2px solid var(--accent);
    margin-left: 0;
    padding-left: 12px;
    color: var(--secondary);
  }
  .ai-error {
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px;
    margin-top: 8px;
  }
  .ai-error h3 {
    margin: 0 0 7px;
    font-size: var(--text-13);
    font-weight: 500;
  }
  .ai-error p {
    margin: 0 0 9px;
    color: var(--secondary);
    font-size: var(--text-12);
    line-height: 1.6;
  }

  .assistant-mark {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
    color: var(--text);
    font-size: var(--text-12);
    font-weight: 500;
    padding-right: 10px;
    border-right: 1px solid var(--border);
  }
  .back-to-search {
    margin-left: -6px;
  }
  .error-actions {
    display: flex;
    gap: 8px;
  }
  .copy-status {
    position: absolute;
    bottom: 98px;
    right: 28px;
    padding: 0;
  }
  .past-turn {
    padding-bottom: 22px;
    margin-bottom: 20px;
    border-bottom: 1px solid var(--border);
  }
  .past-status {
    font-size: var(--text-11);
    color: var(--muted);
  }
  .tools-connected {
    color: var(--accent);
  }
  .tool-approval {
    margin-top: 14px;
    padding: 14px;
    border-radius: 12px;
    border: 1px solid color-mix(in srgb, var(--accent) 30%, var(--border));
    background: color-mix(in srgb, var(--surface) 70%, transparent);
  }
  .tool-heading {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--secondary);
    font-size: var(--text-11);
  }
  .tool-heading strong {
    color: var(--text);
    font-weight: 500;
  }
  .tool-heading > span {
    margin-left: auto;
    color: var(--muted);
  }
  .tool-approval pre {
    margin: 12px 0;
    max-height: 120px;
    overflow: auto;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    font-size: var(--text-11);
    color: var(--secondary);
  }
  .tool-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
  .tool-actions button {
    border: 1px solid var(--border);
    padding: 7px 12px;
  }
  .tool-actions .allow-tool {
    background: var(--accent);
    color: var(--bg);
    border-color: transparent;
  }
</style>
