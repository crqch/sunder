<script lang="ts">
  import { liveQuery } from 'dexie';
  import { db } from '$lib/db';
  import type { Account, AccountEntry, EntryCategory } from '$lib/types';
  import { Plus, Wallet, ArrowRight } from '@lucide/svelte';
  import Modal from '$components/Modal.svelte';
  import EntryForm from '$components/EntryForm.svelte';
  import AccountForm from '$components/AccountForm.svelte';

  let accounts = $state<Account[]>([]);
  let entries = $state<AccountEntry[]>([]);
  let categories = $state<EntryCategory[]>([]);
  let createEntryModalOpen = $state(false);
  let createAccountModalOpen = $state(false);

  $effect(() => {
    const subAccounts = liveQuery(() =>
      db.accounts.filter((a) => !a.deleted_at).toArray()
    ).subscribe({ next: (v) => (accounts = v) });
    const subEntries = liveQuery(() =>
      db.account_entries
        .filter((e) => !e.deleted_at)
        .reverse()
        .sortBy('created_at')
    ).subscribe({ next: (v) => (entries = v) });
    const subCategories = liveQuery(() =>
      db.entry_categories.filter((c) => !c.deleted_at).toArray()
    ).subscribe({ next: (v) => (categories = v) });

    return () => {
      subAccounts.unsubscribe();
      subEntries.unsubscribe();
      subCategories.unsubscribe();
    };
  });

  let totalBalance = $derived(entries.reduce((sum, entry) => sum + entry.amount, 0));
  let recentEntries = $derived(entries.slice(0, 10));

  function getCategoryColor(id: number) {
    return categories.find((c) => c.id === id)?.color || '#cccccc';
  }
  function getCategoryName(id: number) {
    return categories.find((c) => c.id === id)?.name || 'Unknown';
  }
</script>

<div class="space-y-8 font-sans">
  <div class="flex items-center justify-between">
    <h1 class="text-2xl font-semibold tracking-tight">Dashboard</h1>
  </div>

  <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
    <div
      class="bg-card border-border/50 flex flex-col justify-center rounded-xl border p-6 shadow-sm"
    >
      <h2 class="text-muted-foreground mb-1 text-sm font-medium">Total Balance</h2>
      <p
        class="text-4xl font-bold tracking-tight {totalBalance >= 0
          ? 'text-emerald-500'
          : 'text-destructive'}"
      >
        ${Math.abs(totalBalance).toFixed(2)}
      </p>
    </div>

    <div
      class="bg-card border-border/50 flex flex-col justify-center rounded-xl border p-6 shadow-sm"
    >
      <h2 class="text-muted-foreground mb-1 text-sm font-medium">Accounts</h2>
      <p class="text-4xl font-bold tracking-tight">{accounts.length}</p>
    </div>
  </div>

  <div class="flex flex-col gap-3 sm:flex-row">
    <button onclick={() => (createEntryModalOpen = true)} class="btn flex-1 gap-2">
      <Plus size={16} /> Add Entry
    </button>
    <button onclick={() => (createAccountModalOpen = true)} class="btn btn-outline flex-1 gap-2">
      <Wallet size={16} /> Add Account
    </button>
  </div>

  <div class="pt-4">
    <div class="border-border/50 mb-4 flex items-center justify-between border-b pb-2">
      <h2 class="text-lg font-semibold tracking-tight">Recent Transactions</h2>
      <a
        href="/accounts"
        class="text-primary hover:text-primary/80 flex items-center gap-1 text-sm font-medium transition-colors"
        >View All <ArrowRight size={14} /></a
      >
    </div>

    {#if recentEntries.length === 0}
      <div
        class="border-border/60 text-muted-foreground bg-card/50 rounded-xl border border-dashed p-12 text-center text-sm"
      >
        No transactions yet. Start by adding an entry!
      </div>
    {:else}
      <div class="flex flex-col gap-2">
        {#each recentEntries as entry}
          <a
            href="/entries/{entry.id}"
            class="group bg-card border-border/50 hover:border-border rounded-lg border p-3.5 shadow-sm transition-all hover:shadow-md"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3.5">
                <div
                  class="h-8 w-2.5 rounded-full"
                  style="background-color: {getCategoryColor(entry.category_id)}"
                ></div>
                <div>
                  <p class="text-foreground text-sm font-medium">{entry.title}</p>
                  <p class="text-muted-foreground mt-0.5 text-xs">
                    {getCategoryName(entry.category_id)}
                  </p>
                </div>
              </div>
              <p
                class="text-sm font-semibold {entry.amount >= 0
                  ? 'text-emerald-500'
                  : 'text-destructive'}"
              >
                {entry.amount > 0 ? '+' : ''}{entry.amount.toFixed(2)}
              </p>
            </div>
          </a>
        {/each}
      </div>
    {/if}
  </div>
</div>

<Modal bind:open={createEntryModalOpen} title="Create New Entry">
  <EntryForm
    onsuccess={() => (createEntryModalOpen = false)}
    oncancel={() => (createEntryModalOpen = false)}
  />
</Modal>

<Modal bind:open={createAccountModalOpen} title="Create New Account">
  <AccountForm
    onsuccess={() => (createAccountModalOpen = false)}
    oncancel={() => (createAccountModalOpen = false)}
  />
</Modal>
