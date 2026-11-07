<script lang="ts">
  import { tick } from 'svelte';
  import {
    Search,
    X,
    ArrowLeft,
    ArrowUp,
    Square,
    Sparkles,
    Globe,
  } from '@lucide/svelte';
  let {
    value = $bindable(''),
    input = $bindable<HTMLInputElement | HTMLTextAreaElement | undefined>(),
    activeId,
    mode = 'search',
    running = false,
    retry = false,
    aiAvailable = false,
    onsubmit,
    onstop,
    onkeydown,
  }: {
    value: string;
    input?: HTMLInputElement | HTMLTextAreaElement;
    activeId?: string;
    mode?: 'search' | 'ai' | 'web' | 'run';
    running?: boolean;
    retry?: boolean;
    aiAvailable?: boolean;
    onsubmit?: () => void;
    onstop?: () => void;
    onkeydown: (event: KeyboardEvent) => void;
  } = $props();
  const contextual = $derived(mode === 'ai' || mode === 'web');
  const displayValue = $derived(
    contextual
      ? value
          .trimStart()
          .slice(mode === 'ai' ? 2 : 1)
          .trimStart()
      : value,
  );
  function update(next: string) {
    if (mode === 'ai') value = next.startsWith('??') ? next : '?? ' + next;
    else if (mode === 'web')
      value =
        !displayValue && next === '?'
          ? '??'
          : next.startsWith('?')
            ? next
            : '? ' + next;
    else value = next;
  }
  $effect(() => {
    mode;
    void tick().then(() => input?.focus());
  });
  function exitMode() {
    value = '';
    input?.focus();
  }
</script>

<div class="search-header" class:contextual class:composer={mode === 'ai'}>
  {#if mode === 'ai'}<Sparkles size={17} strokeWidth={1.5} />
  {:else if contextual}
    <button
      class="back"
      aria-label="Back to search"
      title="Back to search"
      onclick={exitMode}><ArrowLeft size={18} /></button
    >
    <span class="mode-label"><Globe size={14} /> Web</span>
  {:else}<Search size={21} strokeWidth={1.7} aria-hidden="true" />{/if}

  {#if mode === 'ai'}
    <textarea
      bind:this={input}
      bind:value={() => displayValue, update}
      rows="2"
      aria-label="Ask AI"
      placeholder="Ask a question…"
      maxlength="15998"
      spellcheck="true"
      onkeydown={(event) => {
        if (event.isComposing) return;
        if (event.key === 'Enter' && !event.shiftKey) {
          event.preventDefault();
          if (!event.repeat) onsubmit?.();
          return;
        }
        if (event.key === 'Backspace' && !displayValue) {
          event.preventDefault();
          exitMode();
          return;
        }
        onkeydown(event);
      }}></textarea>
  {:else}
    <input
      bind:this={input}
      bind:value={() => displayValue, update}
      onkeydown={(event) => {
        if (contextual && !displayValue && event.key === 'Backspace') {
          event.preventDefault();
          exitMode();
          return;
        }
        onkeydown(event);
      }}
      aria-label={mode === 'web'
        ? 'Search the web'
        : 'Search apps, commands, or files; use ?? for AI, ? for web search or > to run PowerShell'}
      role="combobox"
      aria-autocomplete="list"
      aria-expanded={!!activeId}
      aria-controls="command-results"
      aria-activedescendant={activeId}
      placeholder={mode === 'web'
        ? 'Search the web…'
        : 'Search apps, commands, files…'}
      autocomplete="off"
      spellcheck="false"
      maxlength={mode === 'run' ? 8193 : 512}
    />
  {/if}
  {#if mode === 'ai'}
    {#if running}<button
        class="send stop"
        aria-label="Stop generating"
        title="Stop generating"
        onclick={onstop}><Square size={13} fill="currentColor" /></button
      >
    {:else if displayValue.trim()}<button
        class="send"
        aria-label={retry ? 'Retry prompt' : 'Send prompt'}
        title={retry ? 'Retry prompt' : 'Send prompt'}
        onclick={onsubmit}><ArrowUp size={18} /></button
      >{/if}
  {:else if value}<button
      class="clear"
      aria-label="Clear search"
      title="Clear search"
      tabindex="-1"
      onclick={() => {
        value = mode === 'web' ? '?' : '';
        input?.focus();
      }}><X size={15} /></button
    >
  {:else if aiAvailable}<button
      class="ai-shortcut"
      title="Ask AI · ??"
      aria-label="Ask AI"
      onclick={() => {
        value = '??';
        input?.focus();
      }}><Sparkles size={14} /><span>AI</span></button
    >{/if}
</div>

<style>
  .search-header {
    display: flex;
    align-items: center;
    gap: 14px;
    height: 73px;
    flex-shrink: 0;
    padding: 0 23px;
    border-bottom: 1px solid var(--border);
    color: var(--muted);
  }
  .search-header.contextual {
    gap: 12px;
    padding-left: 18px;
  }
  input,
  textarea {
    flex: 1;
    width: 100%;
    min-width: 0;
    outline: none;
    border: 0;
    background: transparent;
    color: var(--text);
    font-size: var(--text-17);
    font-weight: 400;
    letter-spacing: -0.25px;
  }
  input::placeholder,
  textarea::placeholder {
    color: var(--muted);
  }
  .mode-label {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding-right: 12px;
    border-right: 1px solid var(--border);
    color: var(--secondary);
    font-size: var(--text-12);
    font-weight: 500;
    flex-shrink: 0;
  }
  button {
    display: grid;
    place-items: center;
    flex-shrink: 0;
    width: 28px;
    height: 28px;
    padding: 0;
    background: transparent;
    border: 0;
    border-radius: 6px;
    color: var(--secondary);
  }
  button:hover {
    background: var(--hover);
    color: var(--text);
  }
  .clear {
    width: 24px;
    height: 24px;
  }
  .ai-shortcut {
    display: inline-flex;
    gap: 6px;
    width: auto;
    padding: 0 8px;
    font-size: var(--text-11);
  }
  .send {
    width: 32px;
    height: 32px;
    background: var(--accent);
    color: var(--bg);
    border-radius: 8px;
  }
  .send:hover {
    background: var(--accent);
    filter: brightness(1.12);
    color: var(--bg);
  }
  .stop {
    background: var(--surface);
    color: var(--text);
    border: 1px solid var(--border);
  }

  .search-header.composer {
    height: 78px;
    padding: 13px 14px 13px 17px;
    gap: 11px;
    align-items: flex-start;
    border: 1px solid color-mix(in srgb, var(--accent) 23%, var(--border));
    border-radius: 14px;
    background: color-mix(in srgb, var(--bg) 92%, var(--surface));
    box-shadow:
      0 4px 20px #0000000c,
      inset 0 1px 0 color-mix(in srgb, var(--text) 3%, transparent);
    transition: border-color calc(160ms * var(--motion-scale, 1));
  }
  .composer:focus-within {
    border-color: color-mix(in srgb, var(--accent) 50%, var(--border));
  }
  .composer :global(svg) {
    margin-top: 4px;
    flex-shrink: 0;
  }
  .composer textarea {
    height: 50px;
    min-height: 0;
    resize: none;
    line-height: 23px;
    font-size: var(--text-14);
    letter-spacing: 0;
    padding: 0;
    overflow-y: auto;
  }
  .composer .send {
    align-self: flex-end;
    width: 32px;
    height: 32px;
  }
  .composer .send :global(svg) {
    margin-top: 0;
  }
</style>
