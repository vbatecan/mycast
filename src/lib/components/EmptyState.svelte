<script lang="ts">
  import { SearchX } from '@lucide/svelte';
  let { query }: { query: string } = $props();
</script>

<div class="empty">
  <span class="empty-icon"><SearchX size={25} strokeWidth={1.4} /></span>
  <h2>No commands found</h2>
  <p>
    Nothing matches “{query.length > 45 ? query.slice(0, 45) + '…' : query}”.
  </p>
  <span>Try an app, a filename, a calculation, or &gt; to run a command.</span>
</div>

<style>
  .empty {
    flex: 1;
    min-height: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-direction: column;
    padding: 25px;
    animation: appear 150ms ease-out;
  }
  .empty-icon {
    display: grid;
    place-items: center;
    width: 50px;
    height: 50px;
    border: 1px solid var(--border);
    background: var(--surface);
    border-radius: 13px;
    color: var(--secondary);
    margin-bottom: 17px;
  }
  h2 {
    font-size: var(--text-14);
    font-weight: 500;
    margin: 0 0 7px;
  }
  p {
    font-size: var(--text-12);
    color: var(--secondary);
    margin: 0 0 7px;
    overflow-wrap: anywhere;
    text-align: center;
  }
  .empty > span:last-child {
    font-size: var(--text-11);
    color: var(--muted);
    text-align: center;
  }
  @keyframes appear {
    from {
      opacity: 0;
      transform: translateY(3px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
