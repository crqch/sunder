<script lang="ts">
  import {
    AlertDialog,
    Badge,
    BreadcrumbItem,
    Breadcrumbs,
    Button,
    Card,
    Divider,
    EmptyState,
    Grid,
    Heading,
    HStack,
    Icon,
    IconButton,
    Layout,
    LayoutContent,
    LayoutHeader,
    List,
    ListItem,
    Text,
    TextInput,
    Token,
    VStack
  } from '@astryx-sv/core';
  import { Check, Pencil, Plus, Trash2, X } from '@lucide/svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { liveQuery } from 'dexie';
  import { db } from '$lib/db';
  import type { Account, AccountEntry, EntryCategory } from '$lib/types';
  import Modal from '$components/Modal.svelte';
  import EntryForm from '$components/EntryForm.svelte';
  import CurrencySelect from '$components/CurrencySelect.svelte';
  import CategoryDot from '$components/CategoryDot.svelte';
  import { currencyStore, normalizeCurrencyCode } from '$lib/currency.svelte';

  let id = $derived($page.params.id || '');

  let account = $state<Account | null>(null);
  let entries = $state<AccountEntry[]>([]);
  let categories = $state<EntryCategory[]>([]);

  let editing = $state(false);
  let editName = $state('');
  let editCurrencyCode = $state('');
  let createEntryModalOpen = $state(false);
  let deleteConfirmOpen = $state(false);

  $effect(() => {
    const subAcc = liveQuery(() => db.accounts.get(id)).subscribe({
      next: (v) => {
        if (v && v.deleted_at) {
          account = null;
        } else {
          account = v || null;
          if (account && !editing) {
            editName = account.name;
            editCurrencyCode = account.currency_code
              ? normalizeCurrencyCode(account.currency_code)
              : '';
          }
        }
      }
    });
    const subEnt = liveQuery(() =>
      db.account_entries
        .where('account_id')
        .equals(id)
        .filter((e) => !e.deleted_at)
        .reverse()
        .sortBy('date')
    ).subscribe({ next: (v) => (entries = v) });
    const subCat = liveQuery(() =>
      db.entry_categories.filter((c) => !c.deleted_at).toArray()
    ).subscribe({ next: (v) => (categories = v) });

    return () => {
      subAcc.unsubscribe();
      subEnt.unsubscribe();
      subCat.unsubscribe();
    };
  });

  let displayCurrency = $derived(
    account?.currency_code
      ? normalizeCurrencyCode(account.currency_code)
      : currencyStore.defaultCurrencyCode
  );

  let balance = $derived(
    entries.reduce(
      (sum, e) =>
        sum +
        currencyStore.tryConvert(e.amount, e.currency_code, displayCurrency, e.date || e.created_at)
          .amount,
      0
    )
  );

  let income = $derived(
    entries
      .filter((e) => e.amount > 0)
      .reduce(
        (sum, e) =>
          sum +
          currencyStore.tryConvert(
            e.amount,
            e.currency_code,
            displayCurrency,
            e.date || e.created_at
          ).amount,
        0
      )
  );

  let expenses = $derived(
    entries
      .filter((e) => e.amount < 0)
      .reduce(
        (sum, e) =>
          sum +
          Math.abs(
            currencyStore.tryConvert(
              e.amount,
              e.currency_code,
              displayCurrency,
              e.date || e.created_at
            ).amount
          ),
        0
      )
  );

  async function handleDelete() {
    if (!account) return;
    const now = new Date().toISOString();
    await db.accounts.update(id, {
      deleted_at: now,
      updated_at: now
    });
    deleteConfirmOpen = false;
    goto('/accounts');
  }

  async function handleSaveEdit(e: SubmitEvent) {
    e.preventDefault();
    if (!account || !editName.trim()) return;
    await db.accounts.update(id, {
      name: editName.trim(),
      currency_code: editCurrencyCode ? normalizeCurrencyCode(editCurrencyCode) : null,
      updated_at: new Date().toISOString()
    });
    editing = false;
  }

  function cancelEdit() {
    editing = false;
    editName = account!.name;
    editCurrencyCode = account!.currency_code ? normalizeCurrencyCode(account!.currency_code) : '';
  }

  function getCategoryColor(catId: string) {
    return categories.find((c) => c.id === catId)?.color || '#71717a';
  }
  function getCategoryName(catId: string) {
    return categories.find((c) => c.id === catId)?.title || 'Unknown';
  }
</script>

