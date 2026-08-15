<script lang="ts">
  import { ChevronDown, Check } from '@lucide/svelte';
  import { fly } from 'svelte/transition';

  let {
    value = $bindable(),
    options,
    placeholder = 'Select...'
  } = $props<{
    value: string | number;
    options: { value: string | number; label: string }[];
    placeholder?: string;
  }>();

  let open = $state(false);
  let containerRef: HTMLDivElement;

  function selectOption(v: string | number) {
    value = v;
    open = false;
  }

  $effect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (open && containerRef && !containerRef.contains(event.target as Node)) {
        open = false;
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  });

  let selectedText = $derived(options.find((o) => o.value === value)?.label || placeholder);
</script>

<div class="relative" bind:this={containerRef}>
  <button
    type="button"
    onclick={() => (open = !open)}
    class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary flex w-full items-center justify-between rounded-lg border p-2.5 text-sm transition-all focus:ring-2 focus:outline-none"
  >
    <span class="truncate">{selectedText}</span>
    <ChevronDown
      size={16}
      class="text-muted-foreground transition-transform {open ? 'rotate-180' : ''} ml-2 shrink-0"
    />
  </button>

  {#if open}
    <div
      class="bg-card border-border/50 absolute z-50 mt-1 flex w-full origin-top flex-col overflow-hidden rounded-lg border shadow-lg"
      transition:fly={{ y: -10, duration: 200 }}
    >
      <div class="max-h-60 overflow-y-auto p-1">
        {#each options as opt}
          <button
            type="button"
            onclick={() => selectOption(opt.value)}
            class="hover:bg-muted focus:bg-muted flex w-full items-center justify-between rounded-md px-3 py-2 text-left text-sm transition-colors focus:outline-none {value ===
            opt.value
              ? 'bg-primary/5 font-medium'
              : 'text-foreground'}"
          >
            {opt.label}
            {#if value === opt.value}
              <Check size={14} class="text-primary" />
            {/if}
          </button>
        {/each}
      </div>
    </div>
  {/if}
</div>
