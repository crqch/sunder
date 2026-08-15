<script lang="ts">
  import { db } from '$lib/db';
  import { liveQuery } from 'dexie';
  import type { Account, EntryCategory } from '$lib/types';
  import { Save, Plus, Minus } from '@lucide/svelte';
  import { toast } from 'svelte-sonner';
  import AccountSelect from '$components/AccountSelect.svelte';
  import CategorySelect from '$components/CategorySelect.svelte';

  let {
    onsuccess,
    oncancel,
    initialAccountId = 0,
    isDirty = $bindable(false)
  } = $props<{
    onsuccess: (id: number) => void;
    oncancel?: () => void;
    initialAccountId?: number;
    isDirty?: boolean;
  }>();

  let title = $state('');
  let amountStr = $state('');
  let description = $state('');
  let location = $state('');

  $effect(() => {
    isDirty =
      title.trim() !== '' ||
      amountStr !== '' ||
      description.trim() !== '' ||
      location.trim() !== '';
  });

  let accounts = $state<Account[]>([]);
  let categories = $state<EntryCategory[]>([]);

  let isIncome = $state(false);
  let account_id = $state(0);
  let category_id = $state(0);

  // Format: YYYY-MM-DDTHH:mm for datetime-local input
  let date = $state(new Date().toISOString().slice(0, 16));

  $effect(() => {
    const subAcc = liveQuery(() => db.accounts.filter((a) => !a.deleted_at).toArray()).subscribe({
      next: (v) => {
        accounts = v;
        if (!account_id && v.length > 0) {
          account_id =
            initialAccountId && v.find((a) => a.id === initialAccountId)
              ? initialAccountId
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

  function handleAmountInput(e: Event) {
    const target = e.target as HTMLInputElement;
    let val = target.value;

    if (val.includes('-')) {
      isIncome = false;
      val = val.replace(/-/g, '');
    }
    if (val.includes('+') || val.includes('=')) {
      isIncome = true;
      val = val.replace(/[\+=]/g, '');
    }

    // Only allow numbers and one decimal point
    val = val.replace(/[^0-9.]/g, '');
    const parts = val.split('.');
    if (parts.length > 2) {
      val = parts[0] + '.' + parts.slice(1).join('');
    }

    amountStr = val;
    target.value = val;
  }

  async function saveEntry(e: Event) {
    e.preventDefault();

    let amount = Math.abs(parseFloat(amountStr));
    if (isNaN(amount) || amount === 0) {
      toast.error('Please enter a valid amount.');
      return;
    }
    if (!isIncome) amount = -amount;

    if (!title.trim() || !account_id || !category_id) {
      toast.error('Please fill out all required fields.');
      return;
    }

    const now = new Date().toISOString();
    const entryDate = new Date(date).toISOString();

    const id = Date.now();
    const newEntry = {
      id,
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
    onsuccess(id);
  }
</script>

<form onsubmit={saveEntry} class="space-y-5">
  <div class="flex gap-3">
    <button
      type="button"
      onclick={() => (isIncome = false)}
      class="flex flex-1 items-center justify-center gap-2 rounded-lg border py-3 text-sm font-medium transition-all {!isIncome
        ? 'bg-destructive/10 text-destructive border-destructive/20'
        : 'border-border/50 text-muted-foreground hover:bg-muted bg-transparent'}"
    >
      <Minus size={16} /> Expense
    </button>
    <button
      type="button"
      onclick={() => (isIncome = true)}
      class="flex flex-1 items-center justify-center gap-2 rounded-lg border py-3 text-sm font-medium transition-all {isIncome
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
        type="text"
        inputmode="decimal"
        bind:value={amountStr}
        oninput={handleAmountInput}
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

  <div class="relative z-10 grid grid-cols-1 gap-4 sm:grid-cols-2">
    <div class="space-y-1.5">
      <label class="text-foreground block text-sm font-medium">Account</label>
      <AccountSelect bind:value={account_id} {accounts} />
    </div>

    <div class="space-y-1.5">
      <label class="text-foreground block text-sm font-medium">Category</label>
      <CategorySelect bind:value={category_id} {categories} />
    </div>
  </div>

  <div class="space-y-1.5 pt-2">
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

  <div class="flex gap-3 pt-2">
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
      <Save size={16} /> Create Entry
    </button>
  </div>
</form>
