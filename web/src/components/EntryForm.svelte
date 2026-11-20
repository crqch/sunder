<script lang="ts">
  import { getContext, hasContext, untrack } from 'svelte';
  import {
    Button,
    Divider,
    FormLayout,
    Grid,
    HStack,
    Icon,
    SegmentedControl,
    SegmentedControlItem,
    TextArea,
    TextInput
  } from '@astryx-sv/core';
  import { Minus, Plus, Save } from '@lucide/svelte';
  import { liveQuery } from 'dexie';
  import { createId } from '@paralleldrive/cuid2';
  import { db } from '$lib/db';
  import type { Account, EntryCategory } from '$lib/types';
  import { toast } from '$lib/toast.svelte';
  import AccountSelect from '$components/AccountSelect.svelte';
  import CategorySelect from '$components/CategorySelect.svelte';
  import CurrencySelect from '$components/CurrencySelect.svelte';
  import DatePicker from '$components/DatePicker.svelte';
  import { MODAL_CTX, type ModalContext } from '$components/Modal.svelte';
  import { currencyStore, normalizeCurrencyCode } from '$lib/currency.svelte';

  let {
    onsuccess,
    oncancel,
    initialAccountId,
    initialCategoryId,
    isDirty = $bindable(false)
  }: {
    onsuccess: (id: string) => void;
    oncancel?: () => void;
    initialAccountId?: string;
    initialCategoryId?: string;
    isDirty?: boolean;
  } = $props();

  const formId = `sunder-entry-form-${createId()}`;
  const modalCtx = hasContext(MODAL_CTX) ? getContext<ModalContext>(MODAL_CTX) : null;

  let title = $state('');
  let amountStr = $state('');
  let description = $state('');
  let location = $state('');
  let amountError = $state<string | null>(null);
  let titleError = $state<string | null>(null);
  let amountWrapEl = $state<HTMLElement | null>(null);

  $effect(() => {
    isDirty =
      title.trim() !== '' ||
      amountStr !== '' ||
      description.trim() !== '' ||
      location.trim() !== '';
  });

  // Register actions into Modal's sticky LayoutFooter when hosted inside Modal
  // so Cancel / Create entry stay pinned even on short viewports.
  $effect(() => {
    if (!modalCtx) return;
    modalCtx.registerFooter(formActions);
    return () => {
      modalCtx.registerFooter(null);
    };
  });

  let accounts = $state<Account[]>([]);
  let categories = $state<EntryCategory[]>([]);

  let isIncome = $state(false);
  let account_id = $state<string>(untrack(() => initialAccountId || ''));
  let category_id = $state<string>(untrack(() => initialCategoryId || ''));
  let currency_code = $state<string>(currencyStore.defaultCurrencyCode);
  let currencyManuallyChosen = $state(false);

  let entryDateObj = $state<Date | null>(new Date());

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
          category_id =
            initialCategoryId && v.find((c) => c.id === initialCategoryId)
              ? initialCategoryId
              : v[0].id;
        }
      }
    });
    return () => {
      subAcc.unsubscribe();
      subCat.unsubscribe();
    };
  });

  $effect(() => {
    if (!currencyManuallyChosen && account_id) {
      const selectedAcc = accounts.find((a) => a.id === account_id);
      if (selectedAcc?.currency_code) {
        currency_code = normalizeCurrencyCode(selectedAcc.currency_code);
      } else {
        currency_code = currencyStore.defaultCurrencyCode;
      }
    }
  });

  // Typing `-` flips to expense, `+`/`=` to income; keep digits and one dot.
  function handleAmountInput(val: string) {
    if (val.includes('-')) {
      isIncome = false;
      val = val.replace(/-/g, '');
    }
    if (val.includes('+') || val.includes('=')) {
      isIncome = true;
      val = val.replace(/[+=]/g, '');
    }

    val = val.replace(/[^0-9.]/g, '');
    const parts = val.split('.');
    if (parts.length > 2) {
      val = parts[0] + '.' + parts.slice(1).join('');
    }

    // If stripping characters leaves amountStr unchanged, explicitly sync the
    // DOM input value so rejected characters never remain visible in the field.
    if (val === amountStr) {
      const inputEl = amountWrapEl?.querySelector('input');
      if (inputEl && inputEl.value !== val) {
        inputEl.value = val;
      }
    }

    amountStr = val;
    if (amountError) amountError = null;
  }

  async function saveEntry(e: SubmitEvent) {
    e.preventDefault();

    let amount = Math.abs(parseFloat(amountStr));
    let hasError = false;
    if (isNaN(amount) || amount === 0) {
      amountError = 'Enter a valid amount greater than 0.';
      hasError = true;
    } else {
      amountError = null;
    }

    if (!title.trim()) {
      titleError = 'Enter a title for this entry.';
      hasError = true;
    } else {
      titleError = null;
    }

    if (hasError) {
      toast.error(amountError ?? titleError ?? 'Please fill out all required fields.');
      return;
    }

    if (!account_id || !category_id) {
      toast.error('Please select an account and category.');
      return;
    }

    if (!isIncome) amount = -amount;

    const now = new Date().toISOString();
    const transactionDate = entryDateObj ? entryDateObj.toISOString() : new Date().toISOString();
    const id = createId();
    const normalizedCurrency = normalizeCurrencyCode(currency_code);
    const newEntry = {
      id,
      account_id,
      category_id,
      title: title.trim(),
      description: description.trim() || null,
      location: location.trim() || null,
      amount,
      currency_code: normalizedCurrency,
      date: transactionDate,
      deleted_at: null,
      created_at: now,
      updated_at: now
    };

    await db.account_entries.add(newEntry);
    if (normalizedCurrency !== currencyStore.defaultCurrencyCode) {
      currencyStore.refreshRatesForEntries().catch(() => {});
    }
    onsuccess(id);
  }
