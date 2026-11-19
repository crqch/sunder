<script lang="ts">
  import { authenticatedFetch } from '$lib/auth';
  import { currencyStore } from '$lib/currency.svelte';
  import type { Currency } from '$lib/types';
  import { onMount } from 'svelte';
  import {
    Trash2,
    Plus,
    ShieldCheck,
    Eye,
    EyeOff,
    Pencil,
    RefreshCw,
    Search,
    Coins
  } from '@lucide/svelte';
  import { toast } from 'svelte-sonner';
  import Modal from '$components/Modal.svelte';

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
      const [uRes, iRes, cRes] = await Promise.all([
        authenticatedFetch('/admin/users'),
        authenticatedFetch('/admin/invites'),
        authenticatedFetch('/admin/currencies')
      ]);
      const [uData, iData, cData] = (await Promise.all([
        uRes.json(),
        iRes.json(),
        cRes.json()
      ])) as any[];
      users = uData.users || [];
      invites = iData.invites || [];
      currencies = cData.currencies || [];
    } catch (e) {
      toast.error('Failed to load admin data');
    } finally {
      loading = false;
    }
  }

  onMount(loadData);

  async function deleteUser(id: string) {
    if (!confirm('Are you sure you want to delete this user?')) return;
    const res = await authenticatedFetch(`/admin/users/${id}`, { method: 'DELETE' });
    if (res.ok) {
      toast.success('User deleted');
      users = users.filter((u) => u.id !== id);
    } else {
      toast.error('Failed to delete user');
    }
  }

  async function saveUser() {
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

  async function createInvite() {
    if (!newInviteToken) return;
    const res = await authenticatedFetch(`/admin/invites`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ token: newInviteToken })
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

  async function deleteInvite(id: string) {
    if (!confirm('Delete this invite?')) return;
    const res = await authenticatedFetch(`/admin/invites/${id}`, { method: 'DELETE' });
    if (res.ok) {
      toast.success('Invite deleted');
      invites = invites.filter((i) => i.id !== id);
    } else {
      toast.error('Failed to delete invite');
    }
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

  async function saveCurrency() {
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

  async function saveManualRate() {
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
</script>

<div class="mx-auto flex max-w-5xl flex-col gap-8 py-6">
  <div class="border-border flex items-center gap-3 border-b pb-4">
    <ShieldCheck class="text-primary size-8" />
    <div>
      <h1 class="text-3xl font-bold tracking-tight">Admin Dashboard</h1>
      <p class="text-muted-foreground text-sm">
        Manage users, invite codes, supported currencies, and exchange rates
      </p>
    </div>
  </div>

  {#if loading}
    <div class="text-muted-foreground animate-pulse py-10 text-center">Loading admin data...</div>
  {:else}
    <!-- Users Section -->
    <section class="flex flex-col gap-4">
      <h2 class="text-2xl font-bold">Users</h2>
      <div class="border-border bg-card overflow-x-auto rounded-xl border">
        <table class="w-full text-left text-sm">
          <thead class="bg-muted/50 border-border border-b">
            <tr>
              <th class="p-3 font-semibold">ID</th>
              <th class="p-3 font-semibold">Username</th>
              <th class="p-3 font-semibold">Email</th>
              <th class="p-3 font-semibold">Flags</th>
              <th class="p-3 text-right font-semibold">Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each users as user}
              <tr class="border-border/50 hover:bg-muted/20 border-b last:border-0">
                <td class="p-3 font-mono text-xs">{user.id}</td>
                <td class="p-3 font-medium">{user.username}</td>
                <td class="text-muted-foreground p-3">{user.email}</td>
                <td class="p-3">
                  {#if user.flags && user.flags.length > 0}
                    <div class="flex gap-1">
                      {#each user.flags as flag}
                        <span
                          class="bg-primary/10 text-primary rounded px-2 py-0.5 text-xs font-medium"
                          >{flag}</span
                        >
                      {/each}
                    </div>
                  {:else}
                    <span class="text-muted-foreground text-xs italic">none</span>
                  {/if}
                </td>
                <td class="p-3 text-right">
                  <button
                    class="btn btn-outline mr-2 h-auto p-1.5 text-xs"
                    onclick={() => {
                      editingUser = user;
                      editUsername = user.username;
                      editEmail = user.email;
                      editFlags = user.flags ? user.flags.join(', ') : '';
                      editPassword = '';
                      editModalOpen = true;
                      showPassword = false;
                    }}
                    title="Edit User"
                  >
                    <Pencil class="size-4" />
                  </button>
                  <button
                    class="btn bg-destructive text-destructive-foreground h-auto p-1.5 text-xs"
                    onclick={() => deleteUser(user.id)}
                    title="Delete User"
                  >
                    <Trash2 class="size-4" />
                  </button>
                </td>
              </tr>
            {/each}
            {#if users.length === 0}
              <tr
                ><td colspan="5" class="text-muted-foreground p-6 text-center">No users found.</td
                ></tr
              >
            {/if}
          </tbody>
        </table>
      </div>
    </section>

    <!-- Invites Section -->
    <section class="flex flex-col gap-4">
      <h2 class="text-2xl font-bold">Invite Codes</h2>
      <div class="flex gap-2">
        <input
          type="text"
          class="input flex-1"
          placeholder="New invite code (e.g. BETA-2024)"
          bind:value={newInviteToken}
        />
        <button
          class="btn bg-primary text-primary-foreground px-4"
          onclick={createInvite}
          disabled={!newInviteToken}
        >
          <Plus class="mr-2 size-4" /> Create Invite
        </button>
      </div>

      <div class="border-border bg-card overflow-x-auto rounded-xl border">
        <table class="w-full text-left text-sm">
          <thead class="bg-muted/50 border-border border-b">
            <tr>
              <th class="p-3 font-semibold">ID</th>
              <th class="p-3 font-semibold">Token</th>
              <th class="p-3 font-semibold">Status</th>
              <th class="p-3 text-right font-semibold">Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each invites as invite}
              <tr class="border-border/50 hover:bg-muted/20 border-b last:border-0">
                <td class="p-3 font-mono text-xs">{invite.id}</td>
                <td class="p-3 font-mono font-medium">{invite.token}</td>
                <td class="p-3">
                  {#if invite.used_by}
                    <span
                      class="bg-muted text-muted-foreground rounded px-2 py-0.5 text-xs font-medium"
                      >Used by {invite.used_by}</span
                    >
                  {:else}
                    <span
                      class="rounded bg-emerald-500/10 px-2 py-0.5 text-xs font-medium text-emerald-500"
                      >Available</span
                    >
                  {/if}
                </td>
                <td class="p-3 text-right">
                  <button
                    class="btn bg-destructive text-destructive-foreground h-auto p-1.5 text-xs"
                    onclick={() => deleteInvite(invite.id)}
                    title="Delete Invite"
                  >
                    <Trash2 class="size-4" />
                  </button>
                </td>
              </tr>
            {/each}
            {#if invites.length === 0}
              <tr
                ><td colspan="4" class="text-muted-foreground p-6 text-center"
                  >No invite codes found.</td
                ></tr
              >
            {/if}
          </tbody>
        </table>
      </div>
    </section>

    <!-- Exchange Rates Section -->
    <section class="flex flex-col gap-4">
      <div class="flex items-center gap-2">
        <Coins class="text-primary size-6" />
        <h2 class="text-2xl font-bold">Exchange Rates</h2>
      </div>

      <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <!-- Sync & Backfill Card -->
        <div class="border-border bg-card space-y-4 rounded-xl border p-5">
          <div>
            <h3 class="font-semibold">Sync Rates from Upstream Providers</h3>
            <p class="text-muted-foreground text-xs">
              Fetch latest daily rates, a specific historical date, or backfill trailing days.
            </p>
          </div>

          <div class="flex flex-wrap gap-2">
            <button
              type="button"
              onclick={() => (syncMode = 'latest')}
              class="btn btn-sm {syncMode === 'latest' ? '' : 'btn-outline'}"
            >
              Latest
            </button>
            <button
              type="button"
              onclick={() => (syncMode = 'date')}
              class="btn btn-sm {syncMode === 'date' ? '' : 'btn-outline'}"
            >
              Specific Date
            </button>
            <button
              type="button"
              onclick={() => (syncMode = 'days')}
              class="btn btn-sm {syncMode === 'days' ? '' : 'btn-outline'}"
            >
              Backfill Days
            </button>
          </div>

          {#if syncMode === 'date'}
            <label class="flex flex-col gap-1 text-xs font-medium">
              Date (YYYY-MM-DD)
              <input type="date" class="input h-9 text-sm" bind:value={syncDate} />
            </label>
          {:else if syncMode === 'days'}
            <div class="flex flex-wrap items-center gap-3">
              <label class="flex flex-1 flex-col gap-1 text-xs font-medium">
                Trailing Days (1-366)
                <input
                  type="number"
                  min="1"
                  max="366"
                  class="input h-9 text-sm"
                  bind:value={syncDays}
                />
              </label>
              <label class="mt-4 flex cursor-pointer items-center gap-2 text-xs font-medium">
                <input type="checkbox" bind:checked={syncForce} />
                Force re-fetch
              </label>
            </div>
          {/if}

          <button
            onclick={triggerRateSync}
            disabled={syncingRates}
            class="btn btn-sm w-full gap-2 disabled:opacity-50"
          >
            <RefreshCw size={14} class={syncingRates ? 'animate-spin' : ''} />
            {syncingRates
              ? 'Syncing...'
              : syncMode === 'latest'
                ? 'Sync Latest Rates'
                : syncMode === 'date'
                  ? `Sync ${syncDate}`
                  : `Backfill Last ${syncDays} Days`}
          </button>
        </div>

        <!-- Manual Rate Override Card -->
        <div class="border-border bg-card space-y-4 rounded-xl border p-5">
          <div>
            <h3 class="font-semibold">Manual Rate Override</h3>
            <p class="text-muted-foreground text-xs">
              Upsert an exact exchange rate for a currency pair on a specific date.
            </p>
          </div>

          <div class="grid grid-cols-2 gap-3">
            <label class="flex flex-col gap-1 text-xs font-medium">
              Base Code
              <input
                type="text"
                maxlength="10"
                class="input h-9 font-mono text-sm uppercase"
                bind:value={manualBase}
                placeholder="USD"
              />
            </label>
            <label class="flex flex-col gap-1 text-xs font-medium">
              Target Code
              <input
                type="text"
                maxlength="10"
                class="input h-9 font-mono text-sm uppercase"
                bind:value={manualTarget}
                placeholder="EUR"
              />
            </label>
            <label class="flex flex-col gap-1 text-xs font-medium">
              Rate (1 Base = X Target)
              <input
                type="number"
                step="any"
                min="0"
                class="input h-9 text-sm"
                bind:value={manualRate}
                placeholder="0.9250"
              />
            </label>
            <label class="flex flex-col gap-1 text-xs font-medium">
              Rate Date
              <input type="date" class="input h-9 text-sm" bind:value={manualDate} />
            </label>
          </div>

          <button
            onclick={saveManualRate}
            disabled={savingManualRate || !manualRate}
            class="btn btn-outline btn-sm w-full disabled:opacity-50"
          >
            {savingManualRate ? 'Saving...' : 'Save Manual Rate'}
          </button>
        </div>
      </div>
    </section>

    <!-- Currencies Section -->
    <section class="flex flex-col gap-4">
      <div class="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 class="text-2xl font-bold">Supported Currencies</h2>
          <p class="text-muted-foreground text-xs">
            {activeCurrencyCount} active of {currencies.length} total server currencies
          </p>
        </div>
        <button onclick={openCreateCurrencyModal} class="btn btn-sm gap-1.5">
          <Plus size={14} /> Add Currency
        </button>
      </div>

      <div class="flex flex-col gap-2 sm:flex-row">
        <div class="relative flex-1">
          <Search
            size={15}
            class="text-muted-foreground absolute top-1/2 left-3 -translate-y-1/2"
          />
          <input
            type="text"
            bind:value={currencySearch}
            placeholder="Search by code, name, or symbol..."
            class="input h-9 w-full pl-9 text-sm"
          />
        </div>
        <div class="flex gap-1.5">
          <button
            type="button"
            onclick={() => (currencyFilter = 'all')}
            class="btn btn-sm {currencyFilter === 'all' ? '' : 'btn-outline'}"
          >
            All ({currencies.length})
          </button>
          <button
            type="button"
            onclick={() => (currencyFilter = 'active')}
            class="btn btn-sm {currencyFilter === 'active' ? '' : 'btn-outline'}"
          >
            Active ({activeCurrencyCount})
          </button>
          <button
            type="button"
            onclick={() => (currencyFilter = 'inactive')}
            class="btn btn-sm {currencyFilter === 'inactive' ? '' : 'btn-outline'}"
          >
            Inactive ({currencies.length - activeCurrencyCount})
          </button>
        </div>
      </div>

      <div class="border-border bg-card max-h-[28rem] overflow-auto rounded-xl border">
        <table class="w-full text-left text-sm">
          <thead class="bg-muted/80 border-border sticky top-0 border-b backdrop-blur">
            <tr>
              <th class="p-3 font-semibold">Code</th>
              <th class="p-3 font-semibold">Name</th>
              <th class="p-3 font-semibold">Symbol</th>
              <th class="p-3 font-semibold">Decimals</th>
              <th class="p-3 font-semibold">Status</th>
              <th class="p-3 text-right font-semibold">Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each filteredCurrencies as curr (curr.code)}
              <tr class="border-border/50 hover:bg-muted/20 border-b last:border-0">
                <td class="p-3 font-mono font-semibold">{curr.code}</td>
                <td class="p-3">{curr.name}</td>
                <td class="p-3 font-mono text-xs">
                  {curr.symbol}
                  <span class="text-muted-foreground ml-1">
                    ({curr.symbol_after ? 'after' : 'before'})
                  </span>
                </td>
                <td class="p-3 font-mono text-xs">{curr.decimal_digits}</td>
                <td class="p-3">
                  <button
                    type="button"
                    onclick={() => toggleCurrencyActive(curr)}
                    class="rounded px-2 py-0.5 text-xs font-medium transition-colors {curr.is_active
                      ? 'bg-emerald-500/10 text-emerald-500 hover:bg-emerald-500/20'
                      : 'bg-muted text-muted-foreground hover:bg-muted/80'}"
                    title="Click to toggle active status"
                  >
                    {curr.is_active ? 'Active' : 'Inactive'}
                  </button>
                </td>
                <td class="p-3 text-right">
                  <button
                    class="btn btn-outline h-auto p-1.5 text-xs"
                    onclick={() => openEditCurrencyModal(curr)}
                    title="Edit Currency"
                  >
                    <Pencil class="size-3.5" />
                  </button>
                </td>
              </tr>
            {/each}
            {#if filteredCurrencies.length === 0}
              <tr>
                <td colspan="6" class="text-muted-foreground p-6 text-center">
                  No currencies match your filter.
                </td>
              </tr>
            {/if}
          </tbody>
        </table>
      </div>
    </section>
  {/if}
</div>

<Modal bind:open={editModalOpen} title="Edit User">
  {#if editingUser}
    <div class="flex flex-col gap-4">
      <p class="text-muted-foreground -mt-4 text-sm">
        Editing <span class="text-foreground font-bold">{editingUser.username}</span>
      </p>

      <label class="flex flex-col gap-1.5 text-sm font-medium">
        Username
        <input type="text" class="input" bind:value={editUsername} />
      </label>

      <label class="flex flex-col gap-1.5 text-sm font-medium">
        Email
        <input type="email" class="input" bind:value={editEmail} />
      </label>

      <label class="flex flex-col gap-1.5 text-sm font-medium">
        Flags (comma separated)
        <input
          type="text"
          class="input"
          bind:value={editFlags}
          placeholder="e.g. is_admin, beta_tester"
        />
      </label>

      <label class="relative mt-2 flex flex-col gap-1.5 text-sm font-medium">
        New Password <span class="text-muted-foreground text-xs font-normal"
          >(Leave blank to keep current)</span
        >
        <div class="relative">
          <input
            type={showPassword ? 'text' : 'password'}
            class="input w-full pr-10"
            bind:value={editPassword}
          />
          <button
            type="button"
            class="text-muted-foreground hover:text-foreground absolute top-1/2 right-2 -translate-y-1/2 p-1"
            onclick={() => (showPassword = !showPassword)}
            title={showPassword ? 'Hide password' : 'Show password'}
          >
            {#if showPassword}
              <EyeOff class="size-4" />
            {:else}
              <Eye class="size-4" />
            {/if}
          </button>
        </div>
      </label>
      <div class="border-border mt-2 flex justify-end gap-3 border-t pt-4">
        <button
          class="btn btn-outline px-4 py-2"
          onclick={() => {
            editModalOpen = false;
          }}>Cancel</button
        >
        <button class="btn bg-primary text-primary-foreground px-4 py-2" onclick={saveUser}
          >Save Changes</button
        >
      </div>
    </div>
  {/if}
</Modal>

<Modal
  bind:open={currencyModalOpen}
  title={editingCurrencyCode ? `Edit Currency (${editingCurrencyCode})` : 'Add Currency'}
>
  <div class="flex flex-col gap-4">
    <div class="grid grid-cols-2 gap-3">
      <label class="flex flex-col gap-1.5 text-sm font-medium">
        Code
        <input
          type="text"
          class="input font-mono uppercase"
          bind:value={currCode}
          disabled={editingCurrencyCode !== null}
          placeholder="BTC"
          maxlength="10"
        />
      </label>
      <label class="flex flex-col gap-1.5 text-sm font-medium">
        Symbol
        <input type="text" class="input" bind:value={currSymbol} placeholder="₿" maxlength="10" />
      </label>
    </div>

    <label class="flex flex-col gap-1.5 text-sm font-medium">
      Name
      <input type="text" class="input" bind:value={currName} placeholder="Bitcoin" />
    </label>

    <div class="grid grid-cols-2 gap-3">
      <label class="flex flex-col gap-1.5 text-sm font-medium">
        Decimal Digits
        <input type="number" min="0" max="8" class="input" bind:value={currDecimalDigits} />
      </label>
      <div class="flex flex-col justify-end gap-2 pb-1">
        <label class="flex cursor-pointer items-center gap-2 text-sm font-medium">
          <input type="checkbox" bind:checked={currSymbolAfter} />
          Symbol after amount
        </label>
        <label class="flex cursor-pointer items-center gap-2 text-sm font-medium">
          <input type="checkbox" bind:checked={currIsActive} />
          Active (selectable by users)
        </label>
      </div>
    </div>

    <div class="border-border mt-2 flex justify-end gap-3 border-t pt-4">
      <button class="btn btn-outline px-4 py-2" onclick={() => (currencyModalOpen = false)}>
        Cancel
      </button>
      <button
        class="btn bg-primary text-primary-foreground px-4 py-2 disabled:opacity-50"
        disabled={savingCurrency}
        onclick={saveCurrency}
      >
        {savingCurrency ? 'Saving...' : 'Save Currency'}
      </button>
    </div>
  </div>
</Modal>
