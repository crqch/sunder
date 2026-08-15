<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { db } from '$lib/db';
  import { liveQuery } from 'dexie';
  import type { Account, EntryCategory } from '$lib/types';
  import { ArrowLeft, Save, Plus, Minus } from '@lucide/svelte';

  let accounts = $state<Account[]>([]);
  let categories = $state<EntryCategory[]>([]);

  let preselectedAccount = $derived(Number($page.url.searchParams.get('account_id')) || 0);

  let title = $state('');
  let amountStr = $state('');
  let isIncome = $state(false);
  let account_id = $state(0);
  let category_id = $state(0);
  let description = $state('');
  let location = $state('');

  // Format: YYYY-MM-DDTHH:mm for datetime-local input
  let date = $state(new Date().toISOString().slice(0, 16));

  $effect(() => {
    const subAcc = liveQuery(() => db.accounts.filter((a) => !a.deleted_at).toArray()).subscribe({
      next: (v) => {
        accounts = v;
        if (!account_id && v.length > 0) {
          account_id =
            preselectedAccount && v.find((a) => a.id === preselectedAccount)
              ? preselectedAccount
              : v[0].id;
        }
      }
    });
    const subCat = liveQuery(() =>
      db.entry_categories.filter((c) => !c.deleted_at).toArray()
    ).subscribe({
      next: (v) => {
        categories = v;
        if (!category_id && v.length > 0) {
          category_id = v[0].id;
        }
      }
    });
    return () => {
      subAcc.unsubscribe();
      subCat.unsubscribe();
    };
  });

  async function saveEntry(e: Event) {
    e.preventDefault();

    let amount = Math.abs(parseFloat(amountStr));
    if (isNaN(amount) || amount === 0) {
      alert('Please enter a valid amount.');
      return;
    }
    if (!isIncome) amount = -amount;

    if (!title.trim() || !account_id || !category_id) {
      alert('Please fill out all required fields.');
      return;
    }

    const now = new Date().toISOString();
    const entryDate = new Date(date).toISOString();

    const newEntry = {
      id: Date.now(),
      account_id,
      category_id,
      title: title.trim(),
      description: description.trim() || null,
      location: location.trim() || null,
      amount,
      deleted_at: null,
      created_at: entryDate,
      updated_at: now
    };

    await db.account_entries.add(newEntry);
    goto(`/accounts/${account_id}`);
  }
</script>

<div class="mx-auto max-w-2xl space-y-6 font-sans">
  <div class="border-border/50 flex items-center gap-4 border-b pb-4">
    <button
      onclick={() => history.back()}
      class="border-border/50 hover:bg-muted rounded-lg border p-2 transition-colors"
    >
      <ArrowLeft size={20} />
    </button>
    <h1 class="text-2xl font-semibold tracking-tight">New Entry</h1>
  </div>

  <form
    onsubmit={saveEntry}
    class="bg-card border-border/50 space-y-5 rounded-xl border p-6 shadow-sm"
  >
    <div class="flex gap-3">
      <button
        type="button"
        onclick={() => (isIncome = false)}
        class="flex flex-1 items-center justify-center gap-2 rounded-none border py-3 text-sm font-medium transition-all {!isIncome
          ? 'bg-destructive/10 text-destructive border-destructive/20'
          : 'border-border/50 text-muted-foreground hover:bg-muted bg-transparent'}"
      >
        <Minus size={16} /> Expense
      </button>
      <button
        type="button"
        onclick={() => (isIncome = true)}
        class="flex flex-1 items-center justify-center gap-2 rounded-none border py-3 text-sm font-medium transition-all {isIncome
          ? 'border-emerald-500/20 bg-emerald-500/10 text-emerald-600'
          : 'border-border/50 text-muted-foreground hover:bg-muted bg-transparent'}"
      >
        <Plus size={16} /> Income
      </button>
    </div>

    <div class="space-y-1.5">
      <label class="text-foreground block text-sm font-medium">Amount</label>
      <div class="relative">
        <span
          class="text-muted-foreground absolute top-1/2 left-4 -translate-y-1/2 text-xl font-medium"
          >$</span
        >
        <input
          type="number"
          step="0.01"
          min="0"
          bind:value={amountStr}
          class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary w-full rounded-lg border p-3 pl-9 text-2xl font-medium transition-all focus:ring-2 focus:outline-none {isIncome
            ? 'text-emerald-500'
            : 'text-destructive'}"
          placeholder="0.00"
          required
          autofocus
        />
      </div>
    </div>

    <div class="space-y-1.5">
      <label for="title" class="text-foreground block text-sm font-medium">Title</label>
      <input
        id="title"
        type="text"
        bind:value={title}
        class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary w-full rounded-lg border p-2.5 text-sm transition-all focus:ring-2 focus:outline-none"
        placeholder="What was this for?"
        required
      />
    </div>

    <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
      <div class="space-y-1.5">
        <label for="account" class="text-foreground block text-sm font-medium">Account</label>
        <select
          id="account"
          bind:value={account_id}
          class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary w-full cursor-pointer rounded-lg border p-2.5 text-sm focus:ring-2 focus:outline-none"
        >
          {#each accounts as acc}
            <option value={acc.id}>{acc.name}</option>
          {/each}
        </select>
      </div>

      <div class="space-y-1.5">
        <label for="category" class="text-foreground block text-sm font-medium">Category</label>
        <select
          id="category"
          bind:value={category_id}
          class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary w-full cursor-pointer rounded-lg border p-2.5 text-sm focus:ring-2 focus:outline-none"
        >
          {#each categories as cat}
            <option value={cat.id}>{cat.name}</option>
          {/each}
        </select>
      </div>
    </div>

    <div class="space-y-1.5">
      <label for="date" class="text-foreground block text-sm font-medium">Date & Time</label>
      <input
        id="date"
        type="datetime-local"
        bind:value={date}
        class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary w-full rounded-lg border p-2.5 text-sm transition-all focus:ring-2 focus:outline-none"
        required
      />
    </div>

    <div class="space-y-1.5">
      <label for="location" class="text-foreground block text-sm font-medium"
        >Location (Optional)</label
      >
      <input
        id="location"
        type="text"
        bind:value={location}
        class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary w-full rounded-lg border p-2.5 text-sm transition-all focus:ring-2 focus:outline-none"
        placeholder="Where did this happen?"
      />
    </div>

    <div class="space-y-1.5">
      <label for="description" class="text-foreground block text-sm font-medium"
        >Note (Optional)</label
      >
      <textarea
        id="description"
        bind:value={description}
        rows="2"
        class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary w-full resize-none rounded-lg border p-2.5 text-sm transition-all focus:ring-2 focus:outline-none"
        placeholder="Additional details..."></textarea>
    </div>

    <button
      type="submit"
      class="btn mt-4 flex w-full items-center justify-center gap-2 rounded-none py-3 text-sm"
    >
      <Save size={16} /> Save Entry
    </button>
  </form>
</div>
