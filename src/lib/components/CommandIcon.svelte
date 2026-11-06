<script lang="ts">
  import {
    AppWindow,
    FileText,
    Calculator,
    FolderOpen,
    Settings2,
    CalendarDays,
    Clock3,
    Fingerprint,
    Download,
    Files,
    Monitor,
    Info,
    Equal,
    SlidersHorizontal,
    Palette,
    Terminal,
  } from '@lucide/svelte';
  import {
    iconGeneration,
    requestApplicationIcon,
  } from '../applications/icons';

  let { name, applicationId }: { name: string; applicationId?: string } =
    $props();
  const icons: Record<string, typeof FileText> = {
    'notebook-pen': FileText,
    calculator: Calculator,
    'folder-open': FolderOpen,
    'settings-2': Settings2,
    'calendar-days': CalendarDays,
    'clock-3': Clock3,
    fingerprint: Fingerprint,
    download: Download,
    files: Files,
    monitor: Monitor,
    info: Info,
    'sliders-horizontal': SlidersHorizontal,
    equal: Equal,
    result: Equal,
    palette: Palette,
    'app-window': AppWindow,
    'file-text': FileText,
    terminal: Terminal,
  };
  let Icon = $derived(icons[name] ?? AppWindow);
  let element = $state<HTMLSpanElement>();
  let iconUrl = $state<string | null>(null);
  let imageFailed = $state(false);

  $effect(() => {
    const id = applicationId;
    const target = element;
    $iconGeneration;
    iconUrl = null;
    imageFailed = false;
    if (!id || !target) return;
    const controller = new AbortController();
    const load = () => {
      void requestApplicationIcon(id, controller.signal).then((value) => {
        if (!controller.signal.aborted) iconUrl = value;
      });
    };
    if (typeof IntersectionObserver === 'undefined') {
      load();
      return () => controller.abort();
    }
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries.some((entry) => entry.isIntersecting)) {
          observer.disconnect();
          load();
        }
      },
      { root: target.closest('[role="listbox"]'), rootMargin: '64px' },
    );
    observer.observe(target);
    return () => {
      controller.abort();
      observer.disconnect();
    };
  });
</script>

<span class="row-icon" bind:this={element} aria-hidden="true">
  {#if iconUrl && !imageFailed}
    <img
      src={iconUrl}
      alt=""
      width="26"
      height="26"
      decoding="async"
      draggable="false"
      onerror={() => (imageFailed = true)}
    />
  {:else}
    <Icon size={18} strokeWidth={1.7} />
  {/if}
</span>

<style>
  .row-icon {
    height: 100%;
    width: 100%;
    display: grid;
    place-items: center;
  }
  img {
    display: block;
    height: 26px;
    width: 26px;
    object-fit: contain;
  }
</style>
