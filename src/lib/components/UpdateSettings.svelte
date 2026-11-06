<script lang="ts">
  import {
    autoUpdates,
    updateState,
    checkUpdates,
    installUpdate,
    restartUpdatedApp,
  } from '../stores/updates';
</script>

<h2>Updates</h2>
<p>
  Updates are downloaded from Mycast’s GitHub Releases and verified before
  installation.
</p>
<label
  ><input type="checkbox" bind:checked={$autoUpdates} /> Automatically check for updates</label
>
<p>
  Checks after startup and every six hours. Nothing downloads or installs until
  you choose it.
</p>
<button
  onclick={() => checkUpdates()}
  disabled={['checking', 'downloading', 'installed'].includes(
    $updateState.status,
  )}>Check for updates</button
>
<a
  href="https://github.com/vbatecan/mycast/releases"
  target="_blank"
  rel="noreferrer">GitHub releases</a
>
<div role="status" class="status">
  {#if $updateState.status === 'checking'}Checking GitHub…
  {:else if $updateState.status === 'current'}You have the latest available
    version.
  {:else if $updateState.status === 'available'}<strong
      >Version {$updateState.version} is available</strong
    >
    <pre>{$updateState.notes ?? 'A new version of Mycast is available.'}</pre>
    <p>
      Mycast will close during installation. Windows may request permission.
      Save unfinished work before continuing.
    </p>
    <button onclick={() => installUpdate()}>Download and install</button>
  {:else if $updateState.status === 'downloading'}Downloading and verifying… {Math.round(
      ($updateState.downloaded ?? 0) / 1024,
    )} KB{#if $updateState.total}
      / {Math.round($updateState.total / 1024)} KB{/if}
  {:else if $updateState.status === 'installed'}Update installed. <button
      onclick={() => restartUpdatedApp()}>Restart Mycast</button
    >
  {/if}
</div>
{#if $updateState.error}<p role="alert">{$updateState.error}</p>
  <p>You can retry later or download the installer from GitHub.</p>{/if}

<style>
  h2 {
    margin: 0;
    font-size: var(--text-16);
    font-weight: 600;
  }
  p,
  label,
  .status,
  a {
    font-size: var(--text-11);
    line-height: 1.5;
    color: var(--secondary);
  }
  label {
    display: flex;
    gap: 8px;
    align-items: center;
    margin: 20px 0 6px;
  }
  button {
    padding: 7px 10px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    font-size: var(--text-11);
  }
  a {
    color: var(--accent);
    margin-left: 12px;
  }
  .status {
    margin-top: 20px;
  }
  pre {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    max-height: 160px;
    overflow: auto;
    font: inherit;
    color: var(--muted);
  }
  [role='alert'] {
    color: var(--danger);
  }
</style>