{#snippet editIcon()}
  <Icon icon={Pencil} size="sm" />
{/snippet}
{#snippet deleteIcon()}
  <Icon icon={Trash2} size="sm" />
{/snippet}
{#snippet addIcon()}
  <Icon icon={Plus} size="sm" />
{/snippet}
{#snippet saveIcon()}
  <Icon icon={Check} size="sm" />
{/snippet}
{#snippet cancelIcon()}
  <Icon icon={X} size="sm" />
{/snippet}
{#snippet notFoundActions()}
  <Button label="Back to accounts" href="/accounts" />
{/snippet}
{#snippet emptyEntryAction()}
  <Button
    label="Add entry"
    variant="primary"
    icon={addIcon}
    onclick={() => (createEntryModalOpen = true)}
  />
{/snippet}

{#snippet header()}
  <LayoutHeader hasDivider padding={5}>
    <VStack gap={2}>
      <Breadcrumbs variant="supporting">
        <BreadcrumbItem href="/accounts">Accounts</BreadcrumbItem>
        <BreadcrumbItem isCurrent>{account?.name ?? 'Account details'}</BreadcrumbItem>
      </Breadcrumbs>

      {#if account && editing}
        <form onsubmit={handleSaveEdit}>
          <HStack gap={3} vAlign="end" wrap="wrap">
            <TextInput
              label="Account name"
              isLabelHidden
              value={editName}
              onChange={(v) => (editName = v)}
              hasAutoFocus
              width={260}
            />
            <CurrencySelect
              bind:value={editCurrencyCode}
              allowEmpty={true}
              emptyLabel="Default currency"
            />
            <IconButton label="Save changes" type="submit" variant="primary" icon={saveIcon} />
            <IconButton label="Cancel editing" icon={cancelIcon} onclick={cancelEdit} />
          </HStack>
        </form>
      {:else}
        <HStack hAlign="between" vAlign="center" wrap="wrap" gap={3}>
          <HStack gap={2} vAlign="center" wrap="wrap">
            <Heading level={1}>{account?.name ?? 'Account details'}</Heading>
            {#if account}
              <Token label={displayCurrency} size="sm" />
            {/if}
          </HStack>

          {#if account}
            <HStack gap={2} wrap="wrap">
              <Button
                label="Add entry"
                variant="primary"
                size="sm"
                icon={addIcon}
                onclick={() => (createEntryModalOpen = true)}
              />
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
      {/if}
    </VStack>
  </LayoutHeader>
{/snippet}

{#snippet content()}
  <LayoutContent padding={6} label="Account details">
    {#if !account}
      <EmptyState title="Account not found or deleted" actions={notFoundActions} />
    {:else}
      <VStack gap={6}>
        <Grid columns={{ minWidth: 200, max: 3 }} gap={4}>
          <Card padding={5}>
            <VStack gap={1}>
              <Text type="supporting" color="secondary">
                Current balance ({displayCurrency})
              </Text>
              <Heading
                level={2}
                class={balance >= 0
                  ? 'sunder-tabular sunder-positive'
                  : 'sunder-tabular sunder-negative'}
              >
                {currencyStore.formatAmount(balance, displayCurrency)}
              </Heading>
            </VStack>
          </Card>

          <Card padding={5}>
            <VStack gap={1}>
              <Text type="supporting" color="secondary">Total income</Text>
              <Heading level={2} class="sunder-tabular sunder-positive">
                {currencyStore.formatAmount(income, displayCurrency, { showPlus: true })}
              </Heading>
            </VStack>
          </Card>

          <Card padding={5}>
            <VStack gap={1}>
              <Text type="supporting" color="secondary">Total expenses</Text>
              <Heading level={2} class="sunder-tabular sunder-negative">
                {currencyStore.formatAmount(-expenses, displayCurrency)}
              </Heading>
            </VStack>
          </Card>
        </Grid>

        <Card padding={0}>
          <VStack gap={0}>
            <HStack paddingInline={5} paddingBlock={4} hAlign="between" vAlign="center" gap={3}>
              <HStack gap={2} vAlign="center">
                <Heading level={3}>Transactions</Heading>
                <Badge label={String(entries.length)} />
              </HStack>
            </HStack>
            <Divider />

            {#if entries.length === 0}
              <EmptyState
                isCompact
                title="No transactions for this account yet"
                description="Log an income or expense entry to track this account."
                actions={emptyEntryAction}
              />
            {:else}
              <List hasDividers edge="bleed">
                {#each entries as entry (entry.id)}
                  {@const entryCode = normalizeCurrencyCode(entry.currency_code)}
                  {@const isForeign = entryCode !== displayCurrency}
                  {@const converted = isForeign
                    ? currencyStore.tryConvert(
                        entry.amount,
                        entryCode,
                        displayCurrency,
                        entry.date || entry.created_at
                      )
                    : null}
                  {#snippet entryStart()}
                    <CategoryDot color={getCategoryColor(entry.category_id)} />
                  {/snippet}
                  {#snippet entryEnd()}
                    <VStack gap={0} hAlign="end">
                      <Text
                        weight="semibold"
                        hasTabularNumbers
                        class={entry.amount >= 0 ? 'sunder-positive' : 'sunder-negative'}
                      >
                        {currencyStore.formatAmount(entry.amount, entryCode, { showPlus: true })}
                      </Text>
                      {#if converted?.converted}
                        <Text type="supporting" color="secondary" hasTabularNumbers>
                          ≈ {currencyStore.formatAmount(converted.amount, displayCurrency, {
                            showPlus: true
                          })}
                        </Text>
                      {/if}
                      <Text type="supporting" color="secondary" hasTabularNumbers>
                        {new Date(entry.date || entry.created_at).toLocaleDateString()}
                      </Text>
                    </VStack>
                  {/snippet}
                  <ListItem
                    label={entry.title}
                    description="{getCategoryName(entry.category_id)}{entry.location
                      ? ` · ${entry.location}`
                      : ''}"
                    href="/entries/{entry.id}"
                    startContent={entryStart}
                    endContent={entryEnd}
                  />
                {/each}
              </List>
            {/if}
          </VStack>
        </Card>
      </VStack>
    {/if}
  </LayoutContent>
{/snippet}

<Layout height="fill" contentWidth={1040} {header} {content} />

<Modal bind:open={createEntryModalOpen} title="Create New Entry">
  <EntryForm
    initialAccountId={account?.id}
    onsuccess={() => (createEntryModalOpen = false)}
    oncancel={() => (createEntryModalOpen = false)}
  />
</Modal>

<AlertDialog
  isOpen={deleteConfirmOpen}
  onOpenChange={(next) => (deleteConfirmOpen = next)}
  title="Delete {account?.name ?? 'account'}?"
  description="The account will be marked as deleted and disappear from your lists. Its entries stay in the ledger."
  actionLabel="Delete account"
  actionVariant="destructive"
  onAction={handleDelete}
/>
