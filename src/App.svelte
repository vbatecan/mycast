<script lang="ts">
  import { watchUpdates } from './lib/stores/updates';
  import { onMount } from 'svelte';
  import CommandPalette from './lib/components/CommandPalette.svelte';
  import { preferences } from './lib/stores/preferences';
  import { isNative, setWindowMaterial } from './lib/native';
  import { materialError } from './lib/stores/effects';
  import {
    activeTheme,
    applyTheme,
    observeSystemTheme,
  } from './lib/stores/themes';

  onMount(observeSystemTheme);
  onMount(watchUpdates);
  const selectedMaterial = $derived($preferences.material);
  const darkMaterial = $derived($activeTheme.scheme === 'dark');
  $effect(() => applyTheme($activeTheme));
  $effect(() => {
    const root = document.documentElement;
    const material = selectedMaterial;
    const dark = darkMaterial;
    let cancelled = false;
    root.dataset.native = String(isNative());
    materialError.set('');
    if (isNative()) {
      void setWindowMaterial(material, dark)
        .then(() => {
          if (!cancelled) root.dataset.material = material;
        })
        .catch((reason) => {
          if (!cancelled) {
            root.dataset.material = 'solid';
            materialError.set(
              `This material is unavailable. Using Solid. ${String(reason)}`,
            );
          }
        });
    } else root.dataset.material = material;
    return () => {
      cancelled = true;
    };
  });
  $effect(() => {
    document.documentElement.style.setProperty(
      '--text-scale',
      String($preferences.textSize / 100),
    );
    document.documentElement.dataset.density = $preferences.density;
    document.documentElement.style.setProperty(
      '--window-opacity',
      `${$preferences.opacity}%`,
    );
    document.documentElement.dataset.glow = String($preferences.glow);
    document.documentElement.dataset.highlightBorder = String(
      $preferences.highlightBorder,
    );
  });
</script>

<main class:motion-off={!$preferences.animations}>
  <CommandPalette />
</main>
