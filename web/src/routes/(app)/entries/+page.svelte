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
    PowerSearch,
    Selector,
    Stack,
    Table,
    TableSelectionToolbar,
    Text,
    Token,
    VStack,
    pixel,
    proportional,
    useTableSelection,
    useTableSelectionState,
    useTableSortable,
    useTableStickyColumns,
    type DateTimeRangePart,
    type EnumItem,
    type FilterValue,
    type PowerSearchConfig,
    type PowerSearchField,
    type PowerSearchFilter,
    type TableColumn,
    type TablePlugin
  } from '@astryx-sv/core';
  import {
    ArrowDownWideNarrow,
    ArrowUpNarrowWide,
    Coins,
    Plus,
    Search,
    Trash2
  } from '@lucide/svelte';
  import { goto } from '$app/navigation';
  import { liveQuery } from 'dexie';
  import { db } from '$lib/db';
  import type { AccountEntry, Account, EntryCategory } from '$lib/types';
  import { modals } from '$lib/modals.svelte';
  import { currencyStore, normalizeCurrencyCode } from '$lib/currency.svelte';
  import { toast } from '$lib/toast.svelte';
  import CurrencySelect from '$components/CurrencySelect.svelte';
  import Modal from '$components/Modal.svelte';
  import CategoryDot from '$components/CategoryDot.svelte';

  type EntryRow = AccountEntry & Record<string, unknown>;

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

  // PowerSearch filters
  let filters = $state<PowerSearchFilter[]>([]);

  let sort = $state<Array<{ sortKey: string; direction: 'ascending' | 'descending' }>>([
    { sortKey: 'date', direction: 'descending' }
  ]);

  const sortOptions = [
    { value: 'date', label: 'Date' },
    { value: 'amount', label: 'Amount' },
    { value: 'title', label: 'Title' }
  ];

  let sortField = $derived(sort[0]?.sortKey ?? 'date');
  let sortDirection = $derived(sort[0]?.direction ?? 'descending');

  function setSortField(field: string | number) {
    sort = [{ sortKey: String(field), direction: sortDirection }];
  }
  function toggleSortDirection() {
    sort = [
      { sortKey: sortField, direction: sortDirection === 'descending' ? 'ascending' : 'descending' }
    ];
  }

  const accountFilterOptions = $derived<EnumItem[]>(
    accounts
      .map((account) => ({ value: account.id, label: account.name }))
      .sort((a, b) => a.label.localeCompare(b.label))
  );

  const categoryFilterOptions = $derived<EnumItem[]>(
    categories
      .map((category) => ({ value: category.id, label: category.title }))
      .sort((a, b) => a.label.localeCompare(b.label))
  );

  const currencyFilterOptions = $derived.by<EnumItem[]>(() => {
    const codes = new Set<string>();
    for (const entry of entries) codes.add(normalizeCurrencyCode(entry.currency_code));
    for (const currency of currencyStore.currencies)
      codes.add(normalizeCurrencyCode(currency.code));
    return [...codes].sort().map((code) => ({ value: code, label: code }));
  });

  function enumFilterField(
    key: string,
    label: string,
    values: EnumItem[],
    description?: string
  ): PowerSearchField {
    return {
      key,
      label,
      ...(description ? { description } : {}),
      defaultOperator: 'is',
      operators: [
        { key: 'is', label: 'is', value: { type: 'enum', values } },
        { key: 'is_not', label: 'is not', value: { type: 'enum', values } },
        { key: 'in', label: 'is any of', value: { type: 'enum_list', values } },
        { key: 'not_in', label: 'is none of', value: { type: 'enum_list', values } }
      ]
    };
  }

  const powerSearchConfig = $derived.by<PowerSearchConfig>(() => ({
    name: 'EntriesSearch',
    contentSearchFieldKey: 'text',
    fields: [
      {
        key: 'text',
        label: 'Text',
        description: 'Search entry title, location, or description',
        typeaheadAliases: ['title', 'description', 'location', 'note'],
        defaultOperator: 'contains',
        operators: [{ key: 'contains', label: 'contains', value: { type: 'string' } }]
      },
      enumFilterField('account', 'Account', accountFilterOptions),
      enumFilterField('category', 'Category', categoryFilterOptions),
      enumFilterField('currency', 'Currency', currencyFilterOptions),
      enumFilterField('type', 'Type', [
        { value: 'income', label: 'Income' },
        { value: 'expense', label: 'Expense' }
      ]),
      {
        key: 'amount',
        label: 'Amount',
        description: 'Compares the absolute value of the entry amount',
        defaultOperator: 'greater_than',
        operators: [
          { key: 'equals', label: '=', value: { type: 'float' } },
          { key: 'not_equals', label: '≠', value: { type: 'float' } },
          { key: 'greater_than', label: '>', value: { type: 'float' } },
          { key: 'greater_than_or_equal', label: '≥', value: { type: 'float' } },
          { key: 'less_than', label: '<', value: { type: 'float' } },
          { key: 'less_than_or_equal', label: '≤', value: { type: 'float' } }
        ]
      },
      {
        key: 'date',
        label: 'Date',
        description: 'Entry date, falling back to when it was created',
        defaultOperator: 'last_30_days',
        operators: [
          { key: 'is_today', label: 'is today', value: { type: 'empty' } },
          { key: 'yesterday', label: 'is yesterday', value: { type: 'empty' } },
          { key: 'last_7_days', label: 'in the last 7 days', value: { type: 'empty' } },
          { key: 'last_30_days', label: 'in the last 30 days', value: { type: 'empty' } },
          { key: 'this_month', label: 'is this month', value: { type: 'empty' } },
          { key: 'last_month', label: 'is last month', value: { type: 'empty' } },
          { key: 'this_year', label: 'is this year', value: { type: 'empty' } },
          {
            key: 'before',
            label: 'is before',
            value: { type: 'date_absolute', isDateOnly: true }
          },
          { key: 'after', label: 'is after', value: { type: 'date_absolute', isDateOnly: true } },
          { key: 'between', label: 'is between', value: { type: 'date_range' } }
        ]
      },
      {
        key: 'location',
        label: 'Location',
        defaultOperator: 'contains',
        operators: [
          { key: 'contains', label: 'contains', value: { type: 'string' } },
          { key: 'is', label: 'is', value: { type: 'string' } },
          { key: 'is_not', label: 'is not', value: { type: 'string' } },
          { key: 'starts_with', label: 'starts with', value: { type: 'string' } },
          { key: 'ends_with', label: 'ends with', value: { type: 'string' } }
        ]
      }
    ]
  }));

  const RELATIVE_UNIT_MS: Record<string, number> = {
    second: 1000,
    minute: 60_000,
    hour: 3_600_000,
    day: 86_400_000,
    week: 604_800_000,
    month: 2_592_000_000,
    year: 31_536_000_000
  };

  function startOfLocalDay(date: Date): Date {
    return new Date(date.getFullYear(), date.getMonth(), date.getDate());
  }

  function addLocalDays(date: Date, days: number): Date {
    const next = new Date(date);
    next.setDate(next.getDate() + days);
    return next;
  }

  function parseEntryDate(value: string): Date {
    return /^\d{4}-\d{2}-\d{2}$/.test(value) ? new Date(`${value}T00:00:00`) : new Date(value);
  }

  function resolveDateRangePart(part: DateTimeRangePart, edge: 'start' | 'end'): number {
    if (part.type === 'ABSOLUTE') {
      if (part.unixSeconds === 0) {
        return edge === 'start' ? Number.NEGATIVE_INFINITY : Number.POSITIVE_INFINITY;
      }
      const date = new Date(part.unixSeconds * 1000);
      const day = new Date(date.getUTCFullYear(), date.getUTCMonth(), date.getUTCDate());
      return edge === 'end' ? addLocalDays(day, 1).getTime() - 1 : day.getTime();
    }
    if (part.type === 'RELATIVE') {
      return Date.now() - part.backValue * (RELATIVE_UNIT_MS[part.unit] ?? 0);
    }
    return Date.now();
  }

  function matchesStringValue(
    actual: string | null | undefined,
    operator: string,
    target: string
  ): boolean {
    const value = (actual ?? '').toLowerCase();
    const expected = target.toLowerCase();
    switch (operator) {
      case 'contains':
        return value.includes(expected);
      case 'not_contains':
        return !value.includes(expected);
      case 'is':
        return value === expected;
      case 'is_not':
        return value !== expected;
      case 'starts_with':
        return value.startsWith(expected);
      case 'not_starts_with':
        return !value.startsWith(expected);
      case 'ends_with':
        return value.endsWith(expected);
      case 'not_ends_with':
        return !value.endsWith(expected);
      default:
        return true;
    }
  }

  function matchesEnumValue(actual: string, operator: string, value: FilterValue): boolean {
    if (value.type === 'enum') {
      if (operator === 'is') return actual === value.value;
      if (operator === 'is_not') return actual !== value.value;
      return true;
    }
    if (value.type === 'enum_list') {
      const included = value.value.includes(actual);
      if (operator === 'in' || operator === 'is_any_of') return included;
      if (operator === 'not_in' || operator === 'is_none_of') return !included;
      return true;
    }
    return true;
  }

  function matchesNumberValue(actual: number, operator: string, target: number): boolean {
    switch (operator) {
      case 'equals':
        return actual === target;
      case 'not_equals':
        return actual !== target;
      case 'greater_than':
        return actual > target;
      case 'greater_than_or_equal':
        return actual >= target;
      case 'less_than':
        return actual < target;
      case 'less_than_or_equal':
        return actual <= target;
      default:
        return true;
    }
  }

  function matchesDateValue(entry: AccountEntry, operator: string, value: FilterValue): boolean {
    const entryDate = parseEntryDate(entry.date || entry.created_at);
    const entryMs = entryDate.getTime();
    if (Number.isNaN(entryMs)) return true;

    if (value.type === 'date_absolute') {
      if (value.unixSeconds === 0) return true;
      const boundary = new Date(value.unixSeconds * 1000);
      const boundaryDay = new Date(
        boundary.getUTCFullYear(),
        boundary.getUTCMonth(),
        boundary.getUTCDate()
      ).getTime();
      const entryDay = startOfLocalDay(entryDate).getTime();
      if (operator === 'before') return entryDay < boundaryDay;
      if (operator === 'after') return entryDay > boundaryDay;
      return true;
    }

    if (value.type === 'date_range') {
      const start = resolveDateRangePart(value.value.start, 'start');
      const end = resolveDateRangePart(value.value.end, 'end');
      return entryMs >= start && entryMs <= end;
    }

    if (value.type === 'empty') {
      const today = startOfLocalDay(new Date());
      const todayStart = today.getTime();
      const tomorrowStart = addLocalDays(today, 1).getTime();
      switch (operator) {
        case 'is_today':
          return entryMs >= todayStart && entryMs < tomorrowStart;
        case 'yesterday':
          return entryMs >= addLocalDays(today, -1).getTime() && entryMs < todayStart;
        case 'last_7_days':
          return entryMs >= addLocalDays(today, -6).getTime() && entryMs < tomorrowStart;
        case 'last_30_days':
          return entryMs >= addLocalDays(today, -29).getTime() && entryMs < tomorrowStart;
        case 'this_month': {
          const start = new Date(today.getFullYear(), today.getMonth(), 1).getTime();
          const end = new Date(today.getFullYear(), today.getMonth() + 1, 1).getTime();
          return entryMs >= start && entryMs < end;
        }
        case 'last_month': {
          const start = new Date(today.getFullYear(), today.getMonth() - 1, 1).getTime();
          const end = new Date(today.getFullYear(), today.getMonth(), 1).getTime();
          return entryMs >= start && entryMs < end;
        }
        case 'this_year': {
          const start = new Date(today.getFullYear(), 0, 1).getTime();
          const end = new Date(today.getFullYear() + 1, 0, 1).getTime();
          return entryMs >= start && entryMs < end;
        }
        default:
          return true;
      }
    }

    return true;
  }

  function matchesPowerSearchFilter(entry: AccountEntry, filter: PowerSearchFilter): boolean {
    const { field, operator, value } = filter;
    switch (field) {
      case 'text': {
        if (value.type !== 'string') return true;
        const query = value.value.trim().toLowerCase();
        if (!query) return true;
        return [entry.title, entry.location, entry.description]
          .filter((part): part is string => typeof part === 'string')
          .some((part) => part.toLowerCase().includes(query));
      }
      case 'account':
        return matchesEnumValue(entry.account_id, operator, value);
      case 'category':
        return matchesEnumValue(entry.category_id, operator, value);
      case 'currency':
        return matchesEnumValue(normalizeCurrencyCode(entry.currency_code), operator, value);
      case 'type':
        return matchesEnumValue(entry.amount >= 0 ? 'income' : 'expense', operator, value);
      case 'amount':
        return value.type === 'float' || value.type === 'integer'
          ? matchesNumberValue(Math.abs(entry.amount), operator, value.value)
          : true;
      case 'date':
        return matchesDateValue(entry, operator, value);
      case 'location':
        return value.type === 'string'
          ? matchesStringValue(entry.location, operator, value.value)
          : true;
      default:
        return true;
    }
  }

  let filteredEntries = $derived.by(() => {
    const result = entries.filter((entry) =>
      filters.every((filter) => matchesPowerSearchFilter(entry, filter))
    );

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

      return sortDirection === 'descending' ? -cmp : cmp;
    });

    return result as EntryRow[];
  });

  function getCategoryColor(id: string) {
    return categories.find((c) => c.id === id)?.color || '#71717a';
  }
  function getCategoryName(id: string) {
    return categories.find((c) => c.id === id)?.title || 'Unknown';
  }
  function getAccountName(id: string) {
    return accounts.find((a) => a.id === id)?.name || 'Unknown';
  }

  function clearFilters() {
    filters = [];
  }

  // Selection
  let selectedKeys = $state<Set<string>>(new Set());
  const selection = useTableSelectionState<EntryRow>(() => ({
    data: filteredEntries,
    idKey: 'id',
    selectedKeys,
    setSelectedKeys: (next) => (selectedKeys = next)
  }));
  const selectionPlugin = useTableSelection<EntryRow>(() => ({
    ...selection.selectionConfig,
    getRowLabel: (item) => item.title
  }));
  const sortablePlugin = useTableSortable<EntryRow>(() => ({
    sort,
    onSortChange: (next) => (sort = next)
  }));
  const stickyColumnsPlugin = useTableStickyColumns<EntryRow>(() => ({
    endKeys: ['amount']
  }));

  const rowActivationPlugin = $derived<TablePlugin<EntryRow>>({
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
          goto(`/entries/${item.id}`);
        }
      }
    })
  });

  const plugins = $derived<Record<string, TablePlugin<EntryRow>>>({
    selection: selectionPlugin,
    sortable: sortablePlugin,
    rowActivation: rowActivationPlugin,
    stickyColumns: stickyColumnsPlugin
  });

  const columns = $derived<TableColumn<EntryRow>[]>([
    {
      key: 'title',
      header: 'Title',
      width: proportional(2, { minWidth: 220 }),
      sortable: true,
      renderCell: titleCell
    },
    {
      key: 'category_id',
      header: 'Category',
      width: proportional(1, { minWidth: 150 }),
      renderCell: categoryCell
    },
    {
      key: 'account_id',
      header: 'Account',
      width: proportional(1, { minWidth: 140 }),
      renderCell: accountCell
    },
    {
      key: 'date',
      header: 'Date',
      width: pixel(120),
      sortable: true,
      renderCell: dateCell
    },
    {
      key: 'currency_code',
      header: 'Currency',
      width: pixel(96),
      renderCell: currencyCell
    },
    {
      key: 'amount',
      header: 'Amount',
      width: pixel(148),
      align: 'end',
      sortable: true,
      renderCell: amountCell
    }
  ]);

  let deleteModalOpen = $state(false);
  let currencyModalOpen = $state(false);
  let bulkCurrencyCode = $state<string>(currencyStore.defaultCurrencyCode);
  let updatingCurrency = $state(false);

  function openBulkCurrencyModal() {
    bulkCurrencyCode = currencyStore.defaultCurrencyCode;
    currencyModalOpen = true;
  }

  async function executeBulkCurrencyChange() {
    if (selectedKeys.size === 0 || !bulkCurrencyCode) return;
    updatingCurrency = true;
    try {
      const targetCode = normalizeCurrencyCode(bulkCurrencyCode);
      const now = new Date().toISOString();
      const count = selectedKeys.size;
      await db.account_entries
        .where('id')
        .anyOf([...selectedKeys])
        .modify({ currency_code: targetCode, updated_at: now });
      selectedKeys = new Set();
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
{#snippet coinsIcon()}
  <Icon icon={Coins} size="sm" />
{/snippet}
{#snippet trashIcon()}
  <Icon icon={Trash2} size="sm" />
{/snippet}
{#snippet sortDescIcon()}
  <Icon icon={ArrowDownWideNarrow} size="sm" />
{/snippet}
{#snippet sortAscIcon()}
  <Icon icon={ArrowUpNarrowWide} size="sm" />
{/snippet}

{#snippet titleCell(entry: EntryRow)}
  <VStack gap={0}>
    <Link
      href={`/entries/${entry.id}`}
      hasUnderline
      weight="medium"
      maxLines={1}
      class="sunder-table-title-link"
    >
      {entry.title}
    </Link>
    {#if entry.location}
      <Text type="supporting" color="secondary" maxLines={1}>{entry.location}</Text>
    {/if}
  </VStack>
{/snippet}

{#snippet categoryCell(entry: EntryRow)}
  <HStack gap={2} vAlign="center">
    <CategoryDot color={getCategoryColor(entry.category_id)} size="sm" />
    <Link
      href={`/categories/${entry.category_id}`}
      color="inherit"
      maxLines={1}
      class="sunder-table-inline-link"
    >
      {getCategoryName(entry.category_id)}
    </Link>
  </HStack>
{/snippet}

{#snippet accountCell(entry: EntryRow)}
  <Link
    href={`/accounts/${entry.account_id}`}
    color="inherit"
    maxLines={1}
    class="sunder-table-inline-link"
  >
    {getAccountName(entry.account_id)}
  </Link>
{/snippet}

{#snippet dateCell(entry: EntryRow)}
  <Text type="supporting" color="secondary" hasTabularNumbers>
    {new Date(entry.date || entry.created_at).toLocaleDateString()}
  </Text>
{/snippet}

{#snippet currencyCell(entry: EntryRow)}
  <Token label={normalizeCurrencyCode(entry.currency_code)} size="sm" />
{/snippet}

{#snippet amountCell(entry: EntryRow)}
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
        ≈ {currencyStore.formatAmount(converted.amount, currencyStore.defaultCurrencyCode, {
          showPlus: true
        })}
      </Text>
    {/if}
  </VStack>
{/snippet}

{#snippet bulkActions()}
  <Button label="Change currency" size="sm" icon={coinsIcon} onclick={openBulkCurrencyModal} />
  <Button
    label="Delete"
    size="sm"
    variant="destructive"
    icon={trashIcon}
    onclick={() => (deleteModalOpen = true)}
  />
{/snippet}

{#snippet emptyActions()}
  <Button
    label="Add entry"
    variant="primary"
    icon={addIcon}
    onclick={() => (modals.createEntry = true)}
  />
{/snippet}

{#snippet clearFiltersActions()}
  <Button label="Clear filters" onclick={clearFilters} />
{/snippet}

{#snippet header()}
  <LayoutHeader hasDivider padding={4}>
    <VStack gap={3}>
      <HStack hAlign="between" vAlign="center" wrap="wrap" gap={3}>
        <HStack gap={2} vAlign="center">
          <Heading level={1}>Entries</Heading>
          <Badge
            label={filteredEntries.length === entries.length
              ? String(entries.length)
              : `${filteredEntries.length} / ${entries.length}`}
          />
        </HStack>

        <HStack gap={2} vAlign="center" wrap="wrap">
          <Selector
            label="Sort entries"
            isLabelHidden
            size="sm"
            variant="secondary"
            width={148}
            options={sortOptions}
            value={sortField}
            onChange={(value: string) => setSortField(value)}
          />
          <IconButton
            label="Toggle sort direction"
            variant="secondary"
            size="sm"
            icon={sortDirection === 'descending' ? sortDescIcon : sortAscIcon}
            onclick={toggleSortDirection}
          />
          <Button
            label="Add entry"
            variant="primary"
            icon={addIcon}
            tooltip="New entry (Alt+N, E)"
            onclick={() => (modals.createEntry = true)}
          />
        </HStack>
      </HStack>

      <PowerSearch
        config={powerSearchConfig}
        {filters}
        onChange={(next) => {
          filters = [...next];
        }}
        label="Search and filter entries"
        placeholder="Search entries or type a filter…"
        startIcon={searchIcon}
        size="sm"
        resultCount={filteredEntries.length}
        maxTokenLength={48}
      />
    </VStack>
  </LayoutHeader>
{/snippet}

{#snippet content()}
  <LayoutContent padding={0} label="Entries">
    {#if entries.length === 0}
      <Stack padding={8}>
        <EmptyState
          title="No entries yet"
          description="Create your first transaction to start tracking your spending."
          actions={emptyActions}
        />
      </Stack>
    {:else if filteredEntries.length === 0}
      <Stack padding={8}>
        <EmptyState
          title="No entries match your filters"
          description="Try broadening your search or clearing the active filters."
          actions={clearFiltersActions}
        />
      </Stack>
    {:else}
      <VStack gap={0}>
        {#if selection.selectionState.hasSelection}
          {#snippet selectionLabel(count: number)}
            {count} {count === 1 ? 'entry' : 'entries'} selected
          {/snippet}
          <TableSelectionToolbar
            selection={selection.selectionState}
            startContent={bulkActions}
            renderSelectionLabel={selectionLabel}
          />
        {/if}
        <Table
          data={filteredEntries}
          {columns}
          idKey="id"
          dividers="rows"
          hasHover
          {plugins}
          rowCount={filteredEntries.length}
        />
      </VStack>
    {/if}
  </LayoutContent>
{/snippet}

<Layout height="fill" {header} {content} />

<Modal bind:open={currencyModalOpen} title="Change Entry Currency">
  <VStack gap={4}>
    <Text type="supporting" color="secondary">
      Change the currency tag for {selectedKeys.size} selected
      {selectedKeys.size === 1 ? 'entry' : 'entries'}. Numeric amounts stay as-is (useful for
      reassigning migrated entries).
    </Text>
    <CurrencySelect bind:value={bulkCurrencyCode} label="Target currency" showLabel />
    <HStack gap={3} hAlign="end">
      <Button label="Cancel" variant="secondary" onclick={() => (currencyModalOpen = false)} />
      <Button
        label={updatingCurrency ? 'Updating...' : `Set to ${bulkCurrencyCode || '...'}`}
        variant="primary"
        isLoading={updatingCurrency}
        isDisabled={!bulkCurrencyCode}
        onclick={executeBulkCurrencyChange}
      />
    </HStack>
  </VStack>
</Modal>

<AlertDialog
  isOpen={deleteModalOpen}
  onOpenChange={(next) => (deleteModalOpen = next)}
  title="Delete {selectedKeys.size} {selectedKeys.size === 1 ? 'entry' : 'entries'}?"
  description="The selected entries will be marked as deleted and removed from your totals. This syncs across your devices."
  actionLabel="Delete"
  actionVariant="destructive"
  onAction={executeDelete}
/>

<style>
  :global(.sunder-table-title-link),
  :global(.sunder-table-inline-link) {
    cursor: pointer;
  }

  :global(.sunder-table-title-link) {
    text-underline-offset: 2px;
    text-decoration-thickness: 1px;
  }

  :global(.sunder-table-title-link:hover) {
    text-decoration-thickness: 2px;
  }
</style>
