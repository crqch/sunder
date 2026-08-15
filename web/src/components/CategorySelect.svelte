<script lang="ts">
  import type { EntryCategory } from '$lib/types';
  import { ChevronDown, Plus } from '@lucide/svelte';
  import { fly } from 'svelte/transition';
  import CategoryForm from './CategoryForm.svelte';
  import Modal from './Modal.svelte';

  let { value = $bindable(0), categories } = $props<{
    value: number;
    categories: EntryCategory[];
  }>();

  let open = $state(false);
  let modalOpen = $state(false);

  let containerRef: HTMLDivElement;

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      open = false;
    }
  }

  function toggleOpen() {
    open = !open;
  }

  function selectOption(id: number) {
    value = id;
    open = false;
  }

  function openModal() {
    open = false;
    modalOpen = true;
  }

  function onCreated(newId: number) {
    value = newId;
    modalOpen = false;
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

  let selectedCat = $derived(categories.find((c) => c.id === value));
</script>

<div
  class="relative"
  bind:this={containerRef}
  onkeydown={handleKeydown}
  role="combobox"
  aria-expanded={open}
  aria-haspopup="listbox"
  aria-controls="category-list"
>
  <button
    type="button"
    onclick={toggleOpen}
    class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary flex w-full items-center justify-between rounded-lg border p-2.5 text-sm transition-all focus:ring-2 focus:outline-none"
  >
    <div class="flex items-center gap-2">
      {#if selectedCat}
        <div class="h-3 w-3 rounded-full" style="background-color: {selectedCat.color}"></div>
        <span>{selectedCat.name}</span>
      {:else}
        <span>Select Category...</span>
      {/if}
    </div>
    <ChevronDown
      size={16}
      class="text-muted-foreground transition-transform {open ? 'rotate-180' : ''}"
    />
  </button>

  {#if open}
    <div
      class="bg-card border-border/50 absolute z-10 mt-1 flex w-full origin-top flex-col overflow-hidden rounded-lg border shadow-lg"
      id="category-list"
      role="listbox"
      transition:fly={{ y: -10, duration: 200 }}
      style="perspective: 1000px;"
    >
      <div class="max-h-60 overflow-y-auto p-1">
        {#each categories as cat}
          <button
            type="button"
            role="option"
            aria-selected={value === cat.id}
            onclick={() => selectOption(cat.id)}
            class="hover:bg-muted focus:bg-muted flex w-full items-center gap-2 rounded-md px-3 py-2 text-left text-sm transition-colors focus:outline-none {value ===
            cat.id
              ? 'bg-primary/10 text-primary font-medium'
              : 'text-foreground'}"
          >
            <div class="h-3 w-3 rounded-full" style="background-color: {cat.color}"></div>
            {cat.name}
          </button>
        {/each}
        {#if categories.length === 0}
          <div class="text-muted-foreground px-3 py-2 text-center text-sm">No categories</div>
        {/if}
      </div>
      <div class="border-border/50 bg-muted/30 border-t p-1">
        <button
          type="button"
          onclick={openModal}
          class="text-primary hover:bg-primary/10 focus:bg-primary/10 flex w-full items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors focus:outline-none"
        >
          <Plus size={16} /> Create new category
        </button>
      </div>
    </div>
  {/if}
</div>

<Modal bind:open={modalOpen} title="Create New Category">
  <CategoryForm onsuccess={onCreated} oncancel={() => (modalOpen = false)} />
</Modal>
