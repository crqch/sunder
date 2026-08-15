<script lang="ts">
  import { liveQuery } from 'dexie';
  import { db } from '$lib/db';
  import type { Account, AccountEntry } from '$lib/types';
  import { Plus, ChevronRight } from '@lucide/svelte';
  import Modal from '$components/Modal.svelte';
  import AccountForm from '$components/AccountForm.svelte';

  let accounts = $state<Account[]>([]);
  let entries = $state<AccountEntry[]>([]);
  let createModalOpen = $state(false);

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

  function getBalance(accountId: number) {
    return entries.filter((e) => e.account_id === accountId).reduce((sum, e) => sum + e.amount, 0);
  }
</script>

<div class="space-y-8 font-sans">
  <div class="border-border/50 flex items-center justify-between border-b pb-4">
    <h1 class="text-2xl font-semibold tracking-tight">Accounts</h1>
    <button onclick={() => (createModalOpen = true)} class="btn gap-2 text-sm">
      <Plus size={16} /> Add
    </button>
  </div>

  {#if accounts.length === 0}
    <div
      class="border-border/60 text-muted-foreground bg-card/50 rounded-xl border border-dashed p-12 text-center text-sm"
    >
      No accounts found. Create your first account!
    </div>
  {:else}
    <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
      {#each accounts as account}
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

<Modal bind:open={createModalOpen} title="Create New Account">
  <AccountForm
    onsuccess={() => (createModalOpen = false)}
    oncancel={() => (createModalOpen = false)}
  />
</Modal>
