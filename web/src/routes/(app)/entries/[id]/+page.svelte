<script lang="ts">
  import { page } from '$app/stores';
  import { liveQuery } from 'dexie';
  import { db } from '$lib/db';
  import type { AccountEntry, Account, EntryCategory } from '$lib/types';
  import { goto } from '$app/navigation';
  import { Trash2, Pencil, Check, X, ArrowLeft } from '@lucide/svelte';
  import { toast } from 'svelte-sonner';
  import AccountSelect from '$components/AccountSelect.svelte';
  import CategorySelect from '$components/CategorySelect.svelte';

  let id = $derived($page.params.id);

  let entry = $state<AccountEntry | null>(null);
  let accounts = $state<Account[]>([]);
  let categories = $state<EntryCategory[]>([]);

  let editing = $state(false);
  let editTitle = $state('');
  let editAmountStr = $state('');
  let editIsIncome = $state(false);
  let editAccountId = $state(0);
  let editCategoryId = $state(0);
  let editDate = $state('');
  let editLocation = $state('');
  let editDescription = $state('');

  $effect(() => {
    const subEnt = liveQuery(() => db.account_entries.get(id)).subscribe({
      next: (v) => {
        if (v && v.deleted_at) {
          entry = null;
        } else {
          entry = v || null;
          if (entry && !editing) {
            editTitle = entry.title;
            editAmountStr = Math.abs(entry.amount).toString();
            editIsIncome = entry.amount >= 0;
            editAccountId = entry.account_id;
            editCategoryId = entry.category_id;
            // Format to datetime-local
            const d = new Date(entry.created_at);
            const localOffset = d.getTimezoneOffset() * 60000;
            const localTime = new Date(d.getTime() - localOffset);
            editDate = localTime.toISOString().slice(0, 16);
            editLocation = entry.location || '';
            editDescription = entry.description || '';
          }
        }
      }
    });
    const subAcc = liveQuery(() => db.accounts.filter((a) => !a.deleted_at).toArray()).subscribe({
      next: (v) => (accounts = v)
    });
    const subCat = liveQuery(() =>
      db.entry_categories.filter((c) => !c.deleted_at).toArray()
    ).subscribe({ next: (v) => (categories = v) });

    return () => {
      subEnt.unsubscribe();
      subAcc.unsubscribe();
      subCat.unsubscribe();
    };
  });

  function handleAmountInput(e: Event) {
    const target = e.target as HTMLInputElement;
    let val = target.value;

    if (val.includes('-')) {
      editIsIncome = false;
      val = val.replace(/-/g, '');
    }
    if (val.includes('+')) {
      editIsIncome = true;
      val = val.replace(/\+/g, '');
    }

    // Only allow numbers and one decimal point
    val = val.replace(/[^0-9.]/g, '');
    const parts = val.split('.');
    if (parts.length > 2) {
      val = parts[0] + '.' + parts.slice(1).join('');
    }

    editAmountStr = val;
    target.value = val;
  }

  async function handleDelete() {
    if (!entry) return;
    if (confirm('Are you sure you want to delete this entry?')) {
      await db.account_entries.update(id, {
        deleted_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      });
      goto(`/accounts/${entry.account_id}`);
    }
  }

  async function handleSaveEdit() {
    if (!entry || !editTitle.trim() || !editAccountId || !editCategoryId) return;

    let amount = Math.abs(parseFloat(editAmountStr));
    if (isNaN(amount) || amount === 0) {
      toast.error('Please enter a valid amount.');
      return;
    }
    if (!editIsIncome) amount = -amount;

    await db.account_entries.update(id, {
      title: editTitle.trim(),
      amount,
      account_id: editAccountId,
      category_id: editCategoryId,
      created_at: new Date(editDate).toISOString(),
      location: editLocation.trim() || null,
      description: editDescription.trim() || null,
      updated_at: new Date().toISOString()
    });
    editing = false;
  }

  function getAccountName(accId: string) {
    return accounts.find((a) => a.id === accId)?.name || 'Unknown';
  }
  function getCategoryName(catId: string) {
    return categories.find((c) => c.id === catId)?.name || 'Unknown';
  }
  function getCategoryColor(catId: string) {
    return categories.find((c) => c.id === catId)?.color || '#cccccc';
  }
</script>

