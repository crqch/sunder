<script lang="ts">
  import {
    AlertDialog,
    BreadcrumbItem,
    Breadcrumbs,
    Button,
    Divider,
    EmptyState,
    FormLayout,
    Grid,
    Heading,
    HStack,
    Icon,
    Layout,
    LayoutContent,
    LayoutHeader,
    Link,
    MetadataList,
    MetadataListItem,
    SegmentedControl,
    SegmentedControlItem,
    Text,
    TextArea,
    TextInput,
    Token,
    VStack
  } from '@astryx-sv/core';
  import { Check, Minus, Pencil, Plus, Trash2, X } from '@lucide/svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { liveQuery } from 'dexie';
  import { db } from '$lib/db';
  import type { AccountEntry, Account, EntryCategory } from '$lib/types';
  import { toast } from '$lib/toast.svelte';
  import AccountSelect from '$components/AccountSelect.svelte';
  import CategorySelect from '$components/CategorySelect.svelte';
  import CurrencySelect from '$components/CurrencySelect.svelte';
  import DatePicker from '$components/DatePicker.svelte';
  import CategoryDot from '$components/CategoryDot.svelte';
  import { currencyStore, normalizeCurrencyCode } from '$lib/currency.svelte';

  let id = $derived($page.params.id || '');

  let entry = $state<AccountEntry | null>(null);
  let accounts = $state<Account[]>([]);
  let categories = $state<EntryCategory[]>([]);

  let editing = $state(false);
  let editTitle = $state('');
  let editAmountStr = $state('');
  let editIsIncome = $state(false);
  let editCurrencyCode = $state(currencyStore.defaultCurrencyCode);
  let editAccountId = $state('');
  let editCategoryId = $state('');
  let editDateObj = $state<Date | null>(null);
  let editLocation = $state('');
  let editDescription = $state('');
  let editAmountError = $state<string | null>(null);
  let editTitleError = $state<string | null>(null);
  let editAmountWrapEl = $state<HTMLElement | null>(null);
  let deleteConfirmOpen = $state(false);

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
            editCurrencyCode = normalizeCurrencyCode(entry.currency_code);
            editAccountId = entry.account_id;
            editCategoryId = entry.category_id;
            editDateObj = new Date(entry.date || entry.created_at);
            editLocation = entry.location || '';
            editDescription = entry.description || '';
            editAmountError = null;
            editTitleError = null;
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

  function handleAmountInput(val: string) {
    if (val.includes('-')) {
      editIsIncome = false;
      val = val.replace(/-/g, '');
    }
    if (val.includes('+') || val.includes('=')) {
      editIsIncome = true;
      val = val.replace(/[+=]/g, '');
    }

    val = val.replace(/[^0-9.]/g, '');
    const parts = val.split('.');
    if (parts.length > 2) {
      val = parts[0] + '.' + parts.slice(1).join('');
    }

    if (val === editAmountStr) {
      const inputEl = editAmountWrapEl?.querySelector('input');
      if (inputEl && inputEl.value !== val) {
        inputEl.value = val;
      }
    }

    editAmountStr = val;
    if (editAmountError) editAmountError = null;
  }

  async function handleDelete() {
    if (!entry) return;
    const now = new Date().toISOString();
    await db.account_entries.update(id, {
      deleted_at: now,
      updated_at: now
    });
    deleteConfirmOpen = false;
    goto(`/accounts/${entry.account_id}`);
  }

  async function handleSaveEdit(e: SubmitEvent) {
    e.preventDefault();
    if (!entry) return;

    let amount = Math.abs(parseFloat(editAmountStr));
    let hasError = false;
    if (isNaN(amount) || amount === 0) {
      editAmountError = 'Enter a valid amount greater than 0.';
      hasError = true;
    } else {
      editAmountError = null;
    }

    if (!editTitle.trim()) {
      editTitleError = 'Enter a title for this entry.';
      hasError = true;
    } else {
      editTitleError = null;
    }

    if (hasError) {
      toast.error(editAmountError ?? editTitleError ?? 'Please fill out all required fields.');
      return;
    }

    if (!editAccountId || !editCategoryId) {
      toast.error('Please select an account and category.');
      return;
    }

    if (!editIsIncome) amount = -amount;

    const normalizedCurrency = normalizeCurrencyCode(editCurrencyCode);

    await db.account_entries.update(id, {
      title: editTitle.trim(),
      amount,
      currency_code: normalizedCurrency,
      account_id: editAccountId,
      category_id: editCategoryId,
      date: editDateObj ? editDateObj.toISOString() : new Date().toISOString(),
      location: editLocation.trim() || null,
      description: editDescription.trim() || null,
      updated_at: new Date().toISOString()
    });
    if (normalizedCurrency !== currencyStore.defaultCurrencyCode) {
      currencyStore.refreshRatesForEntries().catch(() => {});
    }
    editing = false;
  }

  function getAccountName(accId: string) {
    return accounts.find((a) => a.id === accId)?.name || 'Unknown';
  }
  function getCategoryName(catId: string) {
    return categories.find((c) => c.id === catId)?.title || 'Unknown';
  }
  function getCategoryColor(catId: string) {
    return categories.find((c) => c.id === catId)?.color || '#71717a';
  }

  let entryCurrency = $derived(normalizeCurrencyCode(entry?.currency_code));
  let convertedInfo = $derived.by(() => {
    if (!entry) return null;
    if (entryCurrency === currencyStore.defaultCurrencyCode) return null;
    return currencyStore.tryConvert(
      entry.amount,
      entryCurrency,
      currencyStore.defaultCurrencyCode,
      entry.date || entry.created_at
    );
  });
