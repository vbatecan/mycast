<script lang="ts">
  import { onDestroy, onMount, untrack } from 'svelte';
  import { ArrowLeft, Check, CircleAlert, Eye } from '@lucide/svelte';
  import {
    previewTheme,
    resetPreview,
    saveTheme,
    themeNameLimit,
    validateThemeDraft,
    type ThemeDraft,
  } from '../stores/themes';

  let {
    draft,
    ondone,
    oncancel,
  }: { draft: ThemeDraft; ondone: () => void; oncancel: () => void } = $props();
  let name = $state(untrack(() => draft.name));
  let scheme = $state<'dark' | 'light'>(untrack(() => draft.scheme));
  let colors = $state(untrack(() => ({ ...draft.colors })));
  let error = $state('');
  let nameInput: HTMLInputElement;
  const groups = [
    { name: 'Base', fields: ['background', 'surface', 'hover', 'selected'] },
    { name: 'Content', fields: ['text', 'secondary', 'muted'] },
    { name: 'Accent', fields: ['border', 'accent'] },
  ] as const;
  const labels = {
    background: 'Background',
    surface: 'Surface',
    hover: 'Hover',
    selected: 'Selection',
    text: 'Primary text',
    secondary: 'Secondary text',
    muted: 'Muted text',
    border: 'Border',
    accent: 'Accent',
  };
  const validHex = (value: string) => /^#[0-9a-f]{6}$/i.test(value);
  let fieldsValid = $derived(Object.values(colors).every(validHex));

  onMount(() => {
    nameInput?.focus();
    nameInput?.select();
  });
  onDestroy(resetPreview);

  function currentDraft(): ThemeDraft {
    return { id: draft.id, name, scheme, colors: { ...colors } };
  }

  $effect(() => {
    if (!fieldsValid) {
      error = 'Use six-digit hex colors, for example #17171B.';
      return;
    }
    const current = currentDraft();
    const validation = validateThemeDraft(current);
    const result = previewTheme({
      ...current,
      name: name.trim() || 'Theme preview',
    });
    error = !validation.ok ? validation.error : result.ok ? '' : result.error;
  });

  function save(event: SubmitEvent) {
    event.preventDefault();
    if (!fieldsValid) return;
    const result = saveTheme(currentDraft());
    if (!result.ok) {
      error = result.error;
      return;
    }
    resetPreview();
    ondone();
  }
</script>

