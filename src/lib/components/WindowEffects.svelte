<script lang="ts">
  import {
    preferences,
    updatePreference,
    windowMaterials,
  } from '../stores/preferences';
  import { materialError } from '../stores/effects';
  import { isNative } from '../native';

  const labels = {
    solid: 'Solid',
    transparent: 'Transparent',
    blur: 'Blur',
    acrylic: 'Acrylic',
    mica: 'Mica',
  };
  const descriptions = {
    solid: 'An opaque background in your theme colors.',
    transparent: 'See through to the windows behind Mycast.',
    blur: 'Soften the windows behind Mycast with a frosted backdrop.',
    acrylic: 'A diffused glass backdrop with a soft, textured finish.',
    mica: 'A calm Windows 11 material tinted by your desktop wallpaper.',
  };
</script>

<section class="window-effects" aria-labelledby="effects-title">
  <div class="effects-heading">
    <h2 id="effects-title">Window effects</h2>
    <span>LIVE PREVIEW</span>
  </div>
  <div class="materials" role="group" aria-label="Window material">
    {#each windowMaterials as material}
      <button
        class:selected={$preferences.material === material}
        aria-pressed={$preferences.material === material}
        onclick={() => updatePreference('material', material)}
      >
        <span class="material-swatch" data-swatch={material} aria-hidden="true"
          ><i></i></span
        >
        {labels[material]}
      </button>
    {/each}
  </div>
  <p class="material-description">{descriptions[$preferences.material]}</p>
  {#if !isNative()}
    <p class="preview-note">
      Browser preview. Desktop materials take full effect in the Windows app.
    </p>
  {/if}
  {#if $materialError}<p class="material-error" role="alert">
      {$materialError}
    </p>{/if}
  <div class="effect-row" class:inactive={$preferences.material === 'solid'}>
    <label for="window-opacity">Background opacity</label>
    <div class="opacity-control">
      <input
        id="window-opacity"
        type="range"
        min="40"
        max="100"
        step="1"
        disabled={$preferences.material === 'solid'}
        value={$preferences.opacity}
        oninput={(event) =>
          updatePreference('opacity', Number(event.currentTarget.value))}
      />
      <output for="window-opacity"
        >{$preferences.material === 'solid'
          ? 100
          : $preferences.opacity}%</output
      >
    </div>
  </div>
  <div class="finishes">
    <label
      ><input
        type="checkbox"
        checked={$preferences.highlightBorder}
        onchange={(event) =>
          updatePreference('highlightBorder', event.currentTarget.checked)}
      /> Highlight border</label
    >
    <label
      ><input
        type="checkbox"
        checked={$preferences.glow}
        onchange={(event) =>
          updatePreference('glow', event.currentTarget.checked)}
      /> Soft accent glow</label
    >
  </div>
</section>

<style>
  .window-effects {
    margin-bottom: 24px;
    padding-bottom: 20px;
    border-bottom: 1px solid var(--border);
  }
  .effects-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 13px;
  }
  h2 {
    margin: 0;
    font-size: var(--text-16);
    font-weight: 600;
  }
  .effects-heading > span {
    color: var(--muted);
    font-size: var(--text-9);
    letter-spacing: 1px;
  }
  .materials {
    display: grid;
    grid-template-columns: repeat(5, minmax(0, 1fr));
    gap: 8px;
  }
  .materials button {
    min-width: 0;
    padding: 7px;
    border: 1px solid var(--border);
    border-radius: 7px;
    background: color-mix(in srgb, var(--surface) 50%, transparent);
    font-size: var(--text-11);
    color: var(--secondary);
  }
  .materials button:hover {
    background: var(--hover);
    color: var(--text);
  }
  .materials button.selected {
    border-color: var(--accent);
    color: var(--text);
    background: color-mix(in srgb, var(--accent) 8%, transparent);
  }
  .material-swatch {
    position: relative;
    display: block;
    height: 33px;
    margin-bottom: 7px;
    border-radius: 3px;
    overflow: hidden;
    background: linear-gradient(
      135deg,
      var(--accent),
      var(--selected) 55%,
      var(--secondary)
    );
  }
  .material-swatch i {
    position: absolute;
    inset: 6px 11px;
    border: 1px solid color-mix(in srgb, var(--text) 25%, transparent);
    border-radius: 3px;
    background: color-mix(in srgb, var(--bg) 48%, transparent);
  }
  [data-swatch='solid'] i {
    background: var(--bg);
  }
  [data-swatch='blur'] i {
    backdrop-filter: blur(4px);
  }
  [data-swatch='acrylic'] i {
    backdrop-filter: blur(9px) saturate(1.5);
    background: color-mix(in srgb, var(--bg) 65%, transparent);
  }
  [data-swatch='mica'] i {
    background: color-mix(in srgb, var(--accent) 15%, var(--bg));
  }
  .material-description {
    margin: 10px 0 0;
    font-size: var(--text-11);
    color: var(--secondary);
    line-height: 1.5;
  }
  .preview-note,
  .material-error {
    margin: 5px 0 0;
    font-size: var(--text-10);
    color: var(--muted);
    line-height: 1.5;
  }
  .material-error {
    color: var(--danger);
  }
  .effect-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 15px;
    margin-top: 16px;
    font-size: var(--text-11);
  }
  .opacity-control {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 47%;
  }
  input[type='range'] {
    width: 100%;
    min-width: 0;
    height: 16px;
    cursor: pointer;
  }
  output {
    min-width: 35px;
    text-align: right;
    color: var(--secondary);
    font-variant-numeric: tabular-nums;
  }
  .inactive {
    color: var(--muted);
  }
  .finishes {
    display: flex;
    gap: 24px;
    margin-top: 17px;
    font-size: var(--text-11);
    color: var(--secondary);
  }
  .finishes label {
    display: flex;
    align-items: center;
    gap: 7px;
    cursor: pointer;
  }
  input[type='checkbox'] {
    width: 13px;
    height: 13px;
  }
  @media (max-width: 500px) {
    .materials {
      gap: 4px;
    }
    .materials button {
      padding: 5px 2px;
      font-size: var(--text-10);
    }
    .finishes {
      gap: 12px;
      flex-wrap: wrap;
    }
  }
</style>