{#if !entry}
  <div class="text-muted-foreground p-12 text-center font-bold tracking-widest uppercase">
    Entry not found or deleted.
    <br /><br />
    <button onclick={() => history.back()} class="btn btn-outline inline-block">Go Back</button>
  </div>
{:else}
  <div class="mx-auto max-w-2xl space-y-6 font-sans">
    <div class="border-border/50 flex items-center gap-4 border-b pb-4">
      <button onclick={() => history.back()} class="btn-icon">
        <ArrowLeft size={20} />
      </button>
      <h1 class="flex-1 text-2xl font-semibold tracking-tight">Entry Details</h1>

      {#if !editing}
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

    {#if editing}
      <div class="bg-card border-border/50 space-y-5 rounded-xl border p-6 shadow-sm">
        <div class="flex gap-3">
          <button
            type="button"
            onclick={() => (editIsIncome = false)}
            class="flex-1 rounded-lg border py-3 text-sm font-medium transition-all {!editIsIncome
              ? 'bg-destructive/10 text-destructive border-destructive/20'
              : 'border-border/50 text-muted-foreground hover:bg-muted bg-transparent'}"
            >Expense</button
          >
          <button
            type="button"
            onclick={() => (editIsIncome = true)}
            class="flex-1 rounded-lg border py-3 text-sm font-medium transition-all {editIsIncome
              ? 'border-emerald-500/20 bg-emerald-500/10 text-emerald-600'
              : 'border-border/50 text-muted-foreground hover:bg-muted bg-transparent'}"
            >Income</button
          >
        </div>

        <div class="space-y-1.5">
          <label class="text-foreground block text-sm font-medium">Amount</label>
          <input
            type="text"
            inputmode="decimal"
            bind:value={editAmountStr}
            oninput={handleAmountInput}
            class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary w-full rounded-lg border p-3 text-xl font-medium focus:ring-2 focus:outline-none"
            placeholder="Amount"
          />
        </div>

        <div class="space-y-1.5">
          <label class="text-foreground block text-sm font-medium">Title</label>
          <input
            type="text"
            bind:value={editTitle}
            class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary w-full rounded-lg border p-2.5 text-sm focus:ring-2 focus:outline-none"
            placeholder="Title"
          />
        </div>

        <div class="relative z-10 grid grid-cols-2 gap-4">
          <div class="space-y-1.5">
            <label class="text-foreground block text-sm font-medium">Account</label>
            <AccountSelect bind:value={editAccountId} {accounts} />
          </div>
          <div class="space-y-1.5">
            <label class="text-foreground block text-sm font-medium">Category</label>
            <CategorySelect bind:value={editCategoryId} {categories} />
          </div>
        </div>

        <div class="space-y-1.5">
          <label class="text-foreground block text-sm font-medium">Date</label>
          <input
            type="datetime-local"
            bind:value={editDate}
            class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary w-full rounded-lg border p-2.5 text-sm focus:ring-2 focus:outline-none"
          />
        </div>

        <div class="space-y-1.5">
          <label class="text-foreground block text-sm font-medium">Location</label>
          <input
            type="text"
            bind:value={editLocation}
            class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary w-full rounded-lg border p-2.5 text-sm focus:ring-2 focus:outline-none"
            placeholder="Location"
          />
        </div>

        <div class="space-y-1.5">
          <label class="text-foreground block text-sm font-medium">Note</label>
          <textarea
            bind:value={editDescription}
            class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary w-full resize-none rounded-lg border p-2.5 text-sm focus:ring-2 focus:outline-none"
            placeholder="Note"></textarea>
        </div>

        <div class="flex gap-4 pt-4">
          <button onclick={handleSaveEdit} class="btn flex-1 gap-2 py-3"
            ><Check size={16} /> Save</button
          >
          <button
            onclick={() => {
              editing = false;
            }}
            class="btn-outline flex flex-1 items-center justify-center gap-2 py-3"
            ><X size={16} /> Cancel</button
          >
        </div>
      </div>
    {:else}
      <div class="bg-card border-border/50 rounded-xl border p-6 shadow-sm">
        <div class="border-border/50 mb-8 border-b pb-8 text-center">
          <p class="text-muted-foreground mb-4 text-sm font-medium">
            {new Date(entry.created_at).toLocaleString()}
          </p>
          <h2 class="mb-2 text-2xl font-semibold tracking-tight">{entry.title}</h2>
          <p
            class="text-5xl font-bold tracking-tight {entry.amount >= 0
              ? 'text-emerald-500'
              : 'text-destructive'}"
          >
            {entry.amount > 0 ? '+' : ''}{entry.amount.toFixed(2)}
          </p>
        </div>

        <div class="space-y-5">
          <div class="flex items-center justify-between text-sm">
            <span class="text-muted-foreground font-medium">Account</span>
            <a
              href="/accounts/{entry.account_id}"
              class="hover:text-primary font-medium transition-colors"
              >{getAccountName(entry.account_id)}</a
            >
          </div>

          <div class="flex items-center justify-between text-sm">
            <span class="text-muted-foreground font-medium">Category</span>
            <a
              href="/categories/{entry.category_id}"
              class="hover:text-primary flex items-center gap-2 font-medium transition-colors"
            >
              <div
                class="h-3 w-3 rounded-full"
                style="background-color: {getCategoryColor(entry.category_id)}"
              ></div>
              {getCategoryName(entry.category_id)}
            </a>
          </div>

          {#if entry.location}
            <div class="flex items-center justify-between text-sm">
              <span class="text-muted-foreground font-medium">Location</span>
              <span class="text-right font-medium">{entry.location}</span>
            </div>
          {/if}

          {#if entry.description}
            <div class="border-border/50 mt-5 border-t pt-5 text-sm">
              <span class="text-muted-foreground mb-2 block font-medium">Note</span>
              <p class="text-foreground font-medium whitespace-pre-wrap">{entry.description}</p>
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </div>
{/if}
