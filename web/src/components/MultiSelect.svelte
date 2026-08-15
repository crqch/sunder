<script lang="ts">
  import { ChevronDown, Check } from '@lucide/svelte';
  import { fly } from 'svelte/transition';

  let {
    value = $bindable<number[]>([]),
    options,
    placeholder = 'Select options...'
  } = $props<{
    value: number[];
    options: { id: number; name: string; color?: string }[];
    placeholder?: string;
  }>();

  let open = $state(false);
  let containerRef: HTMLDivElement;

  function toggleOption(id: number) {
    if (value.includes(id)) {
      value = value.filter((v) => v !== id);
    } else {
      value = [...value, id];
    }
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

  let selectedText = $derived(() => {
    if (value.length === 0) return placeholder;
    if (value.length === 1) return options.find((o) => o.id === value[0])?.name || placeholder;
    return `${value.length} selected`;
  });
</script>

<div class="relative" bind:this={containerRef}>
  <button
    type="button"
    onclick={() => (open = !open)}
    class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary flex w-full items-center justify-between rounded-lg border p-2.5 text-sm transition-all focus:ring-2 focus:outline-none"
  >
    <span class="truncate">{selectedText()}</span>
    <ChevronDown
      size={16}
      class="text-muted-foreground transition-transform {open ? 'rotate-180' : ''} ml-2 shrink-0"
    />
  </button>

  {#if open}
    <div
      class="bg-card border-border/50 absolute z-10 mt-1 flex w-full origin-top flex-col overflow-hidden rounded-lg border shadow-lg"
      transition:fly={{ y: -10, duration: 200 }}
    >
      <div class="max-h-60 overflow-y-auto p-1">
        {#each options as opt}
          <button
            type="button"
            onclick={() => toggleOption(opt.id)}
            class="hover:bg-muted focus:bg-muted flex w-full items-center justify-between rounded-md px-3 py-2 text-left text-sm transition-colors focus:outline-none {value.includes(
              opt.id
            )
              ? 'bg-primary/5 font-medium'
              : 'text-foreground'}"
          >
            <div class="flex items-center gap-2">
              {#if opt.color}
                <div class="h-2 w-2 rounded-full" style="background-color: {opt.color}"></div>
              {/if}
              {opt.name}
            </div>
            {#if value.includes(opt.id)}
              <Check size={14} class="text-primary" />
            {/if}
          </button>
        {/each}
        {#if options.length === 0}
          <div class="text-muted-foreground px-3 py-2 text-center text-sm">
            No options available
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>
