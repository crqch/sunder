<script lang="ts">
  import {
    AlertDialog,
    Badge,
    Button,
    Card,
    CheckboxInput,
    DateInput,
    EmptyState,
    FormLayout,
    Grid,
    Heading,
    HStack,
    Icon,
    IconButton,
    Layout,
    LayoutContent,
    LayoutHeader,
    NumberInput,
    Section,
    SegmentedControl,
    SegmentedControlItem,
    Spinner,
    Stack,
    StackItem,
    Switch,
    Tab,
    Table,
    TabList,
    Text,
    TextInput,
    Token,
    VStack,
    pixel,
    proportional,
    type ISODateString,
    type TableColumn
  } from '@astryx-sv/core';
  import { Eye, EyeOff, Pencil, Plus, RefreshCw, Search, Trash2 } from '@lucide/svelte';
  import { onMount } from 'svelte';
  import { authenticatedFetch } from '$lib/auth';
  import { currencyStore } from '$lib/currency.svelte';
  import type { Currency } from '$lib/types';
  import { toast } from '$lib/toast.svelte';
  import Modal from '$components/Modal.svelte';

  type AdminTab = 'users' | 'invites' | 'currencies' | 'rates';

  let activeTab = $state<AdminTab>('users');

  let users = $state<any[]>([]);
  let invites = $state<any[]>([]);
  let currencies = $state<Currency[]>([]);
  let loading = $state(true);

  let editingUser = $state<any>(null);
  let editUsername = $state('');
  let editEmail = $state('');
  let editFlags = $state('');
  let editPassword = $state('');
  let editModalOpen = $state(false);
  let showPassword = $state(false);

  let newInviteToken = $state('');

  // Currency admin state
  let currencySearch = $state('');
  let currencyFilter = $state<'all' | 'active' | 'inactive'>('all');
  let currencyModalOpen = $state(false);
  let editingCurrencyCode = $state<string | null>(null);
  let currCode = $state('');
  let currName = $state('');
  let currSymbol = $state('');
  let currSymbolAfter = $state(false);
  let currDecimalDigits = $state(2);
  let currIsActive = $state(true);
  let savingCurrency = $state(false);

  // Rate sync state
  let syncingRates = $state(false);
  let syncMode = $state<'latest' | 'date' | 'days'>('latest');
  let syncDate = $state(new Date().toISOString().slice(0, 10));
  let syncDays = $state(30);
  let syncForce = $state(false);

  // Manual rate override state
  let manualBase = $state('USD');
  let manualTarget = $state('EUR');
  let manualRate = $state('');
  let manualDate = $state(new Date().toISOString().slice(0, 10));
  let savingManualRate = $state(false);

  // Destructive confirmations
  let userToDelete = $state<any>(null);
  let inviteToDelete = $state<any>(null);

  let filteredCurrencies = $derived.by(() => {
    const q = currencySearch.trim().toLowerCase();
    return currencies.filter((c) => {
      if (currencyFilter === 'active' && !c.is_active) return false;
      if (currencyFilter === 'inactive' && c.is_active) return false;
      if (!q) return true;
      return (
        c.code.toLowerCase().includes(q) ||
        c.name.toLowerCase().includes(q) ||
        c.symbol.toLowerCase().includes(q)
      );
    });
  });

  let activeCurrencyCount = $derived(currencies.filter((c) => c.is_active).length);

  async function loadData() {
    try {
      const results = await Promise.allSettled([
        authenticatedFetch('/admin/users'),
        authenticatedFetch('/admin/invites'),
        authenticatedFetch('/admin/currencies')
      ]);

      const parseJson = async (res: PromiseSettledResult<Response>) => {
        if (res.status !== 'fulfilled' || !res.value.ok) return null;
        try {
          return (await res.value.json()) as any;
        } catch {
          return null;
        }
      };

      const [uData, iData, cData] = await Promise.all([
        parseJson(results[0]),
        parseJson(results[1]),
        parseJson(results[2])
      ]);

      if (uData) users = uData.users || [];
      if (iData) invites = iData.invites || [];
      if (cData) currencies = cData.currencies || [];

      if (!uData && !iData && !cData) {
        toast.error('Failed to load admin data');
      } else if (!uData || !iData || !cData) {
        toast.warning('Some admin sections failed to load');
      }
    } catch (e) {
      toast.error('Failed to load admin data');
    } finally {
      loading = false;
    }
  }

  onMount(loadData);

  async function deleteUser() {
    if (!userToDelete) return;
    const res = await authenticatedFetch(`/admin/users/${userToDelete.id}`, { method: 'DELETE' });
    if (res.ok) {
      toast.success('User deleted');
      users = users.filter((u) => u.id !== userToDelete.id);
    } else {
      toast.error('Failed to delete user');
    }
    userToDelete = null;
  }

  async function saveUser(e?: SubmitEvent) {
    e?.preventDefault();
    if (!editingUser) return;

    const flagsArray = editFlags
      .split(',')
      .map((f) => f.trim())
      .filter((f) => f);

    const payload: any = {
      username: editUsername,
      email: editEmail,
      flags: flagsArray
    };

    if (editPassword) {
      payload.pass = editPassword;
    }

    const res = await authenticatedFetch(`/admin/users/${editingUser.id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    if (res.ok) {
      const data = (await res.json()) as any;
      toast.success('User updated');
      users = users.map((u) => (u.id === editingUser.id ? data.user : u));
      editModalOpen = false;
    } else {
      toast.error('Failed to update user');
    }
  }

  async function createInvite(e?: SubmitEvent) {
    e?.preventDefault();
    if (!newInviteToken.trim()) return;
    const res = await authenticatedFetch(`/admin/invites`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ token: newInviteToken.trim() })
    });
    if (res.ok) {
      toast.success('Invite created');
      const data = (await res.json()) as any;
      invites = [...invites, data];
      newInviteToken = '';
    } else {
      toast.error('Failed to create invite');
    }
  }

  async function deleteInvite() {
    if (!inviteToDelete) return;
    const res = await authenticatedFetch(`/admin/invites/${inviteToDelete.id}`, {
      method: 'DELETE'
    });
    if (res.ok) {
      toast.success('Invite deleted');
      invites = invites.filter((i) => i.id !== inviteToDelete.id);
    } else {
      toast.error('Failed to delete invite');
    }
    inviteToDelete = null;
  }

  function openCreateCurrencyModal() {
    editingCurrencyCode = null;
    currCode = '';
    currName = '';
    currSymbol = '';
    currSymbolAfter = false;
    currDecimalDigits = 2;
    currIsActive = true;
    currencyModalOpen = true;
  }

  function openEditCurrencyModal(c: Currency) {
    editingCurrencyCode = c.code;
    currCode = c.code;
    currName = c.name;
    currSymbol = c.symbol;
    currSymbolAfter = Boolean(c.symbol_after);
    currDecimalDigits = c.decimal_digits ?? 2;
    currIsActive = c.is_active ?? true;
    currencyModalOpen = true;
  }

  async function saveCurrency(e?: SubmitEvent) {
    e?.preventDefault();
    const code = currCode.trim().toUpperCase();
    if (!code || !currName.trim() || !currSymbol.trim()) {
      toast.error('Code, name, and symbol are required');
      return;
    }
    savingCurrency = true;
    try {
      const isEdit = editingCurrencyCode !== null;
      const url = isEdit
        ? `/admin/currencies/${encodeURIComponent(editingCurrencyCode!)}`
        : '/admin/currencies';
      const payload = isEdit
        ? {
            name: currName.trim(),
            symbol: currSymbol.trim(),
            symbol_after: currSymbolAfter,
            decimal_digits: Number(currDecimalDigits),
            is_active: currIsActive
          }
        : {
            code,
            name: currName.trim(),
            symbol: currSymbol.trim(),
            symbol_after: currSymbolAfter,
            decimal_digits: Number(currDecimalDigits),
            is_active: currIsActive
          };

      const res = await authenticatedFetch(url, {
        method: isEdit ? 'PUT' : 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
      });
      const data = (await res.json()) as any;
      if (!res.ok) {
        toast.error(data.error || 'Failed to save currency');
        return;
      }

      if (isEdit) {
        currencies = currencies.map((c) => (c.code === editingCurrencyCode ? data.currency : c));
        toast.success(`Updated ${code}`);
      } else {
        currencies = [...currencies, data.currency].sort((a, b) => a.code.localeCompare(b.code));
        toast.success(`Created ${code}`);
      }
      currencyModalOpen = false;
      currencyStore.refreshCurrencies().catch(() => {});
    } catch (err) {
      toast.error('Failed to save currency');
    } finally {
      savingCurrency = false;
    }
  }

  async function toggleCurrencyActive(c: Currency) {
    const nextActive = !c.is_active;
    const res = await authenticatedFetch(`/admin/currencies/${encodeURIComponent(c.code)}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ is_active: nextActive })
    });
    if (res.ok) {
      const data = (await res.json()) as any;
      currencies = currencies.map((item) => (item.code === c.code ? data.currency : item));
      toast.success(`${c.code} ${nextActive ? 'enabled' : 'disabled'}`);
      currencyStore.refreshCurrencies().catch(() => {});
    } else {
      toast.error(`Failed to update ${c.code}`);
    }
  }

  async function triggerRateSync() {
    syncingRates = true;
    try {
      const params = new URLSearchParams();
      if (syncMode === 'date' && syncDate) {
        params.set('date', syncDate);
      } else if (syncMode === 'days') {
        params.set('days', String(Math.max(1, Math.min(366, Number(syncDays) || 30))));
        if (syncForce) params.set('force', 'true');
      }
      const query = params.toString();
      const res = await authenticatedFetch(
        `/admin/currencies/sync-rates${query ? `?${query}` : ''}`,
        { method: 'POST' }
      );
      const data = (await res.json()) as any;
      if (!res.ok) {
        toast.error(data.error || 'Rate sync failed');
        return;
      }
      if (syncMode === 'days') {
        const s = data.stats || {};
        toast.success(
          `Backfill complete (${data.from} to ${data.to}): ${s.synced ?? 0} synced, ${s.skipped ?? 0} skipped, ${s.failed ?? 0} failed`
        );
      } else {
        toast.success(
          `Synced ${data.stored_count ?? 0} rates for ${data.date} (${data.source || 'provider'})`
        );
      }
      currencyStore.refreshRatesForEntries().catch(() => {});
    } catch (err) {
      toast.error('Exchange rate sync failed');
    } finally {
      syncingRates = false;
    }
  }

  async function saveManualRate(e?: SubmitEvent) {
    e?.preventDefault();
    const rateNum = parseFloat(manualRate);
    if (!manualTarget.trim() || isNaN(rateNum) || rateNum <= 0) {
      toast.error('Please enter a valid target currency and positive rate');
      return;
    }
    savingManualRate = true;
    try {
      const res = await authenticatedFetch('/admin/currencies/rates', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          base_currency_code: manualBase.trim().toUpperCase() || 'USD',
          target_currency_code: manualTarget.trim().toUpperCase(),
          rate: rateNum,
          rate_date: manualDate || undefined
        })
      });
      const data = (await res.json()) as any;
      if (!res.ok) {
        toast.error(data.error || 'Failed to save manual rate');
        return;
      }
      toast.success(
        `Saved 1 ${data.rate.base_currency_code} = ${data.rate.rate} ${data.rate.target_currency_code} (${data.rate.rate_date})`
      );
      manualRate = '';
      currencyStore.refreshRatesForEntries().catch(() => {});
    } catch (err) {
      toast.error('Failed to save manual rate');
    } finally {
      savingManualRate = false;
    }
  }

  const userColumns = $derived<TableColumn<any>[]>([
    {
      key: 'username',
      header: 'Username',
      width: proportional(1, { minWidth: 160 }),
      renderCell: userNameCell
    },
    {
      key: 'email',
      header: 'Email',
      width: proportional(1, { minWidth: 200 }),
      renderCell: userEmailCell
    },
    {
      key: 'flags',
      header: 'Flags',
      width: proportional(1, { minWidth: 150 }),
      renderCell: userFlagsCell
    },
    { key: 'id', header: 'ID', width: pixel(180), renderCell: userIdCell },
    {
      key: 'actions',
      header: '',
      width: pixel(100),
      align: 'end',
      renderCell: userActionsCell
    }
  ]);

  const inviteColumns = $derived<TableColumn<any>[]>([
    {
      key: 'token',
      header: 'Token',
      width: proportional(1, { minWidth: 180 }),
      renderCell: inviteTokenCell
    },
    {
      key: 'status',
      header: 'Status',
      width: proportional(1, { minWidth: 160 }),
      renderCell: inviteStatusCell
    },
    { key: 'id', header: 'ID', width: pixel(180), renderCell: inviteIdCell },
    {
      key: 'actions',
      header: '',
      width: pixel(72),
      align: 'end',
      renderCell: inviteActionsCell
    }
  ]);

  const currencyColumns = $derived<TableColumn<Currency>[]>([
    { key: 'code', header: 'Code', width: pixel(96), renderCell: currCodeCell },
    {
      key: 'name',
      header: 'Name',
      width: proportional(1, { minWidth: 180 }),
      renderCell: currNameCell
    },
    { key: 'symbol', header: 'Symbol', width: pixel(150), renderCell: currSymbolCell },
    {
      key: 'decimal_digits',
      header: 'Decimals',
      width: pixel(100),
      align: 'end',
      renderCell: currDecimalsCell
    },
    { key: 'is_active', header: 'Active', width: pixel(90), renderCell: currStatusCell },
    {
      key: 'actions',
      header: '',
      width: pixel(72),
      align: 'end',
      renderCell: currActionsCell
    }
  ]);
