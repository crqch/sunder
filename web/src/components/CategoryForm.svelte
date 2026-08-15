<script lang="ts">
  import { db } from '$lib/db';
  import { Save } from '@lucide/svelte';

  let { onsuccess, oncancel } = $props<{
    onsuccess: (id: number) => void;
    oncancel?: () => void;
  }>();

  let name = $state('');
  let description = $state('');
  let color = $state('#ef4444');

  const PRESET_COLORS = [
    '#ef4444',
    '#f97316',
    '#f59e0b',
    '#84cc16',
    '#22c55e',
    '#10b981',
    '#06b6d4',
    '#3b82f6',
    '#6366f1',
    '#8b5cf6',
    '#d946ef',
    '#f43f5e',
    '#000000',
    '#52525b',
    '#78716c'
  ];

  async function saveCategory(e: Event) {
    e.preventDefault();
    if (!name.trim()) return;

    const now = new Date().toISOString();
    const id = Date.now();
    const newCategory = {
      id,
      name,
      description: description.trim() || null,
      color,
      deleted_at: null,
      created_at: now,
      updated_at: now
    };

    await db.entry_categories.add(newCategory);
    onsuccess(id);
  }
</script>

<form onsubmit={saveCategory} class="space-y-5">
  <div class="space-y-1.5">
    <label for="cat-name" class="text-foreground block text-sm font-medium">Category Name</label>
    <input
      id="cat-name"
      type="text"
      bind:value={name}
      class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary w-full rounded-lg border p-2.5 text-sm transition-all focus:ring-2 focus:outline-none"
      placeholder="e.g. Groceries, Rent, Salary"
      required
      autofocus
    />
  </div>

  <div class="space-y-1.5">
    <label for="cat-description" class="text-foreground block text-sm font-medium"
      >Description (Optional)</label
    >
    <textarea
      id="cat-description"
      bind:value={description}
      rows="2"
      class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary w-full resize-none rounded-lg border p-2.5 text-sm transition-all focus:ring-2 focus:outline-none"
      placeholder="Brief description..."></textarea>
  </div>

  <div class="border-border/50 space-y-3 border-t pt-4">
    <label class="text-foreground block text-sm font-medium">Color</label>

    <div class="flex items-center gap-3">
      <input
        type="color"
        bind:value={color}
        class="h-10 w-10 cursor-pointer rounded border-0 bg-transparent p-0"
      />
      <input
        type="text"
        bind:value={color}
        class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary flex-1 rounded-lg border p-2.5 text-sm uppercase focus:ring-2 focus:outline-none"
        pattern="^#[0-9A-Fa-f]{'{6}'}$"
      />
    </div>

    <div class="mt-3 grid grid-cols-5 gap-2 sm:grid-cols-8">
      {#each PRESET_COLORS as preset}
        <button
          type="button"
          onclick={() => (color = preset)}
          class="border-border/20 h-8 rounded-full border {color === preset
            ? 'ring-primary ring-offset-background ring-2 ring-offset-2'
            : ''} shadow-sm transition-transform hover:scale-110"
          style="background-color: {preset}"
          aria-label="Select color {preset}"
        ></button>
      {/each}
    </div>
  </div>

  <div class="mt-4 flex gap-3 pt-2">
    {#if oncancel}
      <button
        type="button"
        onclick={oncancel}
        class="btn-outline flex w-full flex-1 items-center justify-center"
      >
        Cancel
      </button>
    {/if}
    <button type="submit" class="btn flex w-full flex-1 items-center justify-center gap-2">
      <Save size={16} /> Create
    </button>
  </div>
</form>
