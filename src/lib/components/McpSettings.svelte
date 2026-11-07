<script lang="ts">
  import { onMount } from 'svelte';
  import {
    Plug,
    Plus,
    Play,
    Square,
    Pencil,
    Trash2,
    X,
    Check,
    ArrowUpRight,
  } from '@lucide/svelte';
  import ConnectionField from './ConnectionField.svelte';
  import {
    mcpServers,
    loadMcpServers,
    saveMcpServer,
    removeMcpServer,
    connectMcpServer,
    disconnectMcpServer,
    type McpServerConfig,
  } from '../stores/mcp';
  let draft = $state<McpServerConfig | null>(null);
  let args = $state('[]');
  let busy = $state('');
  let error = $state('');
  let loading = $state(true);
  onMount(() => {
    void loadMcpServers()
      .catch((reason) => (error = String(reason)))
      .finally(() => (loading = false));
  });
  async function perform(id: string, operation: () => Promise<void>) {
    if (busy) return;
    busy = id;
    error = '';
    try {
      await operation();
    } catch (reason) {
      error = String(reason);
    } finally {
      busy = '';
    }
  }
  function edit(config?: McpServerConfig) {
    draft = config
      ? { ...config, args: [...config.args] }
      : { id: '', name: '', command: '', args: [] };
    args = JSON.stringify(draft.args, null, 2);
    error = '';
  }
  async function save() {
    if (!draft) return;
    let parsed: unknown;
    try {
      parsed = JSON.parse(args);
    } catch {
      error = 'Arguments must be a JSON array of strings.';
      return;
    }
    if (
      !Array.isArray(parsed) ||
      !parsed.every((value) => typeof value === 'string')
    ) {
      error = 'Arguments must be a JSON array of strings.';
      return;
    }
    const config = { ...draft, args: parsed as string[] };
    await perform('save', async () => {
      await saveMcpServer(config);
      draft = null;
    });
  }
</script>

