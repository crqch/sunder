<script lang="ts">
  import { db } from '$lib/db';
  import { Save } from '@lucide/svelte';

  let {
    onsuccess,
    oncancel,
    isDirty = $bindable(false)
  } = $props<{
    onsuccess: (id: number) => void;
    oncancel?: () => void;
    isDirty?: boolean;
  }>();

  let name = $state('');

  $effect(() => {
    isDirty = name.trim() !== '';
  });

  async function saveAccount(e: Event) {
    e.preventDefault();
    if (!name.trim()) return;

    const now = new Date().toISOString();
    const id = Date.now();
    const newAccount = {
      id,
      name,
      deleted_at: null,
      created_at: now,
      updated_at: now
    };

    await db.accounts.add(newAccount);
    onsuccess(id);
  }
</script>

<form onsubmit={saveAccount} class="space-y-5">
  <div class="space-y-1.5">
    <label for="acc-name" class="text-foreground block text-sm font-medium">Account Name</label>
    <input
      id="acc-name"
      type="text"
      bind:value={name}
      class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary w-full rounded-lg border p-2.5 text-sm transition-all focus:ring-2 focus:outline-none"
      placeholder="e.g. Checking, Savings, Cash"
      required
      autofocus
    />
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
