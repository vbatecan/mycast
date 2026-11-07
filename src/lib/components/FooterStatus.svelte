<script lang="ts">
  import { CircleAlert, Check, LoaderCircle } from '@lucide/svelte';
  let {
    text = '',
    detail = '',
    description = '',
    tone = 'neutral',
    onretry,
    onsettings,
    settingsLabel = 'Settings',
  }: {
    text?: string;
    detail?: string;
    description?: string;
    tone?: 'neutral' | 'progress' | 'success' | 'error' | 'warning';
    onretry?: () => void;
    onsettings?: () => void;
    settingsLabel?: string;
  } = $props();
  let expanded = $state(false);
  let trigger = $state<HTMLButtonElement>();
  let container = $state<HTMLDivElement>();
  $effect(() => {
    text;
    detail;
    expanded = false;
  });
  function close() {
    expanded = false;
    trigger?.focus();
  }
</script>

<svelte:window
  onkeydowncapture={(event) => {
    if (expanded && event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      close();
    }
  }}
  onpointerdown={(event) => {
    if (
      expanded &&
      event.target instanceof Node &&
      !container?.contains(event.target)
    )
      expanded = false;
  }}
/>

<div class="footer-status" class:error={tone === 'error'} bind:this={container}>
  <div class="announcement" role="status" aria-live="polite" aria-atomic="true">
    {#if text}
      {#if tone === 'error' || tone === 'warning'}<CircleAlert size={12} />
      {:else if tone === 'success'}<Check size={12} />
      {:else if tone === 'progress'}<LoaderCircle size={12} />{/if}
      <span title={text}>{text}</span>
    {/if}
  </div>
  {#if detail || onretry || onsettings}
    <button
      class="details-trigger"
      bind:this={trigger}
      aria-expanded={expanded}
      aria-label={`Details: ${text}`}
      onclick={() => (expanded = !expanded)}>Details</button
    >
  {/if}
  {#if expanded}
    <section class="status-panel" aria-label="Status details">
      <div class="panel-heading">
        <strong>{text}</strong><button
          onclick={close}
          aria-label="Close status details">Close</button
        >
      </div>
      {#if description}<p>{description}</p>{/if}
      <div class="actions">
        {#if onretry}<button
            onclick={() => {
              expanded = false;
              onretry?.();
            }}>Retry</button
          >{/if}
        {#if onsettings}<button
            onclick={() => {
              expanded = false;
              onsettings?.();
            }}>{settingsLabel}</button
          >{/if}
      </div>
      {#if detail}<details>
          <summary>Technical details</summary>
          <pre>{detail}</pre>
        </details>{/if}
    </section>
  {/if}
</div>

<style>
  .footer-status {
    display: flex;
    align-items: center;
    gap: 7px;
    flex: 1;
    min-width: 0;
    color: var(--secondary);
    font-size: var(--text-10);
  }
  .announcement {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }
  .announcement :global(svg) {
    flex-shrink: 0;
  }
  .announcement span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .error .announcement {
    color: var(--danger);
  }
  button {
    border: 1px solid var(--border);
    border-radius: 5px;
    padding: 5px 8px;
    background: var(--surface);
    color: var(--text);
    font: inherit;
  }
  button:hover {
    background: var(--hover);
  }
  .details-trigger {
    flex-shrink: 0;
    border: 0;
    background: transparent;
    color: var(--secondary);
    padding: 3px 0;
    text-decoration: underline;
    text-underline-offset: 3px;
  }
  .status-panel {
    position: absolute;
    z-index: 5;
    bottom: 51px;
    left: 20px;
    right: 20px;
    max-height: min(240px, 65%);
    overflow: auto;
    padding: 15px;
    border: 1px solid var(--border);
    border-radius: 9px;
    background: var(--bg);
    box-shadow: 0 8px 25px #0005;
    color: var(--text);
    font-size: var(--text-12);
  }
  .panel-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  strong {
    font-weight: 500;
  }
  p {
    color: var(--secondary);
    line-height: 1.5;
    margin: 8px 0 12px;
  }
  .actions {
    display: flex;
    gap: 8px;
  }
  details {
    margin-top: 12px;
    color: var(--secondary);
  }
  summary {
    cursor: pointer;
    font-size: var(--text-11);
  }
  pre {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    font: inherit;
    font-size: var(--text-11);
    max-height: 120px;
    overflow: auto;
  }
</style>
