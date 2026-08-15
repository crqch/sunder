<script lang="ts">
  import { liveQuery } from 'dexie';
  import { db } from '$lib/db';
  import type { Account, AccountEntry, EntryCategory } from '$lib/types';
  import { Plus, Wallet, ArrowRight } from '@lucide/svelte';
  import { modals } from '$lib/modals.svelte';
  import { tooltip } from '$lib/tooltip';

  let accounts = $state<Account[]>([]);
  let entries = $state<AccountEntry[]>([]);
  let categories = $state<EntryCategory[]>([]);

  $effect(() => {
    const subAccounts = liveQuery(() =>
      db.accounts.filter((a) => !a.deleted_at).toArray()
    ).subscribe({ next: (v) => (accounts = v) });
    const subEntries = liveQuery(() =>
      db.account_entries
        .filter((e) => !e.deleted_at)
        .reverse()
        .sortBy('date')
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
  let income = $derived(entries.filter((e) => e.amount > 0).reduce((sum, e) => sum + e.amount, 0));
  let expenses = $derived(
    entries.filter((e) => e.amount < 0).reduce((sum, e) => sum + Math.abs(e.amount), 0)
  );
  let recentEntries = $derived(entries.slice(0, 10));

  let categorySpending = $derived.by(() => {
    let spending: Record<string, number> = {};
    entries.forEach((e) => {
      if (e.amount < 0) {
        spending[e.category_id] = (spending[e.category_id] || 0) + Math.abs(e.amount);
      }
    });
    return Object.entries(spending)
      .map(([id, amount]) => ({
        category: categories.find((c) => c.id === id),
        amount
      }))
      .sort((a, b) => b.amount - a.amount)
      .slice(0, 5);
  });

  function getCategoryColor(id: string) {
    return categories.find((c) => c.id === id)?.color || '#cccccc';
  }
  function getCategoryName(id: string) {
    return categories.find((c) => c.id === id)?.title || 'Unknown';
  }
</script>

<div class="space-y-8 font-sans">
  <div class="flex items-center justify-between">
    <h1 class="text-2xl font-semibold tracking-tight">Dashboard</h1>
  </div>

  <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
    <div
      class="bg-card border-border/50 relative flex flex-col justify-center overflow-hidden rounded-xl border p-6 shadow-sm"
    >
      <div class="bg-primary/10 absolute -top-6 -right-6 h-24 w-24 rounded-full blur-2xl"></div>
      <h2 class="text-muted-foreground mb-1 text-sm font-medium">Total Balance</h2>
      <p
        class="text-3xl font-bold tracking-tight {totalBalance >= 0
          ? 'text-emerald-500'
          : 'text-destructive'}"
      >
        {totalBalance >= 0 ? '' : '-'}{Math.abs(totalBalance).toFixed(2)}
      </p>
    </div>

    <div
      class="bg-card border-border/50 relative flex flex-col justify-center overflow-hidden rounded-xl border p-6 shadow-sm"
    >
      <div class="absolute -top-6 -right-6 h-24 w-24 rounded-full bg-emerald-500/10 blur-2xl"></div>
      <h2 class="text-muted-foreground mb-1 text-sm font-medium">Total Income</h2>
      <p class="text-3xl font-bold tracking-tight text-emerald-500">
        +{income.toFixed(2)}
      </p>
    </div>

    <div
      class="bg-card border-border/50 relative flex flex-col justify-center overflow-hidden rounded-xl border p-6 shadow-sm"
    >
      <div class="bg-destructive/10 absolute -top-6 -right-6 h-24 w-24 rounded-full blur-2xl"></div>
      <h2 class="text-muted-foreground mb-1 text-sm font-medium">Total Expenses</h2>
      <p class="text-destructive text-3xl font-bold tracking-tight">
        -{expenses.toFixed(2)}
      </p>
    </div>
  </div>

  <div class="flex flex-col gap-3 sm:flex-row">
    <button
      use:tooltip={{ text: 'New Entry', keys: ['Alt', 'Space'] }}
      onclick={() => (modals.createEntry = true)}
      class="btn flex-1 gap-2"
    >
      <Plus size={16} /> Add Entry
    </button>
    <button
      use:tooltip={{ text: 'New Account', keys: ['Alt', 'N', 'A'] }}
      onclick={() => (modals.createAccount = true)}
      class="btn btn-outline flex-1 gap-2"
    >
      <Wallet size={16} /> Add Account
    </button>
  </div>

  <div class="grid grid-cols-1 gap-6 pt-4 lg:grid-cols-3">
    <!-- Left Column: Transactions -->
    <div class="lg:col-span-2">
      <div class="border-border/50 mb-4 flex items-center justify-between border-b pb-2">
        <h2 class="text-lg font-semibold tracking-tight">Recent Transactions</h2>
        <a
          href="/entries"
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

    <!-- Right Column: Analytics -->
    <div class="lg:col-span-1">
      <div class="bg-card border-border/50 rounded-xl border p-5 shadow-sm">
        <h2 class="mb-4 text-base font-semibold tracking-tight">Top Spending Categories</h2>
        {#if categorySpending.length === 0}
          <p class="text-muted-foreground text-sm">No expenses yet.</p>
        {:else}
          <div class="space-y-4">
            {#each categorySpending as item}
              <div>
                <div class="mb-1.5 flex justify-between text-sm font-medium">
                  <span class="text-foreground">{item.category?.title || 'Unknown'}</span>
                  <span class="text-muted-foreground">{item.amount.toFixed(2)}</span>
                </div>
                <div class="bg-muted h-2 w-full overflow-hidden rounded-full">
                  <div
                    class="h-full rounded-full transition-all duration-1000"
                    style="width: {(item.amount / expenses) * 100}%; background-color: {item
                      .category?.color || '#ccc'};"
                  ></div>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>
