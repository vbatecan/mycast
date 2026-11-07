<script module lang="ts">
  export interface Choice {
    value: string;
    label: string;
    caption?: string;
    mark: string;
  }
</script>

<script lang="ts">
  import { tick } from 'svelte';
  import { ChevronDown, Check } from '@lucide/svelte';
  let {
    id,
    label,
    value,
    choices,
    disabled = false,
    onchange,
  }: {
    id: string;
    label: string;
    value: string;
    choices: Choice[];
    disabled?: boolean;
    onchange: (value: string) => void;
  } = $props();
  let open = $state(false);
  let trigger = $state<HTMLButtonElement>();
  let menu = $state<HTMLDivElement>();
  let position = $state({ left: 0, top: 0, width: 0, maxHeight: 240 });
  let highlight = 0;
  const selected = $derived(
    choices.find((choice) => choice.value === value) ?? choices[0],
  );
  function close(focus = true) {
    menu?.hidePopover();
    open = false;
    if (focus) trigger?.focus();
  }
  async function show() {
    if (!trigger || disabled) return;
    const rect = trigger.getBoundingClientRect();
    const below = window.innerHeight - rect.bottom - 12;
    const height = Math.min(
      240,
      below < 160 && rect.top > below ? rect.top - 12 : below,
    );
    position = {
      left: rect.left,
      top:
        below < 160 && rect.top > below
          ? rect.top - height - 6
          : rect.bottom + 6,
      width: rect.width,
      maxHeight: height,
    };
    highlight = Math.max(
      0,
      choices.findIndex((choice) => choice.value === value),
    );
    open = true;
    await tick();
    menu?.showPopover();
    menu
      ?.querySelectorAll<HTMLButtonElement>('[role="option"]')
      [highlight]?.focus();
  }
  function choose(choice: Choice) {
    onchange(choice.value);
    close();
  }
  function navigate(event: KeyboardEvent) {
    if (event.key === 'Tab') {
      close(false);
      return;
    }
    if (
      ![
        'ArrowDown',
        'ArrowUp',
        'ArrowRight',
        'ArrowLeft',
        'Home',
        'End',
      ].includes(event.key)
    )
      return;
    event.preventDefault();
    event.stopPropagation();
    if (event.key === 'Home') highlight = 0;
    else if (event.key === 'End') highlight = choices.length - 1;
    else
      highlight =
        (highlight +
          (['ArrowDown', 'ArrowRight'].includes(event.key)
            ? 1
            : choices.length - 1)) %
        choices.length;
    menu
      ?.querySelectorAll<HTMLButtonElement>('[role="option"]')
      [highlight]?.focus();
  }
</script>

<svelte:window
  onkeydowncapture={(event) => {
    if (open && event.key === 'Escape') {
      event.preventDefault();
      event.stopPropagation();
      close();
    }
  }}
  onresize={() => {
    if (open) close(false);
  }}
/>
<div class="choice-picker">
  <label for={id}>{label}</label>
  <button
    class="picker-trigger"
    {id}
    bind:this={trigger}
    type="button"
    role="combobox"
    aria-label={label}
    aria-expanded={open}
    aria-haspopup="listbox"
    aria-controls={id + '-options'}
    {disabled}
    onclick={() => (open ? close() : void show())}
    onkeydown={(event) => {
      if (['ArrowDown', 'ArrowUp'].includes(event.key)) {
        event.preventDefault();
        void show();
      }
    }}
  >
    <span class="choice-mark" aria-hidden="true">{selected.mark}</span>
    <span class="choice-copy"
      ><span class="choice-name">{selected.label}</span
      >{#if selected.caption}<span class="choice-caption"
          >{selected.caption}</span
        >{/if}</span
    >
    <ChevronDown size={14} />
  </button>
  <div
    class="choice-menu"
    id={id + '-options'}
    popover="auto"
    role="listbox"
    aria-label={label}
    bind:this={menu}
    style:left={position.left + 'px'}
    style:top={position.top + 'px'}
    style:width={position.width + 'px'}
    style:max-height={position.maxHeight + 'px'}
    ontoggle={(event) => {
      if (event.newState === 'closed') open = false;
    }}
  >
    {#each choices as choice, index}<button
        class="choice-option"
        type="button"
        role="option"
        aria-label={choice.label}
        aria-selected={value === choice.value}
        tabindex="-1"
        onfocus={() => (highlight = index)}
        onclick={() => choose(choice)}
        onkeydown={navigate}
      >
        <span class="choice-mark" aria-hidden="true">{choice.mark}</span><span
          class="choice-copy"
          ><span class="choice-name">{choice.label}</span
          >{#if choice.caption}<span class="choice-caption"
              >{choice.caption}</span
            >{/if}</span
        >
        {#if choice.value === value}<Check size={13} />{/if}
      </button>{/each}
  </div>
</div>

<style>
  label {
    display: block;
    margin-bottom: 8px;
    font-size: var(--text-11);
    color: var(--secondary);
  }
  .picker-trigger {
    display: flex;
    align-items: center;
    gap: 11px;
    width: 100%;
    height: 48px;
    padding: 8px 12px;
    text-align: left;
    background: color-mix(in srgb, var(--surface) 76%, transparent);
    border: 1px solid var(--border);
    border-radius: 10px;
    color: var(--text);
    transition: border-color calc(140ms * var(--motion-scale, 1));
  }
  .picker-trigger .choice-caption {
    display: none;
  }
  .picker-trigger:hover,
  .picker-trigger[aria-expanded='true'] {
    border-color: color-mix(in srgb, var(--accent) 50%, var(--border));
  }
  .picker-trigger:disabled {
    opacity: 0.5;
  }
  .choice-mark {
    flex-shrink: 0;
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    border: 1px solid color-mix(in srgb, var(--accent) 14%, var(--border));
    border-radius: 8px;
    background: var(--accent-soft);
    color: var(--accent);
    font-size: var(--text-12);
    font-weight: 600;
  }
  .choice-copy {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }
  .choice-name {
    font-size: var(--text-12);
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .choice-caption {
    font-size: var(--text-10);
    color: var(--muted);
  }
  .choice-menu {
    position: fixed;
    inset: auto;
    margin: 0;
    padding: 6px;
    overflow-y: auto;
    grid-template-columns: 1fr 1fr;
    gap: 4px;
    border: 1px solid var(--border);
    border-radius: 12px;
    background: var(--bg);
    box-shadow: 0 12px 40px #0004;
    color: var(--text);
  }
  .choice-menu:popover-open {
    display: grid;
  }
  .choice-option {
    display: flex;
    align-items: center;
    gap: 9px;
    border: 1px solid transparent;
    border-radius: 8px;
    padding: 9px 8px;
    background: transparent;
    color: var(--text);
    text-align: left;
    min-width: 0;
  }
  .choice-option:hover,
  .choice-option:focus-visible {
    background: var(--hover);
    outline: none;
    border-color: var(--border);
  }
  .choice-option[aria-selected='true'] {
    background: var(--accent-soft);
  }
  .choice-option .choice-mark {
    width: 25px;
    height: 25px;
    font-size: var(--text-11);
  }
  .choice-option :global(svg) {
    flex-shrink: 0;
    color: var(--accent);
  }
</style>