</script>

{#snippet expenseIcon()}
  <Icon icon={Minus} size="sm" />
{/snippet}
{#snippet incomeIcon()}
  <Icon icon={Plus} size="sm" />
{/snippet}
{#snippet saveIcon()}
  <Icon icon={Save} size="sm" />
{/snippet}

{#snippet formActions()}
  <HStack gap={3} hAlign="end" width="100%">
    {#if oncancel}
      <Button label="Cancel" variant="secondary" onclick={oncancel} />
    {/if}
    <Button label="Create entry" type="submit" form={formId} variant="primary" icon={saveIcon} />
  </HStack>
{/snippet}

<form id={formId} onsubmit={saveEntry} novalidate>
  <FormLayout defaultOptionality="optional">
    <SegmentedControl
      label="Entry type"
      value={isIncome ? 'income' : 'expense'}
      onChange={(v) => (isIncome = v === 'income')}
      layout="fill"
    >
      <SegmentedControlItem value="expense" label="Expense" icon={expenseIcon} />
      <SegmentedControlItem value="income" label="Income" icon={incomeIcon} />
    </SegmentedControl>

    <!-- Amount + Currency hero row: responsive grid keeps both md controls
         aligned on desktop and lets Currency expand full-width on mobile. -->
    <div class="sunder-amount-currency-grid">
      <div bind:this={amountWrapEl} data-modal-autofocus class="sunder-amount-cell">
        <TextInput
          label="Amount"
          size="md"
          placeholder="0.00"
          value={amountStr}
          onChange={handleAmountInput}
          status={amountError ? { type: 'error', message: amountError } : undefined}
          controlClassName={isIncome
            ? 'sunder-tabular sunder-positive'
            : 'sunder-tabular sunder-negative'}
          isRequired
          hasAutoFocus
        />
      </div>
      <div class="sunder-currency-cell">
        <CurrencySelect
          bind:value={currency_code}
          showLabel
          isRequired
          size="md"
          onchange={() => (currencyManuallyChosen = true)}
        />
      </div>
    </div>

    <TextInput
      label="Title"
      placeholder="What was this for?"
      value={title}
      onChange={(v) => {
        title = v;
        if (titleError && v.trim()) titleError = null;
      }}
      status={titleError ? { type: 'error', message: titleError } : undefined}
      isRequired
    />

    <Grid columns={{ minWidth: 200, max: 2 }} gap={4}>
      <AccountSelect bind:value={account_id} {accounts} showLabel isRequired />
      <CategorySelect bind:value={category_id} {categories} showLabel isRequired />
    </Grid>

    <Grid columns={{ minWidth: 200, max: 2 }} gap={4}>
      <DatePicker bind:startDate={entryDateObj} mode="single" label="Date" showLabel />
      <TextInput
        label="Location"
        placeholder="Where did this happen?"
        value={location}
        onChange={(v) => (location = v)}
      />
    </Grid>

    <TextArea
      label="Note"
      rows={2}
      placeholder="Additional details..."
      value={description}
      onChange={(v) => (description = v)}
    />

    {#if !modalCtx}
      <Divider />
      {@render formActions()}
    {/if}
  </FormLayout>
</form>

<style>
  .sunder-amount-currency-grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(176px, 220px);
    gap: var(--spacing-4);
    align-items: start;
    width: 100%;
  }

  .sunder-amount-cell,
  .sunder-currency-cell {
    min-width: 0;
  }

  @media (max-width: 480px) {
    .sunder-amount-currency-grid {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
