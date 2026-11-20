<script lang="ts">
  import {
    AlertDialog,
    Badge,
    Button,
    EmptyState,
    Heading,
    HStack,
    Icon,
    IconButton,
    Layout,
    LayoutContent,
    LayoutHeader,
    Link,
    Stack,
    Table,
    TableSelectionToolbar,
    Text,
    TextInput,
    Token,
    VStack,
    pixel,
    proportional,
    useTableSelection,
    useTableSelectionState,
    useTableSortable,
    type TableColumn,
    type TablePlugin
  } from '@astryx-sv/core';
  import { Plus, Search, Trash2 } from '@lucide/svelte';
  import { goto } from '$app/navigation';
  import { liveQuery } from 'dexie';
  import { db } from '$lib/db';
  import type { Account, AccountEntry } from '$lib/types';
  import { modals } from '$lib/modals.svelte';
  import { currencyStore, normalizeCurrencyCode } from '$lib/currency.svelte';
  import Modal from '$components/Modal.svelte';
  import EntryForm from '$components/EntryForm.svelte';

  interface AccountRow extends Record<string, unknown> {
    id: string;
    account: Account;
    name: string;
    displayCurrency: string;
    hasExplicitCurrency: boolean;
    entryCount: number;
    balance: number;
    defaultCurrencyBalance: number;
  }

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

  let defaultCurrency = $derived(currencyStore.defaultCurrencyCode);
  let searchQuery = $state('');
  let sort = $state<Array<{ sortKey: string; direction: 'ascending' | 'descending' }>>([
    { sortKey: 'name', direction: 'ascending' }
  ]);

  let accountRows = $derived.by<AccountRow[]>(() => {
    const q = searchQuery.trim().toLowerCase();
    const rows = accounts
      .filter((a) => !q || a.name.toLowerCase().includes(q))
      .map((account) => {
        const hasExplicitCurrency = !!account.currency_code;
        const displayCurrency = hasExplicitCurrency
          ? normalizeCurrencyCode(account.currency_code)
          : defaultCurrency;
        const accEntries = entries.filter((e) => e.account_id === account.id);
        const balance = accEntries.reduce(
          (sum, e) =>
            sum +
            currencyStore.tryConvert(
              e.amount,
              e.currency_code,
              displayCurrency,
              e.date || e.created_at
            ).amount,
          0
        );
        const defaultCurrencyBalance = accEntries.reduce(
          (sum, e) => sum + currencyStore.convertEntryToDefault(e),
          0
        );
        return {
          id: account.id,
          account,
          name: account.name,
          displayCurrency,
          hasExplicitCurrency,
          entryCount: accEntries.length,
          balance,
          defaultCurrencyBalance
        };
      });

    const sortKey = sort[0]?.sortKey ?? 'name';
    const dir = sort[0]?.direction ?? 'ascending';
    rows.sort((a, b) => {
      let cmp = 0;
      if (sortKey === 'name') cmp = a.name.localeCompare(b.name);
      else if (sortKey === 'entryCount') cmp = a.entryCount - b.entryCount;
      else if (sortKey === 'balance') cmp = a.defaultCurrencyBalance - b.defaultCurrencyBalance;
      return dir === 'descending' ? -cmp : cmp;
    });

    return rows;
  });

  let totalNetWorth = $derived(
    accountRows.reduce((sum, row) => sum + row.defaultCurrencyBalance, 0)
  );

  let selectedKeys = $state<Set<string>>(new Set());
  const selection = useTableSelectionState<AccountRow>(() => ({
    data: accountRows,
    idKey: 'id',
    selectedKeys,
    setSelectedKeys: (next) => (selectedKeys = next)
  }));
  const selectionPlugin = useTableSelection<AccountRow>(() => ({
    ...selection.selectionConfig,
    getRowLabel: (item) => item.name
  }));
  const sortablePlugin = useTableSortable<AccountRow>(() => ({
    sort,
    onSortChange: (next) => (sort = next)
  }));

  const rowActivationPlugin = $derived<TablePlugin<AccountRow>>({
    transformBodyRow: (props, item) => ({
      ...props,
      htmlProps: {
        ...props.htmlProps,
        style: `${props.htmlProps.style ? `${props.htmlProps.style};` : ''}cursor: pointer;`,
        onclick: (event: MouseEvent) => {
          if ((event.target as HTMLElement).closest('input, button, a, select, textarea') != null) {
            return;
          }
          if (selectedKeys.size > 0) {
            const next = new Set(selectedKeys);
            if (next.has(item.id)) next.delete(item.id);
            else next.add(item.id);
            selectedKeys = next;
            return;
          }
          goto(`/accounts/${item.id}`);
        }
      }
    })
  });

  const plugins = $derived<Record<string, TablePlugin<AccountRow>>>({
    selection: selectionPlugin,
    sortable: sortablePlugin,
    rowActivation: rowActivationPlugin
  });

  const columns = $derived<TableColumn<AccountRow>[]>([
    {
      key: 'name',
      header: 'Account',
      width: proportional(2, { minWidth: 200 }),
      sortable: true,
      renderCell: nameCell
    },
    {
      key: 'displayCurrency',
      header: 'Currency',
      width: pixel(130),
      renderCell: currencyCell
    },
    {
      key: 'entryCount',
      header: 'Entries',
      width: pixel(100),
      align: 'end',
      sortable: true,
      renderCell: countCell
    },
    {
      key: 'balance',
      header: 'Balance',
      width: pixel(180),
      align: 'end',
      sortable: true,
      renderCell: balanceCell
    },
    {
      key: 'actions',
      header: '',
      width: pixel(60),
      align: 'end',
      renderCell: actionCell
    }
  ]);

  let deleteModalOpen = $state(false);
  let quickEntryModalOpen = $state(false);
  let quickEntryAccountId = $state<string | undefined>(undefined);

  function openQuickEntry(accountId: string) {
    quickEntryAccountId = accountId;
    quickEntryModalOpen = true;
  }

  async function executeDelete() {
    const now = new Date().toISOString();
    await db.accounts
      .where('id')
      .anyOf([...selectedKeys])
      .modify({ deleted_at: now, updated_at: now });
    selectedKeys = new Set();
    deleteModalOpen = false;
  }
