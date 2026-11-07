<script lang="ts">
  import {
    Cpu,
    Link,
    KeyRound,
    Search,
    Eye,
    EyeOff,
    Check,
    Server,
    Terminal,
  } from '@lucide/svelte';
  let {
    id,
    label,
    value = $bindable(''),
    kind = 'model',
    placeholder = '',
    hint = '',
    disabled = false,
    required = false,
    saved = false,
    oninput,
  }: {
    id: string;
    label: string;
    value: string;
    kind?: 'model' | 'endpoint' | 'key' | 'search' | 'server' | 'command';
    placeholder?: string;
    hint?: string;
    disabled?: boolean;
    required?: boolean;
    saved?: boolean;
    oninput?: () => void;
  } = $props();
  let reveal = $state(false);
  const Icon = $derived(
    {
      model: Cpu,
      endpoint: Link,
      key: KeyRound,
      search: Search,
      server: Server,
      command: Terminal,
    }[kind],
  );
</script>

<div class="connection-field">
  <div class="field-label">
    <label for={id}>{label}</label>{#if saved}<span class="saved-badge"
        ><Check size={10} /> Saved</span
      >{/if}
  </div>
  <div class="field-surface" class:disabled>
    <Icon size={15} strokeWidth={1.5} />
    <input
      {id}
      bind:value
      type={kind === 'key' && !reveal
        ? 'password'
        : kind === 'endpoint'
          ? 'url'
          : 'text'}
      {placeholder}
      {disabled}
      {required}
      {oninput}
      autocomplete={kind === 'key' ? 'new-password' : 'off'}
      aria-describedby={hint ? id + '-hint' : undefined}
    />
    {#if kind === 'key' && value && !disabled}<button
        type="button"
        aria-label={reveal ? 'Hide API key' : 'Show API key'}
        onclick={() => (reveal = !reveal)}
        >{#if reveal}<EyeOff size={14} />{:else}<Eye size={14} />{/if}</button
      >{/if}
  </div>
  {#if hint}<p id={id + '-hint'}>{hint}</p>{/if}
</div>

<style>
  .connection-field {
    min-width: 0;
  }
  .field-label {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 16px;
    margin-bottom: 8px;
  }
  label {
    font-size: var(--text-11);
    color: var(--secondary);
  }
  .saved-badge {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    color: var(--muted);
    font-size: var(--text-9);
  }
  .field-surface {
    display: flex;
    align-items: center;
    gap: 9px;
    min-height: 48px;
    padding: 0 12px;
    border: 1px solid var(--border);
    border-radius: 9px;
    background: color-mix(in srgb, var(--surface) 76%, transparent);
    color: var(--muted);
    transition:
      border-color calc(140ms * var(--motion-scale, 1)),
      box-shadow calc(140ms * var(--motion-scale, 1));
  }
  .field-surface:focus-within {
    border-color: color-mix(in srgb, var(--accent) 55%, var(--border));
    box-shadow: 0 0 0 2px var(--accent-soft);
  }
  .field-surface :global(svg) {
    flex-shrink: 0;
  }
  .disabled {
    opacity: 0.5;
  }
  input {
    min-width: 0;
    width: 100%;
    height: 46px;
    border: 0;
    background: transparent;
    outline: none;
    color: var(--text);
    font-size: var(--text-12);
    padding: 0;
  }
  input::placeholder {
    color: var(--muted);
  }
  button {
    display: grid;
    place-items: center;
    width: 24px;
    height: 26px;
    flex-shrink: 0;
    border: 0;
    background: transparent;
    color: var(--secondary);
    border-radius: 4px;
    padding: 0;
  }
  button:hover {
    background: var(--hover);
    color: var(--text);
  }
  p {
    color: var(--muted);
    font-size: var(--text-10);
    line-height: 1.5;
    margin: 7px 0 0;
  }
</style>
