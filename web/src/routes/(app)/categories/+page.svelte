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
  import type { AccountEntry, EntryCategory } from '$lib/types';
  import { modals } from '$lib/modals.svelte';
  import { currencyStore } from '$lib/currency.svelte';
  import CategoryDot from '$components/CategoryDot.svelte';
  import Modal from '$components/Modal.svelte';
  import EntryForm from '$components/EntryForm.svelte';

  interface CategoryRow extends Record<string, unknown> {
    id: string;
    category: EntryCategory;
    title: string;
    description: string | null;
    color: string;
    entryCount: number;
    netTotal: number;
  }

  let categories = $state<EntryCategory[]>([]);
  let entries = $state<AccountEntry[]>([]);

  $effect(() => {
    const subCat = liveQuery(() =>
      db.entry_categories.filter((c) => !c.deleted_at).toArray()
    ).subscribe({ next: (v) => (categories = v) });
    const subEnt = liveQuery(() =>
      db.account_entries.filter((e) => !e.deleted_at).toArray()
    ).subscribe({ next: (v) => (entries = v) });
    return () => {
      subCat.unsubscribe();
      subEnt.unsubscribe();
    };
  });

  let defaultCurrency = $derived(currencyStore.defaultCurrencyCode);
  let searchQuery = $state('');
  let sort = $state<Array<{ sortKey: string; direction: 'ascending' | 'descending' }>>([
    { sortKey: 'title', direction: 'ascending' }
  ]);

  let categoryRows = $derived.by<CategoryRow[]>(() => {
    const q = searchQuery.trim().toLowerCase();
    const rows = categories
      .filter(
        (c) =>
          !q ||
          c.title.toLowerCase().includes(q) ||
          (c.description && c.description.toLowerCase().includes(q))
      )
      .map((category) => {
        const catEntries = entries.filter((e) => e.category_id === category.id);
        const netTotal = catEntries.reduce(
          (sum, e) => sum + currencyStore.convertEntryToDefault(e),
          0
        );
        return {
          id: category.id,
          category,
          title: category.title,
          description: category.description,
          color: category.color,
          entryCount: catEntries.length,
          netTotal
        };
      });

    const sortKey = sort[0]?.sortKey ?? 'title';
    const dir = sort[0]?.direction ?? 'ascending';
    rows.sort((a, b) => {
      let cmp = 0;
      if (sortKey === 'title') cmp = a.title.localeCompare(b.title);
      else if (sortKey === 'entryCount') cmp = a.entryCount - b.entryCount;
      else if (sortKey === 'netTotal') cmp = a.netTotal - b.netTotal;
      return dir === 'descending' ? -cmp : cmp;
    });

    return rows;
  });

  let selectedKeys = $state<Set<string>>(new Set());
  const selection = useTableSelectionState<CategoryRow>(() => ({
    data: categoryRows,
    idKey: 'id',
    selectedKeys,
    setSelectedKeys: (next) => (selectedKeys = next)
  }));
  const selectionPlugin = useTableSelection<CategoryRow>(() => ({
    ...selection.selectionConfig,
    getRowLabel: (item) => item.title
  }));
  const sortablePlugin = useTableSortable<CategoryRow>(() => ({
    sort,
    onSortChange: (next) => (sort = next)
  }));

  const rowActivationPlugin = $derived<TablePlugin<CategoryRow>>({
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
          goto(`/categories/${item.id}`);
        }
      }
    })
  });

  const plugins = $derived<Record<string, TablePlugin<CategoryRow>>>({
    selection: selectionPlugin,
    sortable: sortablePlugin,
    rowActivation: rowActivationPlugin
  });

  const columns = $derived<TableColumn<CategoryRow>[]>([
    {
      key: 'title',
      header: 'Category',
      width: proportional(2, { minWidth: 220 }),
      sortable: true,
      renderCell: categoryCell
    },
    {
      key: 'entryCount',
      header: 'Entries',
      width: pixel(110),
      align: 'end',
      sortable: true,
      renderCell: countCell
    },
    {
      key: 'netTotal',
      header: `Net (${defaultCurrency})`,
      width: pixel(180),
      align: 'end',
      sortable: true,
      renderCell: totalCell
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
  let quickEntryCategoryId = $state<string | undefined>(undefined);

  function openQuickEntry(categoryId: string) {
    quickEntryCategoryId = categoryId;
    quickEntryModalOpen = true;
  }

  async function executeDelete() {
    const now = new Date().toISOString();
    await db.entry_categories
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
    label="Add category"
    variant="primary"
    icon={addIcon}
    onclick={() => (modals.createCategory = true)}
  />
{/snippet}

{#snippet categoryCell(row: CategoryRow)}
  <HStack gap={2} vAlign="center">
    <CategoryDot color={row.color} />
    <VStack gap={0}>
      <Link
        href={`/categories/${row.id}`}
        hasUnderline
        weight="medium"
        maxLines={1}
        class="sunder-table-title-link"
      >
        {row.title}
      </Link>
      {#if row.description}
        <Text type="supporting" color="secondary" maxLines={1}>{row.description}</Text>
      {/if}
    </VStack>
  </HStack>
{/snippet}

{#snippet countCell(row: CategoryRow)}
  <Text type="supporting" color="secondary" hasTabularNumbers>{row.entryCount}</Text>
{/snippet}

{#snippet totalCell(row: CategoryRow)}
  {#if row.entryCount === 0}
    <Text type="supporting" color="secondary" hasTabularNumbers>—</Text>
  {:else}
    <Text
      weight="semibold"
      hasTabularNumbers
      class={row.netTotal >= 0 ? 'sunder-positive' : 'sunder-negative'}
    >
      {currencyStore.formatAmount(row.netTotal, defaultCurrency, { showPlus: true })}
    </Text>
  {/if}
{/snippet}

{#snippet actionCell(row: CategoryRow)}
  <IconButton
    label="Add entry to {row.title}"
    variant="ghost"
    size="sm"
    icon={addIcon}
    tooltip="Log entry in {row.title}"
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
      <HStack gap={2} vAlign="center">
        <Heading level={1}>Categories</Heading>
        <Badge label={String(categories.length)} />
      </HStack>

      <HStack gap={2} vAlign="center" wrap="wrap">
        <TextInput
          label="Search categories"
          isLabelHidden
          type="text"
          placeholder="Search categories..."
          value={searchQuery}
          onChange={(v) => (searchQuery = v)}
          startIcon={searchIcon}
          hasClear
          width={240}
        />
        <Button
          label="Add category"
          variant="primary"
          icon={addIcon}
          tooltip="New category (Alt+N, C)"
          onclick={() => (modals.createCategory = true)}
        />
      </HStack>
    </HStack>
  </LayoutHeader>
{/snippet}

{#snippet content()}
  <LayoutContent padding={0} label="Categories">
    {#if categories.length === 0}
      <Stack padding={8}>
        <EmptyState
          title="No categories yet"
          description="Create your first category to organize your spending and income."
          actions={emptyActions}
        />
      </Stack>
    {:else if categoryRows.length === 0}
      <Stack padding={8}>
        <EmptyState
          title="No categories match your search"
          description="Try a different search term."
        />
      </Stack>
    {:else}
      <VStack gap={0}>
        {#if selection.selectionState.hasSelection}
          {#snippet selectionLabel(count: number)}
            {count} {count === 1 ? 'category' : 'categories'} selected
          {/snippet}
          <TableSelectionToolbar
            selection={selection.selectionState}
            startContent={bulkActions}
            renderSelectionLabel={selectionLabel}
          />
        {/if}
        <Table
          data={categoryRows}
          {columns}
          idKey="id"
          dividers="rows"
          hasHover
          {plugins}
          rowCount={categoryRows.length}
        />
      </VStack>
    {/if}
  </LayoutContent>
{/snippet}

<Layout height="fill" {header} {content} />

<Modal bind:open={quickEntryModalOpen} title="Add Entry">
  {#if quickEntryCategoryId}
    <EntryForm
      initialCategoryId={quickEntryCategoryId}
      onsuccess={() => (quickEntryModalOpen = false)}
      oncancel={() => (quickEntryModalOpen = false)}
    />
  {/if}
</Modal>

<AlertDialog
  isOpen={deleteModalOpen}
  onOpenChange={(next) => (deleteModalOpen = next)}
  title="Delete {selectedKeys.size} {selectedKeys.size === 1 ? 'category' : 'categories'}?"
  description="The selected categories will be marked as deleted. Entries keep their category reference."
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