<section class="mcp-settings" aria-label="MCP connections">
  <div class="heading">
    <span class="heading-icon"><Plug size={18} /></span>
    <h2>MCP servers</h2>
    <span class="transport">Local · stdio</span>
    {#if !draft}<button class="add" onclick={() => edit()} disabled={!!busy}
        ><Plus size={13} /> Add server</button
      >{/if}
  </div>
  <div class="content">
    {#if draft}
      <form
        onsubmit={(event) => {
          event.preventDefault();
          void save();
        }}
      >
        <div class="draft-heading">
          <h3>{draft.id ? 'Edit server' : 'New server'}</h3>
          <button
            type="button"
            class="icon-button"
            aria-label="Cancel server edits"
            disabled={!!busy}
            onclick={() => (draft = null)}><X size={14} /></button
          >
        </div>
        <fieldset disabled={!!busy}>
          <ConnectionField
            id="mcp-name"
            label="Name"
            kind="server"
            bind:value={draft.name}
            placeholder="My tools"
            required
          />
          <ConnectionField
            id="mcp-command"
            label="Executable"
            kind="command"
            bind:value={draft.command}
            placeholder="node"
            required
          />
          <div class="arguments">
            <label for="mcp-args">Arguments</label><textarea
              id="mcp-args"
              bind:value={args}
              spellcheck="false"
              placeholder={'["C:/path/to/server.mjs"]'}
              rows="3"></textarea>
          </div>
        </fieldset>
        <div class="draft-actions">
          <span>Starts when you choose Connect.</span><button
            type="submit"
            class="primary"
            disabled={!!busy}
            >{busy === 'save' ? 'Saving…' : 'Save server'}<ArrowUpRight
              size={13}
            /></button
          >
        </div>
      </form>
    {:else if loading}<p class="empty-note" role="status">Loading servers…</p>
    {:else if !$mcpServers.length}<div class="empty">
        <Plug size={25} strokeWidth={1.4} />
        <h3>Connect your tools</h3>
        <p>Add a local MCP server to make its tools available in AI.</p>
      </div>
    {:else}
      {#each $mcpServers as server (server.config.id)}
        <article class="server-card">
          <div class="server-top">
            <span class="server-icon"><Plug size={16} /></span>
            <div class="server-copy">
              <h3>{server.config.name}</h3>
              <span title={server.config.command}>{server.config.command}</span>
            </div>
            <span class="server-status" class:connected={server.connected}
              ><span></span>{busy === server.config.id
                ? 'Working…'
                : server.connected
                  ? 'Connected'
                  : 'Disconnected'}</span
            >
          </div>
          <div class="server-actions">
            {#if server.connected}<button
                disabled={!!busy}
                onclick={() =>
                  perform(server.config.id, () =>
                    disconnectMcpServer(server.config.id),
                  )}><Square size={11} /> Disconnect</button
              >
            {:else}<button
                disabled={!!busy}
                onclick={() =>
                  perform(server.config.id, () =>
                    connectMcpServer(server.config.id),
                  )}><Play size={11} /> Connect</button
              >{/if}
            <button
              class="icon-button"
              disabled={!!busy || server.connected}
              aria-label={'Edit ' + server.config.name}
              onclick={() => edit(server.config)}><Pencil size={12} /></button
            >
            <button
              class="icon-button"
              disabled={!!busy}
              aria-label={'Remove ' + server.config.name}
              onclick={() =>
                perform(server.config.id, () =>
                  removeMcpServer(server.config.id),
                )}><Trash2 size={12} /></button
            >
            {#if server.connected}<details>
                <summary
                  >{server.tools.length}
                  {server.tools.length === 1 ? 'tool' : 'tools'}</summary
                >
                <ul>
                  {#each server.tools as tool}<li>
                      <Check size={10} />
                      {tool}
                    </li>{/each}
                </ul>
              </details>{/if}
          </div>
        </article>
      {/each}
    {/if}
    {#if error}<p class="error" role="alert">{error}</p>{/if}
  </div>
</section>

<style>
  .mcp-settings {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    width: 100%;
    color: var(--text);
  }
  .heading {
    display: flex;
    align-items: center;
    gap: 10px;
    padding-bottom: 16px;
    flex-shrink: 0;
  }
  .heading-icon,
  .server-icon {
    display: grid;
    place-items: center;
    width: 33px;
    height: 33px;
    border-radius: 10px;
    background: var(--accent-soft);
    color: var(--accent);
    border: 1px solid color-mix(in srgb, var(--accent) 15%, var(--border));
    flex-shrink: 0;
  }
  h2 {
    margin: 0;
    font-size: var(--text-16);
    font-weight: 550;
  }
  h3 {
    margin: 0;
    font-size: var(--text-12);
    font-weight: 500;
  }
  .transport {
    font-size: var(--text-10);
    color: var(--muted);
  }
  button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 7px 10px;
    border-radius: 7px;
    background: var(--surface);
    color: var(--text);
    border: 1px solid var(--border);
    font-size: var(--text-11);
  }
  button:hover:not(:disabled) {
    background: var(--hover);
  }
  button:disabled {
    opacity: 0.45;
  }
  .add {
    margin-left: auto;
  }
  .content {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 0 2px 18px 0;
  }
  .empty {
    min-height: 180px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-direction: column;
    gap: 12px;
    color: var(--muted);
    text-align: center;
  }
  .empty h3 {
    color: var(--secondary);
    font-size: var(--text-14);
  }
  p {
    margin: 0;
    font-size: var(--text-11);
    line-height: 1.6;
  }
  .server-card,
  form {
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 14px;
    background: color-mix(in srgb, var(--surface) 24%, transparent);
    margin-bottom: 10px;
  }
  .server-top {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .server-copy {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .server-copy > span {
    font-size: var(--text-10);
    color: var(--muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .server-status {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: var(--text-10);
    color: var(--muted);
  }
  .server-status > span {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--muted);
  }
  .server-status.connected > span {
    background: var(--accent);
  }
  .server-actions {
    display: flex;
    align-items: center;
    gap: 7px;
    margin-top: 13px;
    flex-wrap: wrap;
  }
  .icon-button {
    width: 28px;
    height: 28px;
    padding: 0;
  }
  details {
    margin-left: auto;
    color: var(--muted);
    font-size: var(--text-10);
  }
  summary {
    cursor: pointer;
  }
  ul {
    list-style: none;
    padding: 6px 0 0;
    max-height: 120px;
    overflow: auto;
  }
  li {
    display: flex;
    align-items: center;
    gap: 5px;
    padding: 4px 0;
  }
  .draft-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 13px;
  }
  fieldset {
    display: flex;
    flex-direction: column;
    gap: 13px;
    border: 0;
    margin: 0;
    padding: 0;
    min-width: 0;
  }
  .arguments > label {
    display: block;
    margin-bottom: 8px;
    color: var(--secondary);
    font-size: var(--text-11);
  }
  textarea {
    width: 100%;
    resize: vertical;
    min-height: 64px;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: 9px;
    background: var(--surface);
    color: var(--text);
    font-family: monospace;
    font-size: var(--text-11);
    outline: none;
  }
  textarea:focus {
    border-color: var(--accent);
  }
  .draft-actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    margin-top: 14px;
  }
  .draft-actions > span {
    color: var(--muted);
    font-size: var(--text-10);
  }
  button.primary {
    background: var(--accent);
    color: var(--bg);
    border-color: transparent;
  }
  .error {
    color: var(--danger);
    margin-top: 12px;
  }
  .empty-note {
    color: var(--muted);
    padding: 12px 0;
  }
</style>