</script>

{#snippet editIcon()}
  <Icon icon={Pencil} size="sm" />
{/snippet}
{#snippet deleteIcon()}
  <Icon icon={Trash2} size="sm" />
{/snippet}
{#snippet saveIcon()}
  <Icon icon={Check} size="sm" />
{/snippet}
{#snippet cancelIcon()}
  <Icon icon={X} size="sm" />
{/snippet}
{#snippet expenseIcon()}
  <Icon icon={Minus} size="sm" />
{/snippet}
{#snippet incomeIcon()}
  <Icon icon={Plus} size="sm" />
{/snippet}
{#snippet notFoundActions()}
  <Button label="Back to entries" href="/entries" />
{/snippet}

{#snippet header()}
  <LayoutHeader hasDivider padding={5}>
    <VStack gap={2}>
      <Breadcrumbs variant="supporting">
        <BreadcrumbItem href="/entries">Entries</BreadcrumbItem>
        <BreadcrumbItem isCurrent>{entry?.title ?? 'Entry details'}</BreadcrumbItem>
      </Breadcrumbs>

      <HStack hAlign="between" vAlign="center" wrap="wrap" gap={3}>
        <HStack gap={2} vAlign="center" wrap="wrap">
          <Heading level={1}>{editing ? 'Edit entry' : (entry?.title ?? 'Entry details')}</Heading>
          {#if entry && !editing}
            <Token label={entryCurrency} size="sm" />
          {/if}
        </HStack>

        {#if entry && !editing}
          <HStack gap={2}>
            <Button
              label="Edit"
              variant="secondary"
              size="sm"
              icon={editIcon}
              onclick={() => (editing = true)}
            />
            <Button
              label="Delete"
              variant="destructive"
              size="sm"
              icon={deleteIcon}
              onclick={() => (deleteConfirmOpen = true)}
            />
          </HStack>
        {/if}
      </HStack>
    </VStack>
  </LayoutHeader>
{/snippet}

{#snippet content()}
  <LayoutContent padding={6} label="Entry details">
    {#if !entry}
      <EmptyState title="Entry not found or deleted" actions={notFoundActions} />
    {:else if editing}
      <form onsubmit={handleSaveEdit} novalidate>
        <FormLayout defaultOptionality="optional">
          <SegmentedControl
            label="Entry type"
            value={editIsIncome ? 'income' : 'expense'}
            onChange={(v) => (editIsIncome = v === 'income')}
            layout="fill"
          >
            <SegmentedControlItem value="expense" label="Expense" icon={expenseIcon} />
            <SegmentedControlItem value="income" label="Income" icon={incomeIcon} />
          </SegmentedControl>

          <!-- Matched md heights + responsive grid (identical to EntryForm). -->
          <div class="sunder-amount-currency-grid">
            <div bind:this={editAmountWrapEl} class="sunder-amount-cell">
              <TextInput
                label="Amount"
                size="md"
                placeholder="0.00"
                value={editAmountStr}
                onChange={handleAmountInput}
                status={editAmountError ? { type: 'error', message: editAmountError } : undefined}
                controlClassName={editIsIncome
                  ? 'sunder-tabular sunder-positive'
                  : 'sunder-tabular sunder-negative'}
                isRequired
                hasAutoFocus
              />
            </div>
            <div class="sunder-currency-cell">
              <CurrencySelect bind:value={editCurrencyCode} showLabel isRequired size="md" />
            </div>
          </div>

          <TextInput
            label="Title"
            value={editTitle}
            onChange={(v) => {
              editTitle = v;
              if (editTitleError && v.trim()) editTitleError = null;
            }}
            status={editTitleError ? { type: 'error', message: editTitleError } : undefined}
            isRequired
          />

          <Grid columns={{ minWidth: 200, max: 2 }} gap={4}>
            <AccountSelect bind:value={editAccountId} {accounts} showLabel isRequired />
            <CategorySelect bind:value={editCategoryId} {categories} showLabel isRequired />
          </Grid>

          <Grid columns={{ minWidth: 200, max: 2 }} gap={4}>
            <DatePicker bind:startDate={editDateObj} mode="single" label="Date" showLabel />
            <TextInput
              label="Location"
              placeholder="Where did this happen?"
              value={editLocation}
              onChange={(v) => (editLocation = v)}
            />
          </Grid>

          <TextArea
            label="Note"
            rows={3}
            value={editDescription}
            onChange={(v) => (editDescription = v)}
          />

          <HStack gap={3} hAlign="end" paddingBlockStart={2}>
            <Button
              label="Cancel"
              variant="secondary"
              icon={cancelIcon}
              onclick={() => (editing = false)}
            />
            <Button label="Save changes" type="submit" variant="primary" icon={saveIcon} />
          </HStack>
        </FormLayout>
      </form>
    {:else}
      <VStack gap={6}>
        <VStack gap={1}>
          <Text type="supporting" color="secondary">
            {entry.amount >= 0 ? 'Income' : 'Expense'} · {new Date(
              entry.date || entry.created_at
            ).toLocaleDateString()}
          </Text>
          <Heading
            level={2}
            type="display-2"
            class={entry.amount >= 0
              ? 'sunder-tabular sunder-positive'
              : 'sunder-tabular sunder-negative'}
          >
            {currencyStore.formatAmount(entry.amount, entryCurrency, { showPlus: true })}
          </Heading>
          {#if convertedInfo?.converted}
            <Text type="supporting" color="secondary" hasTabularNumbers>
              ≈ {currencyStore.formatAmount(
                convertedInfo.amount,
                currencyStore.defaultCurrencyCode,
                { showPlus: true }
              )}
              {#if convertedInfo.rate}
                (1 {entryCurrency} = {convertedInfo.rate.toFixed(4)}
                {currencyStore.defaultCurrencyCode})
              {/if}
            </Text>
          {/if}
        </VStack>

        <Divider />

        <MetadataList columns="multi">
          <MetadataListItem label="Account">
            <Link href="/accounts/{entry.account_id}" type="inherit">
              {getAccountName(entry.account_id)}
            </Link>
          </MetadataListItem>
          <MetadataListItem label="Category">
            <Link href="/categories/{entry.category_id}" type="inherit">
              <HStack gap={2} vAlign="center">
                <CategoryDot color={getCategoryColor(entry.category_id)} size="sm" />
                {getCategoryName(entry.category_id)}
              </HStack>
            </Link>
          </MetadataListItem>
          <MetadataListItem label="Currency">
            <Text type="code">
              {entryCurrency} · {currencyStore.getCurrency(entryCurrency).name}
            </Text>
          </MetadataListItem>
          <MetadataListItem label="Date">
            <Text hasTabularNumbers>
              {new Date(entry.date || entry.created_at).toLocaleDateString()}
            </Text>
          </MetadataListItem>
          {#if entry.location}
            <MetadataListItem label="Location">
              <Text>{entry.location}</Text>
            </MetadataListItem>
          {/if}
        </MetadataList>

        {#if entry.description}
          <Divider />
          <VStack gap={1.5}>
            <Text type="label" color="secondary" weight="medium">Note</Text>
            <Text class="sunder-pre-wrap">{entry.description}</Text>
          </VStack>
        {/if}
      </VStack>
    {/if}
  </LayoutContent>
{/snippet}

<Layout height="fill" contentWidth={760} {header} {content} />

<AlertDialog
  isOpen={deleteConfirmOpen}
  onOpenChange={(next) => (deleteConfirmOpen = next)}
  title="Delete this entry?"
  description="“{entry?.title ?? ''}” will be marked as deleted and removed from your totals."
  actionLabel="Delete entry"
  actionVariant="destructive"
  onAction={handleDelete}
/>

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
