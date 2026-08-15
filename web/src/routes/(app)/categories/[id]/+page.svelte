<script lang="ts">
  import { page } from '$app/stores';
  import { liveQuery } from 'dexie';
  import { db } from '$lib/db';
  import type { EntryCategory, AccountEntry, Account } from '$lib/types';
  import { goto } from '$app/navigation';
  import { Trash2, Pencil, Check, X, ArrowLeft } from '@lucide/svelte';
  import { toast } from 'svelte-sonner';

  let id = $derived(Number($page.params.id));

  let category = $state<EntryCategory | null>(null);
  let entries = $state<AccountEntry[]>([]);
  let accounts = $state<Account[]>([]);

  let editing = $state(false);
  let editName = $state('');
  let editDescription = $state('');
  let editColor = $state('');

  $effect(() => {
    const subCat = liveQuery(() => db.entry_categories.get(id)).subscribe({
      next: (v) => {
        if (v && v.deleted_at) {
          category = null;
        } else {
          category = v || null;
          if (category && !editing) {
            editName = category.name;
            editDescription = category.description || '';
            editColor = category.color;
          }
        }
      }
    });
    const subEnt = liveQuery(() =>
      db.account_entries
        .where('category_id')
        .equals(id)
        .filter((e) => !e.deleted_at)
        .reverse()
        .sortBy('created_at')
    ).subscribe({ next: (v) => (entries = v) });
    const subAcc = liveQuery(() => db.accounts.toArray()).subscribe({
      next: (v) => (accounts = v)
    });

    return () => {
      subCat.unsubscribe();
      subEnt.unsubscribe();
      subAcc.unsubscribe();
    };
  });

  async function handleDelete() {
    if (!category) return;

    if (entries.length > 0) {
      toast.error(`Cannot delete category while it has ${entries.length} active entries.`);
      return;
    }

    if (confirm('Are you sure you want to delete this category?')) {
      await db.entry_categories.update(id, {
        deleted_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      });
      goto('/categories');
    }
  }

  async function handleSaveEdit() {
    if (!category || !editName.trim()) return;
    await db.entry_categories.update(id, {
      name: editName,
      description: editDescription.trim() || null,
      color: editColor,
      updated_at: new Date().toISOString()
    });
    editing = false;
  }

  function getAccountName(accId: number) {
    return accounts.find((a) => a.id === accId)?.name || 'Unknown';
  }
</script>

{#if !category}
  <div class="text-muted-foreground p-12 text-center font-bold tracking-widest uppercase">
    Category not found or deleted.
    <br /><br />
    <a href="/categories" class="btn btn-outline inline-block">Back to Categories</a>
  </div>
{:else}
  <div class="space-y-8 font-sans">
    <div class="border-border/50 flex items-center gap-4 border-b pb-4">
      <a href="/categories" class="btn-icon">
        <ArrowLeft size={20} />
      </a>

      {#if editing}
        <div
          class="bg-card border-border/50 flex flex-1 flex-col gap-4 rounded-xl border p-4 shadow-sm"
        >
          <input
            type="text"
            bind:value={editName}
            class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary w-full rounded-lg border p-2.5 text-sm focus:ring-2 focus:outline-none"
            placeholder="Name"
          />
          <textarea
            bind:value={editDescription}
            class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary w-full resize-none rounded-lg border p-2.5 text-sm focus:ring-2 focus:outline-none"
            placeholder="Description"></textarea>
          <div class="flex items-center gap-3">
            <input
              type="color"
              bind:value={editColor}
              class="h-10 w-10 cursor-pointer rounded border-0 bg-transparent p-0"
            />
            <div class="ml-auto flex gap-2">
              <button
                onclick={handleSaveEdit}
                class="btn btn-sm bg-primary text-primary-foreground hover:bg-primary/90"
                >Save</button
              >
              <button
                onclick={() => {
                  editing = false;
                  editName = category!.name;
                  editDescription = category!.description || '';
                  editColor = category!.color;
                }}
                class="btn-outline btn-sm">Cancel</button
              >
            </div>
          </div>
        </div>
      {:else}
        <div
          class="h-8 w-8 shrink-0 rounded-full shadow-sm"
          style="background-color: {category.color}"
        ></div>
        <div class="flex-1">
          <h1 class="text-2xl font-semibold tracking-tight">{category.name}</h1>
          {#if category.description}
            <p class="text-muted-foreground mt-0.5 text-sm">{category.description}</p>
          {/if}
        </div>
        <div class="flex gap-2">
          <button onclick={() => (editing = true)} class="btn-icon" title="Edit">
            <Pencil size={18} />
          </button>
          <button onclick={handleDelete} class="btn-icon-destructive" title="Delete">
            <Trash2 size={18} />
          </button>
        </div>
      {/if}
    </div>

    <div class="pt-4">
      <div class="border-border/50 mb-4 flex items-center justify-between border-b pb-2">
        <h2 class="text-lg font-semibold tracking-tight">Entries using this category</h2>
      </div>

      {#if entries.length === 0}
        <div
          class="border-border/60 text-muted-foreground bg-card/50 rounded-xl border border-dashed p-12 text-center text-sm"
        >
          No entries use this category.
        </div>
      {:else}
        <div class="space-y-2">
          {#each entries as entry}
            <a
              href="/entries/{entry.id}"
              class="group bg-card border-border/50 hover:border-border block rounded-lg border p-3.5 shadow-sm transition-all hover:shadow-md"
            >
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-foreground text-sm font-medium">{entry.title}</p>
                  <div class="text-muted-foreground mt-0.5 flex items-center gap-1.5 text-xs">
                    <span>{getAccountName(entry.account_id)}</span>
                    {#if entry.location}
                      <span>•</span>
                      <span>{entry.location}</span>
                    {/if}
                  </div>
                </div>
                <div class="text-right">
                  <p
                    class="text-sm font-semibold {entry.amount >= 0
                      ? 'text-emerald-500'
                      : 'text-destructive'}"
                  >
                    {entry.amount > 0 ? '+' : ''}{entry.amount.toFixed(2)}
                  </p>
                  <p class="text-muted-foreground mt-0.5 text-xs">
                    {new Date(entry.created_at).toLocaleDateString()}
                  </p>
                </div>
              </div>
            </a>
          {/each}
        </div>
      {/if}
    </div>
  </div>
{/if}
