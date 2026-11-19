<script lang="ts">
  import { currencyStore } from '$lib/currency.svelte';
  import type { Currency } from '$lib/types';
  import { ChevronDown, Search, Check } from '@lucide/svelte';
  import { fly } from 'svelte/transition';

  let {
    value = $bindable<string>(''),
    currencies = undefined,
    allowEmpty = false,
    emptyLabel = 'Use default currency',
    placeholder = 'Select currency...',
    disabled = false,
    onchange
  } = $props<{
    value: string;
    currencies?: Currency[];
    allowEmpty?: boolean;
    emptyLabel?: string;
    placeholder?: string;
    disabled?: boolean;
    onchange?: (code: string) => void;
  }>();

  let open = $state(false);
  let search = $state('');
  let containerRef: HTMLDivElement;
  let searchInputRef: HTMLInputElement | null = $state(null);

  let availableCurrencies = $derived<Currency[]>(currencies ?? currencyStore.currencies);

  let filteredCurrencies = $derived.by(() => {
    const q = search.trim().toLowerCase();
    if (!q) return availableCurrencies;
    return availableCurrencies.filter(
      (c: Currency) =>
        c.code.toLowerCase().includes(q) ||
        c.name.toLowerCase().includes(q) ||
        c.symbol.toLowerCase().includes(q)
    );
  });

  let selectedCurrency = $derived.by(() => {
    if (!value) return null;
    const upper = value.toUpperCase();
    return (
      availableCurrencies.find((c: Currency) => c.code.toUpperCase() === upper) ||
      currencyStore.getCurrency(upper)
    );
  });

  function toggleOpen() {
    if (disabled) return;
    open = !open;
    if (open) {
      search = '';
      setTimeout(() => searchInputRef?.focus(), 20);
    }
  }

  function selectCode(code: string) {
    value = code;
    open = false;
    search = '';
    onchange?.(code);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      open = false;
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
</script>

<div
  class="relative {open ? 'z-50' : ''}"
  bind:this={containerRef}
  onkeydown={handleKeydown}
  role="combobox"
  aria-expanded={open}
  aria-haspopup="listbox"
  aria-controls="currency-list"
>
  <button
    type="button"
    {disabled}
    onclick={toggleOpen}
    class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary flex w-full items-center justify-between gap-2 rounded-lg border p-2.5 text-sm transition-all focus:ring-2 focus:outline-none disabled:cursor-not-allowed disabled:opacity-50"
  >
    {#if selectedCurrency}
      <span class="flex min-w-0 items-center gap-2 truncate">
        <span
          class="bg-muted text-foreground rounded px-1.5 py-0.5 font-mono text-xs font-semibold"
        >
          {selectedCurrency.code}
        </span>
        <span class="text-muted-foreground shrink-0 text-xs">{selectedCurrency.symbol}</span>
        <span class="truncate">{selectedCurrency.name}</span>
      </span>
    {:else if allowEmpty}
      <span class="text-muted-foreground truncate">{emptyLabel}</span>
    {:else}
      <span class="text-muted-foreground truncate">{placeholder}</span>
    {/if}
    <ChevronDown
      size={16}
      class="text-muted-foreground shrink-0 transition-transform {open ? 'rotate-180' : ''}"
    />
  </button>

  {#if open}
    <div
      id="currency-list"
      role="listbox"
      class="bg-card border-border/50 absolute z-50 mt-1 flex w-full min-w-[260px] origin-top flex-col overflow-hidden rounded-lg border shadow-lg"
      transition:fly={{ y: -8, duration: 150 }}
    >
      <div class="border-border/50 relative border-b p-2">
        <Search
          size={14}
          class="text-muted-foreground pointer-events-none absolute top-1/2 left-4 -translate-y-1/2"
        />
        <input
          bind:this={searchInputRef}
          type="text"
          bind:value={search}
          placeholder="Search code, name, symbol..."
          class="bg-background border-border/50 focus:ring-primary/40 w-full rounded-md border py-1.5 pr-2.5 pl-8 text-xs focus:ring-2 focus:outline-none"
        />
      </div>

      <div class="max-h-60 overflow-y-auto p-1">
        {#if allowEmpty && !search.trim()}
          <button
            type="button"
            role="option"
            aria-selected={!value}
            onclick={() => selectCode('')}
            class="hover:bg-muted focus:bg-muted flex w-full items-center justify-between rounded-md px-3 py-2 text-left text-sm transition-colors focus:outline-none {!value
              ? 'bg-primary/10 text-primary font-medium'
              : 'text-muted-foreground'}"
          >
            <span>{emptyLabel}</span>
            {#if !value}
              <Check size={14} class="text-primary shrink-0" />
            {/if}
          </button>
        {/if}

        {#each filteredCurrencies as curr (curr.code)}
          {@const selected = value?.toUpperCase() === curr.code.toUpperCase()}
          <button
            type="button"
            role="option"
            aria-selected={selected}
            onclick={() => selectCode(curr.code)}
            class="hover:bg-muted focus:bg-muted flex w-full items-center justify-between gap-2 rounded-md px-3 py-2 text-left text-sm transition-colors focus:outline-none {selected
              ? 'bg-primary/10 text-primary font-medium'
              : 'text-foreground'}"
          >
            <div class="flex min-w-0 items-center gap-2">
              <span class="w-10 shrink-0 font-mono text-xs font-semibold">{curr.code}</span>
              <span class="text-muted-foreground w-8 shrink-0 truncate text-xs">{curr.symbol}</span>
              <span class="truncate text-xs">{curr.name}</span>
            </div>
            {#if selected}
              <Check size={14} class="text-primary shrink-0" />
            {/if}
          </button>
        {/each}

        {#if filteredCurrencies.length === 0}
          <div class="text-muted-foreground px-3 py-4 text-center text-xs">
            {availableCurrencies.length === 0
              ? 'Loading currencies from server...'
              : 'No matching currencies'}
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>
