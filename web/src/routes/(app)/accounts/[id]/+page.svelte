<script lang="ts">
  import { page } from '$app/stores';
  import { liveQuery } from 'dexie';
  import { db } from '$lib/db';
  import type { Account, AccountEntry, EntryCategory } from '$lib/types';
  import { goto } from '$app/navigation';
  import { Plus, Trash2, Pencil, Check, X, ArrowLeft } from '@lucide/svelte';
  import Modal from '$components/Modal.svelte';
  import EntryForm from '$components/EntryForm.svelte';

  let id = $derived($page.params.id);

  let account = $state<Account | null>(null);
  let entries = $state<AccountEntry[]>([]);
  let categories = $state<EntryCategory[]>([]);

  let editing = $state(false);
  let editName = $state('');
  let createEntryModalOpen = $state(false);

  $effect(() => {
    const subAcc = liveQuery(() => db.accounts.get(id)).subscribe({
      next: (v) => {
        if (v && v.deleted_at) {
          account = null;
        } else {
          account = v || null;
          if (account && !editing) editName = account.name;
        }
      }
    });
    const subEnt = liveQuery(() =>
      db.account_entries
        .where('account_id')
        .equals(id)
        .filter((e) => !e.deleted_at)
        .reverse()
        .sortBy('date')
    ).subscribe({ next: (v) => (entries = v) });
    const subCat = liveQuery(() =>
      db.entry_categories.filter((c) => !c.deleted_at).toArray()
    ).subscribe({ next: (v) => (categories = v) });

    return () => {
      subAcc.unsubscribe();
      subEnt.unsubscribe();
      subCat.unsubscribe();
    };
  });

  let balance = $derived(entries.reduce((sum, e) => sum + e.amount, 0));

  async function handleDelete() {
    if (!account) return;
    if (confirm('Are you sure you want to delete this account?')) {
      await db.accounts.update(id, {
        deleted_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      });
      goto('/accounts');
    }
  }

  async function handleSaveEdit() {
    if (!account || !editName.trim()) return;
    await db.accounts.update(id, {
      name: editName,
      updated_at: new Date().toISOString()
    });
    editing = false;
  }

  function getCategoryColor(catId: number) {
    return categories.find((c) => c.id === catId)?.color || '#ccc';
  }
  function getCategoryName(catId: number) {
    return categories.find((c) => c.id === catId)?.name || 'Unknown';
  }
</script>

{#if !account}
  <div class="text-muted-foreground p-12 text-center font-bold tracking-widest uppercase">
    Account not found or deleted.
    <br /><br />
    <a href="/accounts" class="btn btn-outline inline-block">Back to Accounts</a>
  </div>
{:else}
  <div class="space-y-8 font-sans">
    <div class="border-border/50 flex items-center gap-4 border-b pb-4">
      <a href="/accounts" class="btn-icon">
        <ArrowLeft size={20} />
      </a>

      {#if editing}
        <form
          onsubmit={(e) => {
            e.preventDefault();
            handleSaveEdit();
          }}
          class="flex flex-1 items-center gap-2"
        >
          <input
            type="text"
            bind:value={editName}
            class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary flex-1 rounded-lg border px-3 py-2 text-xl font-semibold focus:ring-2 focus:outline-none"
            autofocus
          />
          <button
            type="submit"
            class="btn-icon text-emerald-500 hover:bg-emerald-500/10 hover:text-emerald-600"
          >
            <Check size={20} />
          </button>
          <button
            type="button"
            onclick={() => {
              editing = false;
              editName = account!.name;
            }}
            class="btn-icon text-destructive hover:bg-destructive/10 hover:text-destructive"
          >
            <X size={20} />
          </button>
        </form>
      {:else}
        <h1 class="flex-1 text-2xl font-semibold tracking-tight">{account.name}</h1>
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

    <div class="bg-card border-border/50 rounded-xl border p-6 shadow-sm">
      <h2 class="text-muted-foreground mb-1 text-sm font-medium">Current Balance</h2>
      <p
        class="text-4xl font-bold tracking-tight {balance >= 0
          ? 'text-emerald-500'
          : 'text-destructive'}"
      >
        {Math.abs(balance).toFixed(2)}
      </p>
    </div>

    <div class="pt-4">
      <div class="border-border/50 mb-4 flex items-center justify-between border-b pb-2">
        <h2 class="text-lg font-semibold tracking-tight">Transactions</h2>
        <button onclick={() => (createEntryModalOpen = true)} class="btn gap-2 text-sm">
          <Plus size={16} /> Add Entry
        </button>
      </div>

      {#if entries.length === 0}
        <div
          class="border-border/60 text-muted-foreground bg-card/50 rounded-xl border border-dashed p-12 text-center text-sm"
        >
          No transactions for this account yet.
        </div>
      {:else}
        <div class="space-y-2">
          {#each entries as entry}
            <a
              href="/entries/{entry.id}"
              class="group bg-card border-border/50 hover:border-border block rounded-lg border p-3.5 shadow-sm transition-all hover:shadow-md"
            >
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-3.5">
                  <div
                    class="h-8 w-2.5 rounded-full"
                    style="background-color: {getCategoryColor(entry.category_id)}"
                  ></div>
                  <div>
                    <p class="text-foreground text-sm font-medium">{entry.title}</p>
                    <div class="text-muted-foreground mt-0.5 flex items-center gap-1.5 text-xs">
                      <span>{getCategoryName(entry.category_id)}</span>
                      {#if entry.location}
                        <span>•</span>
                        <span>{entry.location}</span>
                      {/if}
                    </div>
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
                    {new Date(entry.date || entry.created_at).toLocaleDateString()}
                  </p>
                </div>
              </div>
            </a>
          {/each}
        </div>
      {/if}
    </div>
  </div>

  <Modal bind:open={createEntryModalOpen} title="Create New Entry">
    <EntryForm
      initialAccountId={account.id}
      onsuccess={() => (createEntryModalOpen = false)}
      oncancel={() => (createEntryModalOpen = false)}
    />
  </Modal>
{/if}
