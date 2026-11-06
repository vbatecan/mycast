<script lang="ts">
  import type { CommandScope } from '../stores/applications';
  import { modules } from '../stores/modules';
  import { preferences } from '../stores/preferences';
  import { shortcutLabel } from '../shortcuts';
  let {
    scope,
    onselect,
  }: { scope: CommandScope; onselect: (scope: CommandScope) => void } =
    $props();
  const scopes = $derived(
    (
      [
        { id: 'all', label: 'All' },
        { id: 'applications', label: 'Applications' },
        { id: 'commands', label: 'Commands' },
        { id: 'files', label: 'Files' },
        { id: 'favorites', label: 'Favorites' },
        { id: 'recents', label: 'Recents' },
      ] as { id: CommandScope; label: string }[]
    )
      .filter((entry) => entry.id !== 'applications' || $modules.applications)
      .filter((entry) => entry.id !== 'files' || $modules.files),
  );
  function navigate(event: KeyboardEvent, index: number) {
    if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) return;
    event.preventDefault();
    const next =
      event.key === 'Home'
        ? 0
        : event.key === 'End'
          ? scopes.length - 1
          : (index + (event.key === 'ArrowRight' ? 1 : -1) + scopes.length) %
            scopes.length;
    onselect(scopes[next].id);
  }
</script>

<div class="catalogue-bar">
  <div class="scopes" role="toolbar" aria-label="Search filters">
    {#each scopes as entry, index}
      <button
        type="button"
        aria-pressed={scope === entry.id}
        tabindex={scope === entry.id ? 0 : -1}
        class:active={scope === entry.id}
        title={entry.label +
          ' · ' +
          shortcutLabel($preferences.shortcuts[entry.id])}
        onmousedown={(event) => event.preventDefault()}
        onclick={() => onselect(entry.id)}
        onkeydown={(event) => navigate(event, index)}>{entry.label}</button
      >
    {/each}
  </div>
</div>

<style>
  .catalogue-bar {
    min-height: 39px;
    flex-shrink: 0;
    padding: 5px 19px 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    flex-wrap: wrap;
  }
  .scopes {
    display: flex;
    gap: 3px;
    align-items: center;
    flex-wrap: wrap;
  }
  .scopes button {
    border: 0;
    background: transparent;
    color: var(--muted);
    padding: 4px 8px;
    font-size: var(--text-10);
    border-radius: 5px;
    transition:
      color calc(120ms * var(--motion-scale, 1)),
      background calc(120ms * var(--motion-scale, 1));
  }
  .scopes button.active {
    color: var(--text);
    background: var(--surface);
  }
  .scopes button:hover {
    background: var(--hover);
    color: var(--text);
  }
</style>
