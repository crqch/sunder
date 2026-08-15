<script lang="ts">
  import { liveQuery } from 'dexie';
  import { db } from '$lib/db';
  import type { Account, AccountEntry } from '$lib/types';
  import { Plus, ChevronRight } from '@lucide/svelte';
  import { modals } from '$lib/modals.svelte';
  import { tooltip } from '$lib/tooltip';

  let accounts = $state<Account[]>([]);
  let entries = $state<AccountEntry[]>([]);

  $effect(() => {
    const subAcc = liveQuery(() => db.accounts.filter((a) => !a.deleted_at).toArray()).subscribe({
      next: (v) => (accounts = v)
    });
    const subEnt = liveQuery(() =>
      db.account_entries.filter((e) => !e.deleted_at).toArray()
    ).subscribe({ next: (v) => (entries = v) });
    return () => {
      subAcc.unsubscribe();
      subEnt.unsubscribe();
    };
  });

  let searchQuery = $state('');
  let filteredAccounts = $derived(
    accounts.filter((a) => a.name.toLowerCase().includes(searchQuery.toLowerCase()))
  );

  function getBalance(accountId: number) {
    return entries.filter((e) => e.account_id === accountId).reduce((sum, e) => sum + e.amount, 0);
  }
</script>

<div class="space-y-8 font-sans">
  <div class="border-border/50 flex flex-col gap-4 border-b pb-4">
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-semibold tracking-tight">Accounts</h1>
      <button
        use:tooltip={{ text: 'New Account', keys: ['Alt', 'N', 'A'] }}
        onclick={() => (modals.createAccount = true)}
        class="btn shrink-0 gap-2 text-sm"
      >
        <Plus size={16} /> Add
      </button>
    </div>

    <div class="relative w-full">
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Search accounts..."
        class="input h-10 w-full"
      />
    </div>
  </div>

  {#if accounts.length === 0}
    <div
      class="border-border/60 text-muted-foreground bg-card/50 rounded-xl border border-dashed p-12 text-center text-sm"
    >
      No accounts found. Create your first account!
    </div>
  {:else if filteredAccounts.length === 0}
    <div
      class="border-border/60 text-muted-foreground bg-card/50 rounded-xl border border-dashed p-12 text-center text-sm"
    >
      No accounts match your search.
    </div>
  {:else}
    <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
      {#each filteredAccounts as account}
        {@const balance = getBalance(account.id)}
        <a
          href="/accounts/{account.id}"
          class="bg-card border-border/50 hover:border-border group block rounded-xl border p-6 shadow-sm transition-all hover:shadow-md"
        >
          <div class="mb-4 flex items-start justify-between">
            <h2 class="text-lg font-medium tracking-tight">{account.name}</h2>
            <ChevronRight
              size={20}
              class="text-muted-foreground group-hover:text-foreground transition-colors"
            />
          </div>
          <div>
            <p class="text-muted-foreground mb-1 text-sm font-medium">Balance</p>
            <p
              class="text-3xl font-bold tracking-tight {balance >= 0
                ? 'text-emerald-500'
                : 'text-destructive'}"
            >
              ${Math.abs(balance).toFixed(2)}
            </p>
          </div>
        </a>
      {/each}
    </div>
  {/if}
</div>
