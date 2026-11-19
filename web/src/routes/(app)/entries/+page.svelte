<script lang="ts">
  import { liveQuery } from 'dexie';
  import { db } from '$lib/db';
  import type { AccountEntry, Account, EntryCategory } from '$lib/types';
  import { Plus, Filter, SortDesc, SortAsc, Search, X, Trash2, Check, Coins } from '@lucide/svelte';
  import { modals } from '$lib/modals.svelte';
  import { tooltip } from '$lib/tooltip';
  import { currencyStore, normalizeCurrencyCode } from '$lib/currency.svelte';
  import { toast } from 'svelte-sonner';

  let entries = $state<AccountEntry[]>([]);
  let accounts = $state<Account[]>([]);
  let categories = $state<EntryCategory[]>([]);

  $effect(() => {
    const subEnt = liveQuery(() =>
      db.account_entries.filter((e) => !e.deleted_at).toArray()
    ).subscribe({ next: (v) => (entries = v) });
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

  import MultiSelect from '$components/MultiSelect.svelte';
  import Select from '$components/Select.svelte';
  import CurrencySelect from '$components/CurrencySelect.svelte';
  import DatePicker from '$components/DatePicker.svelte';
  import Modal from '$components/Modal.svelte';

  // Filters state
  let searchQuery = $state('');
  let selectedAccounts = $state<any[]>([]);
  let selectedCategories = $state<any[]>([]);
  let selectedCurrency = $state<string>('');
  let minAmount = $state<number | null>(null);
  let maxAmount = $state<number | null>(null);
  let startDate = $state<Date | null>(null);
  let endDate = $state<Date | null>(null);

  // Sorting state
  let sortField = $state<'date' | 'amount' | 'title'>('date');
  let sortDirection = $state<'desc' | 'asc'>('desc');

  const sortOptions = [
    { value: 'date', label: 'Date' },
    { value: 'amount', label: 'Amount' },
    { value: 'title', label: 'Title' }
  ];

  let showFilters = $state(false);

  let filteredEntries = $derived.by(() => {
    let result = entries.filter((e) => {
      // Search
      if (searchQuery && !e.title.toLowerCase().includes(searchQuery.toLowerCase())) return false;

      // Accounts
      if (selectedAccounts.length > 0 && !selectedAccounts.includes(e.account_id)) return false;

      // Categories
      if (selectedCategories.length > 0 && !selectedCategories.includes(e.category_id))
        return false;

      // Currency
      if (
        selectedCurrency &&
        normalizeCurrencyCode(e.currency_code) !== normalizeCurrencyCode(selectedCurrency)
      ) {
        return false;
      }

      // Amount
      if (minAmount !== null && e.amount < minAmount) return false;
      if (maxAmount !== null && e.amount > maxAmount) return false;

      // Date
      if (startDate || endDate) {
        let entryDate = new Date(e.date || e.created_at);
        entryDate.setHours(0, 0, 0, 0);

        if (startDate) {
          let s = new Date(startDate);
          s.setHours(0, 0, 0, 0);
          if (entryDate < s) return false;
        }
        if (endDate) {
          let ed = new Date(endDate);
          ed.setHours(0, 0, 0, 0);
          if (entryDate > ed) return false;
        }
      }

      return true;
    });

    // Sorting
    result.sort((a, b) => {
      let cmp = 0;
      if (sortField === 'date')
        cmp =
          new Date(a.date || a.created_at).getTime() - new Date(b.date || b.created_at).getTime();
      else if (sortField === 'amount')
        cmp =
          Math.abs(currencyStore.convertEntryToDefault(a)) -
          Math.abs(currencyStore.convertEntryToDefault(b));
      else if (sortField === 'title') cmp = a.title.localeCompare(b.title);

      return sortDirection === 'desc' ? -cmp : cmp;
    });

    return result;
  });

  function getCategoryColor(id: string) {
    return categories.find((c) => c.id === id)?.color || '#cccccc';
  }
  function getCategoryName(id: string) {
    return categories.find((c) => c.id === id)?.title || 'Unknown';
  }
  function getAccountName(id: string) {
    return accounts.find((a) => a.id === id)?.name || 'Unknown';
  }

  function clearFilters() {
    searchQuery = '';
    selectedAccounts = [];
    selectedCategories = [];
    selectedCurrency = '';
    minAmount = null;
    maxAmount = null;
    startDate = null;
    endDate = null;
  }

  let selectedIds = $state<string[]>([]);
  let allSelected = $derived(
    filteredEntries.length > 0 && selectedIds.length === filteredEntries.length
  );
  let deleteModalOpen = $state(false);
  let currencyModalOpen = $state(false);
  let bulkCurrencyCode = $state<string>(currencyStore.defaultCurrencyCode);
  let updatingCurrency = $state(false);

  function toggleSelectAll() {
    if (allSelected) {
      selectedIds = [];
    } else {
      selectedIds = filteredEntries.map((e) => e.id);
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

  function openBulkCurrencyModal() {
    bulkCurrencyCode = currencyStore.defaultCurrencyCode;
    currencyModalOpen = true;
  }

  async function executeBulkCurrencyChange() {
    if (selectedIds.length === 0 || !bulkCurrencyCode) return;
    updatingCurrency = true;
    try {
      const targetCode = normalizeCurrencyCode(bulkCurrencyCode);
      const now = new Date().toISOString();
      const count = selectedIds.length;
      await db.account_entries
        .where('id')
        .anyOf(selectedIds)
        .modify({ currency_code: targetCode, updated_at: now });
      selectedIds = [];
      currencyModalOpen = false;
      toast.success(`Updated ${count} ${count === 1 ? 'entry' : 'entries'} to ${targetCode}`);
      currencyStore.refreshRatesForEntries().catch(() => {});
    } catch (err) {
      console.error('Failed to bulk update currency', err);
      toast.error('Failed to update entry currencies');
    } finally {
      updatingCurrency = false;
    }
  }

  async function executeDelete() {
    const now = new Date().toISOString();
    await db.account_entries
      .where('id')
      .anyOf(selectedIds)
      .modify({ deleted_at: now, updated_at: now });
    selectedIds = [];
    deleteModalOpen = false;
  }
</script>

<div class="space-y-6 font-sans">
  <div class="border-border/50 flex flex-col gap-4 border-b pb-4">
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-semibold tracking-tight">Entries</h1>
      <div class="flex items-center gap-3">
        <button
          onclick={() => (showFilters = !showFilters)}
          class="btn btn-outline shrink-0 gap-2 text-sm {showFilters ? 'bg-muted' : ''}"
        >
          <Filter size={16} /> Filters
        </button>
        <button
          use:tooltip={{ text: 'New Entry', keys: ['Alt', 'N', 'E'] }}
          onclick={() => (modals.createEntry = true)}
          class="btn shrink-0 gap-2 text-sm"
        >
          <Plus size={16} /> Add
        </button>
      </div>
    </div>
    <div class="relative w-full">
      <Search class="text-muted-foreground absolute top-1/2 left-3 -translate-y-1/2" size={16} />
      <input
        type="text"
        bind:value={searchQuery}
        placeholder="Search entries..."
        class="input h-10 w-full pl-9"
      />
    </div>
  </div>

  {#if showFilters}
    <div
      class="bg-card border-border/50 animate-in slide-in-from-top-2 fade-in space-y-6 rounded-xl border p-5 shadow-sm duration-200"
    >
      <div class="mb-2 flex items-center justify-between">
        <h3 class="font-medium">Advanced Filters</h3>
        <button
          onclick={clearFilters}
          class="text-muted-foreground hover:text-foreground flex items-center gap-1 text-sm"
        >
          <X size={14} /> Clear all
        </button>
      </div>

      <div class="space-y-5">
        <div class="grid grid-cols-1 gap-5 md:grid-cols-3">
          <!-- Sort By (Custom Select) -->
          <div class="relative z-30 space-y-2">
            <label class="text-muted-foreground text-sm font-medium">Sort By</label>
            <div class="flex items-center gap-2">
              <div class="flex-1">
                <Select bind:value={sortField} options={sortOptions} />
              </div>
              <button
                onclick={() => (sortDirection = sortDirection === 'desc' ? 'asc' : 'desc')}
                class="btn btn-outline h-[38px] shrink-0 px-2.5"
                title="Toggle sort direction"
              >
                {#if sortDirection === 'desc'}
                  <SortDesc size={16} />
                {:else}
                  <SortAsc size={16} />
                {/if}
              </button>
            </div>
          </div>

          <!-- Date Range (Custom Picker) -->
          <div class="relative z-20 space-y-2">
            <label class="text-muted-foreground text-sm font-medium">Date Range</label>
            <DatePicker bind:startDate bind:endDate />
          </div>

          <!-- Amount Filter -->
          <div class="space-y-2">
            <label class="text-muted-foreground text-sm font-medium">Amount Range</label>
            <div class="flex items-center gap-2">
              <input
                type="number"
                bind:value={minAmount}
                placeholder="Min"
                class="input h-[38px] w-full min-w-0 text-sm"
              />
              <span class="text-muted-foreground">-</span>
              <input
                type="number"
                bind:value={maxAmount}
                placeholder="Max"
                class="input h-[38px] w-full min-w-0 text-sm"
              />
            </div>
          </div>
        </div>

        <div class="grid grid-cols-1 gap-5 md:grid-cols-3">
          <!-- Accounts Filter -->
          <div class="relative z-10 space-y-2">
            <label class="text-muted-foreground text-sm font-medium">Accounts</label>
            <MultiSelect
              bind:value={selectedAccounts}
              options={accounts as any}
              placeholder="All Accounts"
            />
          </div>

          <!-- Categories Filter -->
          <div class="relative space-y-2">
            <label class="text-muted-foreground text-sm font-medium">Categories</label>
            <MultiSelect
              bind:value={selectedCategories}
              options={categories.map((c) => ({ id: c.id, name: c.title, color: c.color })) as any}
              placeholder="All Categories"
            />
          </div>

          <!-- Currency Filter -->
          <div class="relative space-y-2">
            <label class="text-muted-foreground text-sm font-medium">Currency</label>
            <CurrencySelect
              bind:value={selectedCurrency}
              allowEmpty={true}
              emptyLabel="All Currencies"
            />
          </div>
        </div>
      </div>
    </div>
  {/if}

  {#if entries.length === 0}
    <div
      class="border-border/60 text-muted-foreground bg-card/50 rounded-xl border border-dashed p-12 text-center text-sm"
    >
      No entries found. Create your first transaction!
    </div>
  {:else if filteredEntries.length === 0}
    <div
      class="border-border/60 text-muted-foreground bg-card/50 rounded-xl border border-dashed p-12 text-center text-sm"
    >
      No entries match your current filters.
      <button onclick={clearFilters} class="text-primary ml-1 hover:underline">Clear filters</button
      >
    </div>
  {:else}
    <div class="flex flex-col gap-3">
      <div
        class="text-muted-foreground flex flex-wrap items-center justify-between gap-2 px-2 text-xs font-medium"
      >
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
          Select All ({filteredEntries.length})
        </label>

        {#if selectedIds.length > 0}
          <div class="flex flex-wrap items-center gap-2">
            <button onclick={openBulkCurrencyModal} class="btn btn-outline btn-sm h-8 gap-1.5">
              <Coins size={14} /> Change Currency ({selectedIds.length})
            </button>
            <button
              onclick={confirmDelete}
              class="btn btn-sm bg-destructive text-destructive-foreground hover:bg-destructive/90 h-8 gap-1.5"
            >
              <Trash2 size={14} /> Delete Selected ({selectedIds.length})
            </button>
          </div>
        {/if}
      </div>
      {#each filteredEntries as entry (entry.id)}
        {@const entryCode = normalizeCurrencyCode(entry.currency_code)}
        {@const isForeign = entryCode !== currencyStore.defaultCurrencyCode}
        {@const converted = isForeign
          ? currencyStore.tryConvert(
              entry.amount,
              entryCode,
              currencyStore.defaultCurrencyCode,
              entry.date || entry.created_at
            )
          : null}
        <a
          href="/entries/{entry.id}"
          onclick={(e) => handleCardClick(e, entry.id)}
          class="group bg-card border-border/50 hover:border-border has-[:checked]:border-primary has-[:checked]:ring-primary/50 has-[:checked]:bg-primary/5 relative rounded-xl border p-4 shadow-sm transition-all hover:shadow-md has-[:checked]:ring-1"
        >
          <div class="flex items-center justify-between sm:items-start">
            <div class="flex items-center gap-4">
              <label
                class="-m-1 flex cursor-pointer items-center p-1"
                onclick={(e) => e.stopPropagation()}
              >
                <input
                  type="checkbox"
                  value={entry.id}
                  bind:group={selectedIds}
                  class="peer sr-only"
                />
                <div
                  class="peer-checked:bg-primary peer-checked:border-primary peer-checked:text-primary-foreground border-border/60 bg-background hover:border-primary/50 flex h-5 w-5 items-center justify-center rounded-full border-2 text-transparent transition-all peer-checked:scale-110 [&>svg]:opacity-0 peer-checked:[&>svg]:opacity-100"
                >
                  <Check size={12} strokeWidth={3} />
                </div>
              </label>
              <div
                class="h-10 w-3 shrink-0 rounded-full"
                style="background-color: {getCategoryColor(entry.category_id)}"
              ></div>
              <div class="flex flex-col">
                <p class="text-foreground font-semibold tracking-tight">{entry.title}</p>
                <div class="mt-1 flex flex-wrap items-center gap-2">
                  <span class="text-muted-foreground text-xs font-medium"
                    >{getCategoryName(entry.category_id)}</span
                  >
                  <span class="text-border text-xs">•</span>
                  <span class="text-muted-foreground text-xs"
                    >{getAccountName(entry.account_id)}</span
                  >
                  <span class="text-border text-xs">•</span>
                  <span class="text-muted-foreground text-xs"
                    >{new Date(entry.date || entry.created_at).toLocaleDateString()}</span
                  >
                  <span
                    class="bg-muted text-muted-foreground rounded px-1.5 py-0.5 font-mono text-[10px] font-semibold"
                  >
                    {entryCode}
                  </span>
                </div>
              </div>
            </div>
            <div class="flex flex-col items-end">
              <p
                class="text-lg font-bold tracking-tight {entry.amount >= 0
                  ? 'text-emerald-500'
                  : 'text-destructive'}"
              >
                {currencyStore.formatAmount(entry.amount, entryCode, { showPlus: true })}
              </p>
              {#if converted?.converted}
                <p class="text-muted-foreground text-xs">
                  ≈ {currencyStore.formatAmount(
                    converted.amount,
                    currencyStore.defaultCurrencyCode,
                    {
                      showPlus: true
                    }
                  )}
                </p>
              {/if}
            </div>
          </div>
        </a>
      {/each}
    </div>

    <Modal bind:open={currencyModalOpen} title="Change Entry Currency">
      <div class="space-y-4">
        <p class="text-muted-foreground text-sm">
          Change the currency tag for
          <strong class="text-foreground font-semibold">{selectedIds.length}</strong>
          selected {selectedIds.length === 1 ? 'entry' : 'entries'}. Numeric amounts stay as-is
          (useful for reassigning migrated entries).
        </p>

        <div class="space-y-1.5">
          <label class="text-foreground block text-sm font-medium">Target Currency</label>
          <CurrencySelect bind:value={bulkCurrencyCode} />
        </div>

        <div class="flex flex-col gap-2 pt-2 sm:flex-row">
          <button
            onclick={executeBulkCurrencyChange}
            disabled={updatingCurrency || !bulkCurrencyCode}
            class="btn flex-1 disabled:opacity-50"
          >
            {updatingCurrency ? 'Updating...' : `Set to ${bulkCurrencyCode || '...'}`}
          </button>
          <button onclick={() => (currencyModalOpen = false)} class="btn-outline flex-1">
            Cancel
          </button>
        </div>
      </div>
    </Modal>

    <Modal bind:open={deleteModalOpen} title="Delete Entries">
      <div class="space-y-4">
        <p class="text-sm">
          Are you sure you want to delete <strong class="text-foreground font-semibold"
            >{selectedIds.length}</strong
          > entries?
        </p>
        <div class="flex flex-col gap-2 pt-2 sm:flex-row">
          <button
            onclick={executeDelete}
            class="btn bg-destructive text-destructive-foreground hover:bg-destructive/90 flex-1"
          >
            Delete
          </button>
          <button onclick={() => (deleteModalOpen = false)} class="btn-outline flex-1">
            Cancel
          </button>
        </div>
      </div>
    </Modal>
  {/if}
</div>
