<script lang="ts">
  let { keys = [] }: { keys?: string[] } = $props();
</script>

<span class="keyboard-hint" aria-label={keys.join(' + ')}>
  {#each keys as key}
    <kbd>{key}</kbd>
  {/each}
</span>

<style>
  .keyboard-hint {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    white-space: nowrap;
  }
  kbd {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 20px;
    min-height: 20px;
    padding-block: 2px;
    padding: 0 5px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--surface);
    box-shadow: 0 1px 0 rgba(0, 0, 0, 0.18);
    color: var(--secondary);
    font-family: inherit;
    font-size: var(--text-10);
    font-weight: 500;
    line-height: 1;
  }
</style>