</script>

{#snippet plusIcon()}
  <Icon icon={Plus} size="sm" />
{/snippet}
{#snippet editIcon()}
  <Icon icon={Pencil} size="sm" />
{/snippet}
{#snippet trashIcon()}
  <Icon icon={Trash2} size="sm" />
{/snippet}
{#snippet refreshIcon()}
  <Icon icon={RefreshCw} size="sm" />
{/snippet}
{#snippet searchIcon()}
  <Icon icon={Search} size="sm" color="secondary" />
{/snippet}
{#snippet eyeIcon()}
  <Icon icon={Eye} size="sm" />
{/snippet}
{#snippet eyeOffIcon()}
  <Icon icon={EyeOff} size="sm" />
{/snippet}

{#snippet usersCountBadge()}
  {#if !loading}
    <Badge label={String(users.length)} />
  {/if}
{/snippet}
{#snippet invitesCountBadge()}
  {#if !loading}
    <Badge label={String(invites.length)} />
  {/if}
{/snippet}
{#snippet currenciesCountBadge()}
  {#if !loading}
    <Badge label={String(currencies.length)} />
  {/if}
{/snippet}

{#snippet userIdCell(user: any)}
  <Text type="code" size="sm" color="secondary" maxLines={1}>{user.id}</Text>
{/snippet}
{#snippet userNameCell(user: any)}
  <Text weight="medium">{user.username}</Text>
{/snippet}
{#snippet userEmailCell(user: any)}
  <Text color="secondary">{user.email}</Text>
{/snippet}
{#snippet userFlagsCell(user: any)}
  {#if user.flags && user.flags.length > 0}
    <HStack gap={1} wrap="wrap">
      {#each user.flags as flag (flag)}
        <Token label={flag} size="sm" />
      {/each}
    </HStack>
  {:else}
    <Text type="supporting" color="secondary">none</Text>
  {/if}
{/snippet}
{#snippet userActionsCell(user: any)}
  <HStack gap={1} hAlign="end">
    <IconButton
      label="Edit user"
      size="sm"
      variant="ghost"
      icon={editIcon}
      onclick={() => {
        editingUser = user;
        editUsername = user.username;
        editEmail = user.email;
        editFlags = user.flags ? user.flags.join(', ') : '';
        editPassword = '';
        editModalOpen = true;
        showPassword = false;
      }}
    />
    <IconButton
      label="Delete user"
      size="sm"
      variant="destructive"
      icon={trashIcon}
      onclick={() => (userToDelete = user)}
    />
  </HStack>
{/snippet}

{#snippet inviteIdCell(invite: any)}
  <Text type="code" size="sm" color="secondary" maxLines={1}>{invite.id}</Text>
{/snippet}
{#snippet inviteTokenCell(invite: any)}
  <Text type="code" weight="medium">{invite.token}</Text>
{/snippet}
{#snippet inviteStatusCell(invite: any)}
  {#if invite.used_by}
    <Badge label="Used by {invite.used_by}" />
  {:else}
    <Badge label="Available" variant="success" />
  {/if}
{/snippet}
{#snippet inviteActionsCell(invite: any)}
  <HStack gap={1} hAlign="end">
    <IconButton
      label="Delete invite"
      size="sm"
      variant="destructive"
      icon={trashIcon}
      onclick={() => (inviteToDelete = invite)}
    />
  </HStack>
{/snippet}

{#snippet currCodeCell(curr: Currency)}
  <Text type="code" weight="semibold">{curr.code}</Text>
{/snippet}
{#snippet currNameCell(curr: Currency)}
  <Text>{curr.name}</Text>
{/snippet}
{#snippet currSymbolCell(curr: Currency)}
  <Text type="code" size="sm">
    {curr.symbol} ({curr.symbol_after ? 'after' : 'before'})
  </Text>
{/snippet}
{#snippet currDecimalsCell(curr: Currency)}
  <Text type="code" size="sm" hasTabularNumbers>{curr.decimal_digits}</Text>
{/snippet}
{#snippet currStatusCell(curr: Currency)}
  <Switch
    label="{curr.code} active"
    isLabelHidden
    size="sm"
    value={curr.is_active ?? true}
    onChange={() => toggleCurrencyActive(curr)}
  />
{/snippet}
{#snippet currActionsCell(curr: Currency)}
  <HStack gap={1} hAlign="end">
    <IconButton
      label="Edit currency"
      size="sm"
      variant="ghost"
      icon={editIcon}
      onclick={() => openEditCurrencyModal(curr)}
    />
  </HStack>
{/snippet}

{#snippet header()}
  <LayoutHeader hasDivider padding={5} paddingBlockEnd={0}>
    <VStack gap={4}>
      <HStack hAlign="between" vAlign="center" wrap="wrap" gap={3}>
        <VStack gap={0.5}>
          <Heading level={1}>Admin</Heading>
          <Text type="supporting" color="secondary">
            Manage users, invite codes, supported currencies, and exchange rates.
          </Text>
        </VStack>
        {#if activeTab === 'currencies'}
          <Button
            label="Add currency"
            variant="primary"
            size="sm"
            icon={plusIcon}
            onclick={openCreateCurrencyModal}
          />
        {/if}
      </HStack>

      <TabList
        value={activeTab}
        onChange={(v) => (activeTab = v as AdminTab)}
        isFullBleed
        role="tablist"
      >
        <Tab value="users" label="Users" panelId="admin-panel-users" endContent={usersCountBadge} />
        <Tab
          value="invites"
          label="Invites"
          panelId="admin-panel-invites"
          endContent={invitesCountBadge}
        />
        <Tab
          value="currencies"
          label="Currencies"
          panelId="admin-panel-currencies"
          endContent={currenciesCountBadge}
        />
        <Tab value="rates" label="Exchange rates" panelId="admin-panel-rates" />
      </TabList>
    </VStack>
  </LayoutHeader>
{/snippet}

{#snippet content()}
  <LayoutContent padding={0} label="Admin workspace">
    {#if loading}
      <Stack padding={8}>
        <HStack hAlign="center">
          <Spinner size="lg" label="Loading admin data..." />
        </HStack>
      </Stack>
    {:else if activeTab === 'users'}
      <Section id="admin-panel-users" role="tabpanel" variant="transparent" padding={0}>
        {#if users.length === 0}
          <Stack padding={8}>
            <EmptyState title="No users found" />
          </Stack>
        {:else}
          <Table
            data={users}
            columns={userColumns}
            idKey="id"
            dividers="rows"
            hasHover
            density="compact"
          />
        {/if}
      </Section>
    {:else if activeTab === 'invites'}
      <Section id="admin-panel-invites" role="tabpanel" variant="transparent" padding={0}>
        <Section variant="transparent" padding={4} dividers={['bottom']}>
          <form onsubmit={createInvite}>
            <HStack gap={2} wrap="wrap" vAlign="center">
              <StackItem size="fill">
                <TextInput
                  label="New invite code"
                  isLabelHidden
                  placeholder="New invite code (e.g. BETA-2025)"
                  value={newInviteToken}
                  onChange={(v) => (newInviteToken = v)}
                />
              </StackItem>
              <Button
                label="Create invite"
                type="submit"
                variant="primary"
                icon={plusIcon}
                isDisabled={!newInviteToken.trim()}
              />
            </HStack>
          </form>
        </Section>
        {#if invites.length === 0}
          <Stack padding={8}>
            <EmptyState
              title="No invite codes yet"
              description="Create an invite code above to allow new sign-ups."
            />
          </Stack>
        {:else}
          <Table
            data={invites}
            columns={inviteColumns}
            idKey="id"
            dividers="rows"
            hasHover
            density="compact"
          />
        {/if}
      </Section>
    {:else if activeTab === 'currencies'}
      <Section id="admin-panel-currencies" role="tabpanel" variant="transparent" padding={0}>
        <Section variant="transparent" padding={4} dividers={['bottom']}>
          <HStack gap={3} wrap="wrap" vAlign="center" hAlign="between">
            <StackItem size="fill">
              <TextInput
                label="Search currencies"
                isLabelHidden
                type="text"
                placeholder="Search by code, name, or symbol..."
                value={currencySearch}
                onChange={(v) => (currencySearch = v)}
                startIcon={searchIcon}
                hasClear
              />
            </StackItem>
            <SegmentedControl
              label="Currency filter"
              value={currencyFilter}
              onChange={(v) => (currencyFilter = v as typeof currencyFilter)}
              size="sm"
            >
              <SegmentedControlItem value="all" label="All ({currencies.length})" />
              <SegmentedControlItem value="active" label="Active ({activeCurrencyCount})" />
              <SegmentedControlItem
                value="inactive"
                label="Inactive ({currencies.length - activeCurrencyCount})"
              />
            </SegmentedControl>
          </HStack>
        </Section>
        {#if filteredCurrencies.length === 0}
          <Stack padding={8}>
            <EmptyState
              title="No matching currencies"
              description="Adjust your search or status filter."
            />
          </Stack>
        {:else}
          <Table
            data={filteredCurrencies}
            columns={currencyColumns}
            idKey="code"
            dividers="rows"
            hasHover
            density="compact"
          />
        {/if}
      </Section>
    {:else if activeTab === 'rates'}
      <Section id="admin-panel-rates" role="tabpanel" variant="transparent" padding={6}>
        <Grid columns={{ minWidth: 320, max: 2 }} gap={5}>
          <Card padding={5}>
            <VStack gap={4}>
              <VStack gap={1}>
                <Heading level={3}>Sync from upstream providers</Heading>
                <Text type="supporting" color="secondary">
                  Fetch latest daily rates, a specific historical date, or backfill trailing days.
                </Text>
              </VStack>
              <SegmentedControl
                label="Sync mode"
                value={syncMode}
                onChange={(v) => (syncMode = v as typeof syncMode)}
                layout="fill"
                size="sm"
              >
                <SegmentedControlItem value="latest" label="Latest" />
                <SegmentedControlItem value="date" label="Specific date" />
                <SegmentedControlItem value="days" label="Backfill days" />
              </SegmentedControl>

              {#if syncMode === 'date'}
                <DateInput
                  label="Date"
                  value={syncDate as ISODateString}
                  onChange={(v: string | undefined) => (syncDate = v ?? '')}
                  format="system_date"
                />
              {:else if syncMode === 'days'}
                <HStack gap={4} wrap="wrap" vAlign="end">
                  <NumberInput
                    label="Trailing days (1-366)"
                    min={1}
                    max={366}
                    value={syncDays}
                    onChange={(v: number | null) => (syncDays = v ?? 30)}
                    isIntegerOnly
                  />
                  <CheckboxInput
                    label="Force re-fetch"
                    value={syncForce}
                    onChange={(checked) => (syncForce = checked)}
                  />
                </HStack>
              {/if}

              <Button
                label={syncingRates
                  ? 'Syncing...'
                  : syncMode === 'latest'
                    ? 'Sync latest rates'
                    : syncMode === 'date'
                      ? `Sync ${syncDate}`
                      : `Backfill last ${syncDays} days`}
                variant="primary"
                icon={refreshIcon}
                isLoading={syncingRates}
                onclick={triggerRateSync}
                width="100%"
              />
            </VStack>
          </Card>

          <Card padding={5}>
            <form onsubmit={saveManualRate}>
              <FormLayout>
                <VStack gap={1}>
                  <Heading level={3}>Manual rate override</Heading>
                  <Text type="supporting" color="secondary">
                    Upsert an exact exchange rate for a currency pair on a specific date.
                  </Text>
                </VStack>
                <Grid columns={2} gap={3}>
                  <TextInput
                    label="Base code"
                    value={manualBase}
                    onChange={(v) => (manualBase = v)}
                    placeholder="USD"
                  />
                  <TextInput
                    label="Target code"
                    value={manualTarget}
                    onChange={(v) => (manualTarget = v)}
                    placeholder="EUR"
                  />
                  <TextInput
                    label="Rate (1 base = X target)"
                    value={manualRate}
                    onChange={(v) => (manualRate = v)}
                    placeholder="0.9250"
                  />
                  <DateInput
                    label="Rate date"
                    value={manualDate as ISODateString}
                    onChange={(v: string | undefined) => (manualDate = v ?? '')}
                    format="system_date"
                  />
                </Grid>
                <Button
                  label={savingManualRate ? 'Saving...' : 'Save manual rate'}
                  type="submit"
                  variant="secondary"
                  isLoading={savingManualRate}
                  isDisabled={!manualRate}
                  width="100%"
                />
              </FormLayout>
            </form>
          </Card>
        </Grid>
      </Section>
    {/if}
  </LayoutContent>
{/snippet}

<Layout height="fill" {header} {content} />

<Modal bind:open={editModalOpen} title="Edit User">
  {#if editingUser}
    <form onsubmit={saveUser}>
      <FormLayout>
        <Text type="supporting" color="secondary">
          Editing {editingUser.username}
        </Text>
        <TextInput label="Username" value={editUsername} onChange={(v) => (editUsername = v)} />
        <TextInput label="Email" type="email" value={editEmail} onChange={(v) => (editEmail = v)} />
        <TextInput
          label="Flags (comma separated)"
          placeholder="e.g. is_admin, beta_tester"
          value={editFlags}
          onChange={(v) => (editFlags = v)}
        />
        <HStack gap={2} vAlign="end">
          <StackItem size="fill">
            <TextInput
              label="New password (leave blank to keep current)"
              type={showPassword ? 'text' : 'password'}
              value={editPassword}
              onChange={(v) => (editPassword = v)}
            />
          </StackItem>
          <IconButton
            label={showPassword ? 'Hide password' : 'Show password'}
            icon={showPassword ? eyeOffIcon : eyeIcon}
            onclick={() => (showPassword = !showPassword)}
          />
        </HStack>
        <HStack gap={3} hAlign="end" paddingBlockStart={2}>
          <Button label="Cancel" variant="secondary" onclick={() => (editModalOpen = false)} />
          <Button label="Save changes" type="submit" variant="primary" />
        </HStack>
      </FormLayout>
    </form>
  {/if}
</Modal>

<Modal
  bind:open={currencyModalOpen}
  title={editingCurrencyCode ? `Edit Currency (${editingCurrencyCode})` : 'Add Currency'}
>
  <form onsubmit={saveCurrency}>
    <FormLayout>
      <Grid columns={2} gap={3}>
        <TextInput
          label="Code"
          value={currCode}
          onChange={(v) => (currCode = v)}
          isDisabled={editingCurrencyCode !== null}
          placeholder="BTC"
        />
        <TextInput
          label="Symbol"
          value={currSymbol}
          onChange={(v) => (currSymbol = v)}
          placeholder="₿"
        />
      </Grid>
      <TextInput
        label="Name"
        value={currName}
        onChange={(v) => (currName = v)}
        placeholder="Bitcoin"
      />
      <NumberInput
        label="Decimal digits"
        min={0}
        max={8}
        value={currDecimalDigits}
        onChange={(v: number | null) => (currDecimalDigits = v ?? 2)}
        isIntegerOnly
      />
      <CheckboxInput
        label="Symbol after amount"
        value={currSymbolAfter}
        onChange={(checked) => (currSymbolAfter = checked)}
      />
      <CheckboxInput
        label="Active (selectable by users)"
        value={currIsActive}
        onChange={(checked) => (currIsActive = checked)}
      />
      <HStack gap={3} hAlign="end" paddingBlockStart={2}>
        <Button label="Cancel" variant="secondary" onclick={() => (currencyModalOpen = false)} />
        <Button
          label={savingCurrency ? 'Saving...' : 'Save currency'}
          type="submit"
          variant="primary"
          isLoading={savingCurrency}
        />
      </HStack>
    </FormLayout>
  </form>
</Modal>

<AlertDialog
  isOpen={userToDelete !== null}
  onOpenChange={(next) => {
    if (!next) userToDelete = null;
  }}
  title="Delete {userToDelete?.username ?? 'user'}?"
  description="This permanently removes the user and their access. This cannot be undone."
  actionLabel="Delete user"
  actionVariant="destructive"
  onAction={deleteUser}
/>

<AlertDialog
  isOpen={inviteToDelete !== null}
  onOpenChange={(next) => {
    if (!next) inviteToDelete = null;
  }}
  title="Delete invite {inviteToDelete?.token ?? ''}?"
  description="The invite code will stop working immediately."
  actionLabel="Delete invite"
  actionVariant="destructive"
  onAction={deleteInvite}
/>
