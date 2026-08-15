<script lang="ts">
  import { liveQuery } from 'dexie';
  import { db } from '$lib/db';
  import type { Account, AccountEntry } from '$lib/types';
  import { Plus, ChevronRight, Trash2, Check } from '@lucide/svelte';
  import { modals } from '$lib/modals.svelte';
  import { tooltip } from '$lib/tooltip';
  import Modal from '$components/Modal.svelte';

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

  function getBalance(accountId: string) {
    return entries.filter((e) => e.account_id === accountId).reduce((sum, e) => sum + e.amount, 0);
  }

  let selectedIds = $state<string[]>([]);
  let allSelected = $derived(
    filteredAccounts.length > 0 && selectedIds.length === filteredAccounts.length
  );
  let deleteModalOpen = $state(false);

  function toggleSelectAll() {
    if (allSelected) {
      selectedIds = [];
    } else {
      selectedIds = filteredAccounts.map((a) => a.id);
    }
  }

  function handleCardClick(e: MouseEvent, id: string) {
    if (selectedIds.length > 0) {
      e.preventDefault();
      if (selectedIds.includes(id)) {
        selectedIds = selectedIds.filter((x) => x !== id);
      } else {
        selectedIds = [...selectedIds, id];
      }
    }
  }

  function confirmDelete() {
    deleteModalOpen = true;
  }

  async function executeDelete() {
    const now = new Date().toISOString();
    await db.accounts.where('id').anyOf(selectedIds).modify({ deleted_at: now, updated_at: now });
    selectedIds = [];
    deleteModalOpen = false;
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
    <div class="flex flex-col gap-3">
      <div class="flex items-center justify-between px-2">
        <label class="group flex cursor-pointer items-center gap-2.5 text-sm font-medium">
          <div
            class="relative flex h-5 w-5 items-center justify-center rounded border-2 transition-all {allSelected
              ? 'bg-primary border-primary text-primary-foreground'
              : 'border-border/60 bg-background group-hover:border-primary/50 text-transparent'}"
          >
            <input
              type="checkbox"
              checked={allSelected}
              onchange={toggleSelectAll}
              class="sr-only"
            />
            <Check size={12} strokeWidth={3} class={allSelected ? 'opacity-100' : 'opacity-0'} />
          </div>
          Select All ({filteredAccounts.length})
        </label>

        {#if selectedIds.length > 0}
          <button
            onclick={confirmDelete}
            class="btn btn-sm bg-destructive text-destructive-foreground hover:bg-destructive/90 h-8 gap-1.5"
          >
            <Trash2 size={14} /> Delete Selected ({selectedIds.length})
          </button>
        {/if}
      </div>

      <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
        {#each filteredAccounts as account}
          {@const balance = getBalance(account.id)}
          <a
            href="/accounts/{account.id}"
            onclick={(e) => handleCardClick(e, account.id)}
            class="bg-card border-border/50 hover:border-border group has-[:checked]:border-primary has-[:checked]:ring-primary/50 has-[:checked]:bg-primary/5 relative block rounded-xl border p-6 shadow-sm transition-all hover:shadow-md has-[:checked]:ring-1"
          >
            <label
              class="absolute top-4 right-4 z-10 -m-2 cursor-pointer p-2"
              onclick={(e) => e.stopPropagation()}
            >
              <input
                type="checkbox"
                value={account.id}
                bind:group={selectedIds}
                class="peer sr-only"
              />
              <div
                class="peer-checked:bg-primary peer-checked:border-primary peer-checked:text-primary-foreground border-border/60 bg-background hover:border-primary/50 flex h-[22px] w-[22px] items-center justify-center rounded-full border-2 text-transparent transition-all peer-checked:scale-110 [&>svg]:opacity-0 peer-checked:[&>svg]:opacity-100"
              >
                <Check size={12} strokeWidth={3} />
              </div>
            </label>
            <div class="mb-4 flex items-start justify-between pr-8">
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
                {Math.abs(balance).toFixed(2)}
              </p>
            </div>
          </a>
        {/each}
      </div>
    </div>
  {/if}
</div>

<Modal bind:open={deleteModalOpen} title="Delete Accounts">
  <div class="space-y-4">
    <p class="text-sm">
      Are you sure you want to delete <strong class="text-foreground font-semibold"
        >{selectedIds.length}</strong
      > accounts?
    </p>
    <div class="flex flex-col gap-2 pt-2 sm:flex-row">
      <button
        onclick={executeDelete}
        class="btn bg-destructive text-destructive-foreground hover:bg-destructive/90 flex-1"
      >
        Delete
      </button>
      <button onclick={() => (deleteModalOpen = false)} class="btn-outline flex-1"> Cancel </button>
    </div>
  </div>
</Modal>