<form class="theme-editor" onsubmit={save}>
  <div class="editor-heading">
    <button
      class="sub-back"
      type="button"
      onclick={oncancel}
      aria-label="Cancel theme editing"><ArrowLeft size={14} /></button
    >
    <div>
      <h2>{draft.id ? 'Edit theme' : 'Create a theme'}</h2>
      <p>A personal touch, in every detail.</p>
    </div>
    <span class="live-badge"><Eye size={11} /> Live preview</span>
  </div>

  <div class="identity-fields">
    <label class="name-field"
      ><span>Theme name</span><input
        bind:this={nameInput}
        bind:value={name}
        maxlength={themeNameLimit}
        placeholder="My theme"
        required
        autocomplete="off"
      /></label
    >
    <label class="scheme-field"
      ><span>Color scheme</span><select bind:value={scheme}
        ><option value="dark">Dark</option><option value="light">Light</option
        ></select
      ></label
    >
  </div>

  {#each groups as group}
    <fieldset>
      <legend>{group.name}</legend>
      <div class="color-grid">
        {#each group.fields as field}
          <div class="color-field" role="group" aria-label={labels[field]}>
            <span class="color-label">{labels[field]}</span><span
              class="color-control"
              class:invalid={!validHex(colors[field])}
            >
              <input
                class="swatch-input"
                type="color"
                value={validHex(colors[field]) ? colors[field] : '#000000'}
                oninput={(event) => (colors[field] = event.currentTarget.value)}
                aria-label={`${labels[field]} color picker`}
              />
              <input
                class="hex-input"
                bind:value={colors[field]}
                spellcheck="false"
                maxlength="7"
                pattern="#[0-9a-fA-F]{6}"
                aria-label={`${labels[field]} hex color`}
                aria-invalid={!validHex(colors[field])}
                required
              />
            </span>
          </div>
        {/each}
      </div>
    </fieldset>
  {/each}

  <div class="editor-footer">
    <div class="editor-status" aria-live="polite">
      {#if error}<CircleAlert size={12} /><span>{error}</span>{:else}<Check
          size={12}
        /><span>Previewing your theme. Save to keep it.</span>{/if}
    </div>
    <div class="editor-actions">
      <button type="button" class="secondary-button" onclick={oncancel}
        >Cancel</button
      ><button
        type="submit"
        class="save-button"
        disabled={!fieldsValid || !!error}
        ><Check size={12} /> Save theme</button
      >
    </div>
  </div>
</form>

<style>
  .theme-editor {
    color: var(--text);
  }
  .editor-heading {
    display: flex;
    align-items: center;
    gap: 9px;
    margin-bottom: 18px;
  }
  .sub-back {
    width: 25px;
    height: 27px;
    border: 1px solid var(--border);
    border-radius: 6px;
    display: grid;
    place-items: center;
    background: transparent;
    color: var(--secondary);
  }
  h2 {
    font-size: var(--text-15);
    font-weight: 550;
    letter-spacing: -0.2px;
    margin: 0;
  }
  .editor-heading p {
    color: var(--muted);
    font-size: var(--text-10);
    margin: 4px 0 0;
  }
  .live-badge {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: var(--text-9);
    color: var(--accent);
    border: 1px solid var(--border);
    padding: 4px 6px;
    border-radius: 5px;
  }
  .identity-fields {
    display: flex;
    gap: 12px;
    margin-bottom: 18px;
  }
  .identity-fields label {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .identity-fields label > span {
    color: var(--secondary);
    font-size: var(--text-10);
  }
  .name-field {
    flex: 1;
    min-width: 0;
  }
  .scheme-field {
    width: 110px;
  }
  .identity-fields input,
  select {
    height: 30px;
    background: var(--surface);
    color: var(--text);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 0 9px;
    font: inherit;
    font-size: var(--text-11);
    min-width: 0;
  }
  input:focus-visible,
  select:focus-visible,
  button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  fieldset {
    border: 0;
    padding: 0;
    margin: 0 0 15px;
    min-width: 0;
  }
  legend {
    font-size: var(--text-9);
    text-transform: uppercase;
    letter-spacing: 0.9px;
    color: var(--muted);
    font-weight: 550;
    margin-bottom: 9px;
    padding: 0;
  }
  .color-grid {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 10px;
  }
  .color-field {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
  }
  .color-label {
    font-size: var(--text-10);
    color: var(--secondary);
    white-space: nowrap;
  }
  .color-control {
    display: flex;
    align-items: center;
    height: 31px;
    padding: 4px 6px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface);
    gap: 6px;
  }
  .color-control.invalid {
    border-color: var(--accent);
  }
  .swatch-input {
    appearance: none;
    -webkit-appearance: none;
    border: 0;
    background: none;
    width: 19px;
    height: 19px;
    padding: 0;
    flex-shrink: 0;
    cursor: pointer;
    border-radius: 4px;
    overflow: hidden;
  }
  .swatch-input::-webkit-color-swatch-wrapper {
    padding: 0;
  }
  .swatch-input::-webkit-color-swatch {
    border-radius: 4px;
    border: 1px solid color-mix(in srgb, var(--text) 15%, transparent);
  }
  .hex-input {
    width: 100%;
    min-width: 0;
    font:
      10px 'Cascadia Code',
      Consolas,
      monospace;
    color: var(--text);
    background: none;
    border: 0;
    padding: 3px 0;
    text-transform: uppercase;
  }
  .editor-status {
    display: flex;
    gap: 6px;
    align-items: flex-start;
    color: var(--secondary);
    min-height: 25px;
    font-size: var(--text-10);
    line-height: 1.4;
  }
  .editor-footer {
    position: sticky;
    bottom: -18px;
    background: var(--bg);
    border-top: 1px solid var(--border);
    margin-top: 8px;
    padding: 9px 0 12px;
    z-index: 2;
  }
  .editor-status :global(svg) {
    flex-shrink: 0;
    margin-top: 1px;
    color: var(--accent);
  }
  .editor-actions {
    display: flex;
    justify-content: flex-end;
    gap: 7px;
    padding-top: 7px;
  }
  .editor-actions button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 29px;
    border: 1px solid var(--border);
    border-radius: 6px;
    font: inherit;
    font-size: var(--text-11);
    padding: 0 12px;
  }
  .secondary-button {
    background: transparent;
    color: var(--secondary);
  }
  .save-button {
    background: var(--accent);
    color: var(--bg);
    font-weight: 600;
  }
  .save-button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }
  button:hover:not(:disabled) {
    filter: brightness(1.08);
  }
  @media (max-width: 520px) {
    .color-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
</style>
