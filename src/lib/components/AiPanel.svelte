<script lang="ts">
  import { onDestroy, tick } from 'svelte';
  import { marked } from 'marked';
  import DOMPurify from 'dompurify';
  import {
    aiResponse,
    aiProvider,
    providers,
    askAi,
    stopAi,
    clearAi,
  } from '../stores/ai';
  import { copyText } from '../native';
  import { invoke } from '@tauri-apps/api/core';
  let { prompt }: { prompt: string } = $props();
  let copyStatus = $state('');
  let panel = $state<HTMLElement>();
  $effect(() => {
    $aiResponse.text;
    $aiResponse.thinking;
    const element = panel;
    if (
      element &&
      element.scrollHeight - element.scrollTop - element.clientHeight < 90
    )
      void tick().then(() => {
        element.scrollTop = element.scrollHeight;
      });
  });
  const html = $derived(
    DOMPurify.sanitize(
      marked.parse($aiResponse.text, { async: false, gfm: true }) as string,
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
    ),
  );
  export function submit() {
    void askAi(prompt.trim());
  }
  onDestroy(() => {
    void clearAi();
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

<section bind:this={panel} class="ai-panel" aria-label="AI answer">
  <div class="ai-heading">
    <span>✦ {providers[$aiProvider]} <small>AI assistant</small></span><button
      onclick={() => void clearAi()}
      disabled={$aiResponse.running}>Clear</button
    >
  </div>
  {#if !$aiResponse.prompt}
    <div class="ai-empty">
      <h2>What’s on your mind?</h2>
      <p>
        {prompt.trim()
          ? 'Press Enter to ask. Your answer will appear here.'
          : 'Type ?? followed by a question, explanation, or idea.'}
      </p>
      <small>Set up your provider and model in Preferences → AI.</small>
    </div>
  {:else}
    <p class="ai-prompt">{$aiResponse.prompt}</p>
    <div class="ai-status" role="status">
      <span class:pulse={$aiResponse.running}>●</span>
      {$aiResponse.status}
    </div>
    {#if $aiResponse.thinking}<details>
        <summary>Thinking preview · supplied by model</summary>
        <pre>{$aiResponse.thinking}</pre>
      </details>{/if}
    {#if $aiResponse.text}<!-- svelte-ignore a11y_no_noninteractive_element_interactions (Delegates sanitized Markdown link activation to the native browser command.) -->
      <div
        class="markdown"
        role="document"
        aria-label="AI response"
        onclick={openLink}
        onkeydown={(event) => {
          if (
            event.key === 'Enter' &&
            event.target instanceof HTMLAnchorElement
          ) {
            event.preventDefault();
            void invoke('open_ai_link', {
              url: event.target.getAttribute('href'),
            }).catch((error) => (copyStatus = String(error)));
          }
        }}
      >
        {@html html}
      </div>{/if}
    {#if $aiResponse.error}<p role="alert" class="ai-error">
        {$aiResponse.error}
      </p>{/if}
  {/if}
  <div class="ai-actions">
    {#if $aiResponse.running}<button onclick={() => void stopAi()}
        >Stop generating</button
      >
    {:else}<button disabled={!prompt.trim()} onclick={submit}
        >{$aiResponse.error ? 'Retry' : 'Ask AI'}</button
      >{/if}
    {#if $aiResponse.text}<button onclick={copy}>Copy Markdown</button>{/if}
    <span role="status">{copyStatus}</span>
  </div>
</section>

<style>
  .ai-panel {
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding: 18px 22px;
    color: var(--text);
  }
  .ai-heading,
  .markdown :global(h1) {
    font-size: 1.65em;
    font-weight: 650;
    margin: 18px 0 10px;
  }
  .markdown :global(h2) {
    font-size: 1.4em;
    font-weight: 650;
    margin: 16px 0 8px;
  }
  .markdown :global(h3),
  .markdown :global(h4),
  .markdown :global(h5),
  .markdown :global(h6) {
    font-size: 1.15em;
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
  .ai-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }
  .ai-heading {
    color: var(--accent);
    font-weight: 600;
  }
  small {
    font-size: var(--text-11);
    color: var(--muted);
    font-weight: 400;
  }
  .ai-heading small {
    margin-left: 8px;
  }
  button {
    border: 1px solid var(--border);
    border-radius: 7px;
    background: var(--surface);
    padding: 6px 10px;
    font-size: var(--text-12);
  }
  .ai-empty {
    padding: 35px 0;
    text-align: center;
    color: var(--secondary);
  }
  h2 {
    font-size: var(--text-18);
    color: var(--text);
  }
  p {
    font-size: var(--text-13);
    line-height: 1.6;
  }
  .ai-prompt {
    color: var(--secondary);
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .ai-status {
    font-size: var(--text-12);
    color: var(--muted);
    margin: 12px 0;
  }
  .pulse {
    color: var(--accent);
    animation: pulse calc(1s * var(--motion-scale, 1)) ease-in-out infinite
      alternate;
  }
  @keyframes pulse {
    to {
      opacity: 0.3;
    }
  }
  details {
    font-size: var(--text-12);
    background: var(--surface);
    border-radius: 8px;
    padding: 10px;
    margin: 10px 0;
  }
  pre {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    max-height: 130px;
    overflow: auto;
  }
  .markdown {
    font-size: var(--text-13);
    line-height: 1.65;
    overflow-wrap: anywhere;
    user-select: text;
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
  .markdown :global(h1) {
    font-size: 1.65em;
    font-weight: 650;
    margin: 18px 0 10px;
  }
  .markdown :global(h2) {
    font-size: 1.4em;
    font-weight: 650;
    margin: 16px 0 8px;
  }
  .markdown :global(h3),
  .markdown :global(h4),
  .markdown :global(h5),
  .markdown :global(h6) {
    font-size: 1.15em;
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
  .ai-actions {
    justify-content: flex-start;
    margin-top: 18px;
  }
  .ai-error {
    color: var(--danger);
  }
</style>