</script>

{#snippet searchIcon()}
  <Icon icon={Search} size="sm" color="secondary" />
{/snippet}
{#snippet addIcon()}
  <Icon icon={Plus} size="sm" />
{/snippet}
{#snippet trashIcon()}
  <Icon icon={Trash2} size="sm" />
{/snippet}
{#snippet emptyActions()}
  <Button
    label="Add account"
    variant="primary"
    icon={addIcon}
    onclick={() => (modals.createAccount = true)}
  />
{/snippet}

{#snippet nameCell(row: AccountRow)}
  <Link
    href={`/accounts/${row.id}`}
    hasUnderline
    weight="medium"
    maxLines={1}
    class="sunder-table-title-link"
  >
    {row.name}
  </Link>
{/snippet}

{#snippet currencyCell(row: AccountRow)}
  <HStack gap={1.5} vAlign="center">
    <Token label={row.displayCurrency} size="sm" />
    {#if !row.hasExplicitCurrency}
      <Text type="supporting" color="secondary">Default</Text>
    {/if}
  </HStack>
{/snippet}

{#snippet countCell(row: AccountRow)}
  <Text type="supporting" color="secondary" hasTabularNumbers>{row.entryCount}</Text>
{/snippet}

{#snippet balanceCell(row: AccountRow)}
  <VStack gap={0} hAlign="end">
    <Text
      weight="semibold"
      hasTabularNumbers
      class={row.balance >= 0 ? 'sunder-positive' : 'sunder-negative'}
    >
      {currencyStore.formatAmount(row.balance, row.displayCurrency)}
    </Text>
    {#if row.displayCurrency !== defaultCurrency}
      <Text type="supporting" color="secondary" hasTabularNumbers>
        ≈ {currencyStore.formatAmount(row.defaultCurrencyBalance, defaultCurrency)}
      </Text>
    {/if}
  </VStack>
{/snippet}

{#snippet actionCell(row: AccountRow)}
  <IconButton
    label="Add entry to {row.name}"
    variant="ghost"
    size="sm"
    icon={addIcon}
    tooltip="Log entry in {row.name}"
    onclick={() => openQuickEntry(row.id)}
  />
{/snippet}

{#snippet bulkActions()}
  <Button
    label="Delete"
    size="sm"
    variant="destructive"
    icon={trashIcon}
    onclick={() => (deleteModalOpen = true)}
  />
{/snippet}

{#snippet header()}
  <LayoutHeader hasDivider padding={4}>
    <HStack hAlign="between" vAlign="center" wrap="wrap" gap={3}>
      <VStack gap={0.5}>
        <HStack gap={2} vAlign="center">
          <Heading level={1}>Accounts</Heading>
          <Badge label={String(accounts.length)} />
        </HStack>
        {#if accounts.length > 0}
          <HStack gap={1.5} vAlign="center">
            <Text type="supporting" color="secondary">Total across shown accounts:</Text>
            <Text
              type="supporting"
              weight="semibold"
              hasTabularNumbers
              class={totalNetWorth >= 0 ? 'sunder-positive' : 'sunder-negative'}
            >
              {currencyStore.formatAmount(totalNetWorth, defaultCurrency)}
            </Text>
          </HStack>
        {/if}
      </VStack>

      <HStack gap={2} vAlign="center" wrap="wrap">
        <TextInput
          label="Search accounts"
          isLabelHidden
          type="text"
          placeholder="Search accounts..."
          value={searchQuery}
          onChange={(v) => (searchQuery = v)}
          startIcon={searchIcon}
          hasClear
          width={240}
        />
        <Button
          label="Add account"
          variant="primary"
          icon={addIcon}
          tooltip="New account (Alt+N, A)"
          onclick={() => (modals.createAccount = true)}
        />
      </HStack>
    </HStack>
  </LayoutHeader>
{/snippet}

{#snippet content()}
  <LayoutContent padding={0} label="Accounts">
    {#if accounts.length === 0}
      <Stack padding={8}>
        <EmptyState
          title="No accounts yet"
          description="Create your first account to start tracking balances."
          actions={emptyActions}
        />
      </Stack>
    {:else if accountRows.length === 0}
      <Stack padding={8}>
        <EmptyState
          title="No accounts match your search"
          description="Try a different search term."
        />
      </Stack>
    {:else}
      <VStack gap={0}>
        {#if selection.selectionState.hasSelection}
          {#snippet selectionLabel(count: number)}
            {count} {count === 1 ? 'account' : 'accounts'} selected
          {/snippet}
          <TableSelectionToolbar
            selection={selection.selectionState}
            startContent={bulkActions}
            renderSelectionLabel={selectionLabel}
          />
        {/if}
        <Table
          data={accountRows}
          {columns}
          idKey="id"
          dividers="rows"
          hasHover
          {plugins}
          rowCount={accountRows.length}
        />
      </VStack>
    {/if}
  </LayoutContent>
{/snippet}

<Layout height="fill" {header} {content} />

<Modal bind:open={quickEntryModalOpen} title="Add Entry">
  {#if quickEntryAccountId}
    <EntryForm
      initialAccountId={quickEntryAccountId}
      onsuccess={() => (quickEntryModalOpen = false)}
      oncancel={() => (quickEntryModalOpen = false)}
    />
  {/if}
</Modal>

<AlertDialog
  isOpen={deleteModalOpen}
  onOpenChange={(next) => (deleteModalOpen = next)}
  title="Delete {selectedKeys.size} {selectedKeys.size === 1 ? 'account' : 'accounts'}?"
  description="The selected accounts will be marked as deleted. Their entries stay in the ledger but the accounts disappear from your lists."
  actionLabel="Delete"
  actionVariant="destructive"
  onAction={executeDelete}
/>

<style>
  :global(.sunder-table-title-link) {
    cursor: pointer;
    text-underline-offset: 2px;
    text-decoration-thickness: 1px;
  }

  :global(.sunder-table-title-link:hover) {
    text-decoration-thickness: 2px;
  }
</style>
