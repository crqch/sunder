<script lang="ts">
  import { liveQuery } from 'dexie';
  import { db } from '$lib/db';
  import type { EntryCategory } from '$lib/types';
  import { Plus, ChevronRight } from '@lucide/svelte';
  import { modals } from '$lib/modals.svelte';
  import { tooltip } from '$lib/tooltip';

  let categories = $state<EntryCategory[]>([]);

  $effect(() => {
    const sub = liveQuery(() =>
      db.entry_categories.filter((c) => !c.deleted_at).toArray()
    ).subscribe({ next: (v) => (categories = v) });
    return () => sub.unsubscribe();
  });
  let searchQuery = $state('');
  let filteredCategories = $derived(
    categories.filter((c) => c.name.toLowerCase().includes(searchQuery.toLowerCase()))
  );
</script>

<div class="space-y-8 font-sans">
  <div class="border-border/50 flex flex-col gap-4 border-b pb-4">
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-semibold tracking-tight">Categories</h1>
      <button
        use:tooltip={{ text: 'New Category', keys: ['Alt', 'N', 'C'] }}
        onclick={() => (modals.createCategory = true)}
        class="btn shrink-0 gap-2 text-sm"
      >
        <Plus size={16} /> Add
      </button>
    </div>
    <div class="relative w-full">
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Search categories..."
        class="input h-10 w-full"
      />
    </div>
  </div>

  {#if categories.length === 0}
    <div
      class="border-border/60 text-muted-foreground bg-card/50 rounded-xl border border-dashed p-12 text-center text-sm"
    >
      No categories found. Create your first category!
    </div>
  {:else if filteredCategories.length === 0}
    <div
      class="border-border/60 text-muted-foreground bg-card/50 rounded-xl border border-dashed p-12 text-center text-sm"
    >
      No categories match your search.
    </div>
  {:else}
    <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 md:grid-cols-3">
      {#each filteredCategories as category}
        <a
          href="/categories/{category.id}"
          class="bg-card border-border/50 hover:border-border group block rounded-xl border p-5 shadow-sm transition-all hover:shadow-md"
        >
          <div class="mb-2 flex items-center gap-3">
            <div
              class="h-4 w-4 rounded-full shadow-sm"
              style="background-color: {category.color}"
            ></div>
            <h2 class="truncate text-lg font-medium tracking-tight">{category.name}</h2>
          </div>
          {#if category.description}
            <p class="text-muted-foreground line-clamp-2 text-sm">{category.description}</p>
          {/if}
        </a>
      {/each}
    </div>
  {/if}
</div>
