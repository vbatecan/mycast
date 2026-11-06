<script lang="ts">
  import {
    moduleDefinitions,
    modules,
    setModule,
    initializeModules,
    type ModuleId,
  } from '../stores/modules';
  import { onMount } from 'svelte';
  let busy = $state(false);
  let error = $state('');
  onMount(() => {
    void initializeModules().catch((reason) => (error = String(reason)));
  });
  async function toggle(id: ModuleId) {
    busy = true;
    error = '';
    try {
      await setModule(id, !$modules[id]);
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = false;
    }
  }
</script>

<h2>Modules</h2>
<p>
  Enable what you use. Providers load when needed; disabling Files stops its
  scanner and watcher. Saved favorites, aliases, and settings are retained.
</p>
{#each moduleDefinitions as [id, label, description]}<div class="module-row">
    <div><strong>{label}</strong><small>{description}</small></div>
    <button
      role="switch"
      aria-label={label}
      aria-checked={$modules[id]}
      disabled={busy}
      onclick={() => toggle(id)}>{$modules[id] ? 'Enabled' : 'Disabled'}</button
    >
  </div>{/each}
{#if error}<p role="alert">{error}</p>{/if}

<style>
  h2 {
    font-size: var(--text-16);
    font-weight: 600;
    margin: 0;
  }
  p {
    font-size: var(--text-11);
    color: var(--muted);
    line-height: 1.5;
  }
  .module-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 20px;
    padding: 14px 0;
    border-bottom: 1px solid var(--border);
  }
  strong {
    font-size: var(--text-12);
    font-weight: 500;
  }
  small {
    display: block;
    font-size: var(--text-10);
    color: var(--muted);
    margin-top: 5px;
  }
  button {
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 6px 10px;
    background: var(--surface);
    font-size: var(--text-11);
  }
  button[aria-checked='true'] {
    color: var(--accent);
  }
</style>
