<script lang="ts">
  import {
    Avatar,
    Button,
    Divider,
    FileInput,
    Heading,
    HStack,
    Icon,
    Layout,
    LayoutContent,
    LayoutHeader,
    MetadataList,
    MetadataListItem,
    SegmentedControl,
    SegmentedControlItem,
    StackItem,
    Switch,
    Text,
    VStack
  } from '@astryx-sv/core';
  import { Download, LogOut, Monitor, Moon, RefreshCw, Sun } from '@lucide/svelte';
  import { authStore, logout, authenticatedFetch } from '$lib/auth';
  import { syncAll, getUnsyncedChanges, clearLocalDatabase } from '$lib/sync';
  import { syncState } from '$lib/syncState.svelte';
  import { currencyStore } from '$lib/currency.svelte';
  import { themeStore, type ThemeMode } from '$lib/themeStore';
  import { toast } from '$lib/toast.svelte';
  import Modal from '$components/Modal.svelte';
  import CurrencySelect from '$components/CurrencySelect.svelte';
  import Select from '$components/Select.svelte';

  let lastSync = $state(
    typeof window !== 'undefined'
      ? localStorage.getItem('sunder_last_sync_timestamp') || 'Never'
      : 'Never'
  );
  let syncing = $state(false);
  let savingCurrency = $state(false);
  let refreshingRates = $state(false);
  let selectedDefaultCurrency = $state(currencyStore.defaultCurrencyCode);

  $effect(() => {
    selectedDefaultCurrency = currencyStore.defaultCurrencyCode;
  });

  // Keep "Last synced" fresh whenever background auto-sync or SyncIndicator finishes.
  $effect(() => {
    const _status = syncState.status;
    const _pending = syncState.pendingCount;
    if (typeof window !== 'undefined') {
      lastSync = localStorage.getItem('sunder_last_sync_timestamp') || 'Never';
    }
  });

  let logoutModalOpen = $state(false);
  let unsyncedCount = $state(0);

  let sunderFile = $state<File | null>(null);
  let bagelsFile = $state<File | null>(null);

  const intervalOptions = [
    { value: 1, label: 'Every 1 minute' },
    { value: 5, label: 'Every 5 minutes' },
    { value: 15, label: 'Every 15 minutes' },
    { value: 60, label: 'Every 1 hour' }
  ];

  async function handleDefaultCurrencyChange(code: string) {
    if (!code || code === currencyStore.defaultCurrencyCode) return;
    savingCurrency = true;
    try {
      const updated = await currencyStore.updateDefaultCurrency(code);
      selectedDefaultCurrency = updated;
      toast.success(`Default currency set to ${updated}`);
    } catch (err) {
      console.error('Failed to update default currency', err);
      selectedDefaultCurrency = currencyStore.defaultCurrencyCode;
      const msg =
        err instanceof Error && err.message ? err.message : 'Failed to update default currency';
      toast.error(msg);
    } finally {
      savingCurrency = false;
    }
  }

  async function handleRefreshCurrencyData() {
    refreshingRates = true;
    try {
      await currencyStore.syncAllCurrencyData(true);
      toast.success('Currencies and exchange rates updated');
    } catch (err) {
      console.error('Failed to refresh currency data', err);
      toast.error('Failed to refresh currencies and rates');
    } finally {
      refreshingRates = false;
    }
  }

  async function handleLogoutClick() {
    try {
      const unsynced = await getUnsyncedChanges();
      unsyncedCount =
        unsynced.accounts.length + unsynced.categories.length + unsynced.entries.length;

      if (unsyncedCount > 0) {
        logoutModalOpen = true;
      } else {
        await executeLogout();
      }
    } catch (err) {
      console.error('Failed to check sync status', err);
      await executeLogout();
    }
  }

  async function executeLogout() {
    await clearLocalDatabase();
    logout();
  }

  async function handleLogoutAndSync() {
    const success = await handleSync();
    if (success) {
      await executeLogout();
    }
  }

  async function handleSync() {
    syncing = true;
    try {
      await syncAll();
      lastSync = localStorage.getItem('sunder_last_sync_timestamp') || new Date().toISOString();
      toast.success('Sync completed successfully!');
      return true;
    } catch (err) {
      console.error('Sync failed', err);
      toast.error('Sync failed. Please try again.');
      return false;
    } finally {
      syncing = false;
    }
  }

  function handleAutoSyncToggle(checked: boolean) {
    syncState.autoSyncEnabled = checked;
    localStorage.setItem('sunder_auto_sync', checked.toString());
  }

  function handleIntervalChange(value: string | number) {
    const val = Number(value);
    syncState.autoSyncInterval = val;
    localStorage.setItem('sunder_auto_sync_interval', val.toString());
  }

  async function downloadSunderBackup() {
    try {
      const res = await authenticatedFetch('/dashboard/eco/data/export');
      if (!res.ok) throw new Error('Failed to export data');
      const blob = await res.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `sunder_backup_${new Date().toISOString().split('T')[0]}.json`;
      document.body.appendChild(a);
      a.click();
      a.remove();
      window.URL.revokeObjectURL(url);
    } catch (e) {
      toast.error('Failed to download backup');
    }
  }

  async function uploadFile(file: File, type: 'sunder' | 'bagels') {
    const formData = new FormData();
    formData.append('file', file);
    try {
      toast.loading(`Importing ${type === 'sunder' ? 'Sunder' : 'Bagels'} data...`);
      const res = await authenticatedFetch(`/dashboard/eco/data/import/${type}`, {
        method: 'POST',
        body: formData
      });
      toast.dismiss();
      if (!res.ok) throw new Error('Import failed');
      const data = (await res.json()) as any;

      toast.loading('Syncing to device...');
      await syncAll();
      toast.dismiss();

      const counts = data.stats || {};
      toast.success(
        `Successfully imported ${counts.accounts || 0} accounts, ${counts.categories || 0} categories, and ${counts.entries || 0} entries!`
      );
    } catch (e) {
      toast.dismiss();
      toast.error(`Failed to import ${type} data`);
    }
  }

  function handleSunderImport(files: File | File[] | null) {
    const file = Array.isArray(files) ? (files[0] ?? null) : files;
    sunderFile = file;
    if (file) {
      uploadFile(file, 'sunder');
      sunderFile = null;
    }
  }

  function handleBagelsImport(files: File | File[] | null) {
    const file = Array.isArray(files) ? (files[0] ?? null) : files;
    bagelsFile = file;
    if (file) {
      uploadFile(file, 'bagels');
      bagelsFile = null;
    }
  }

  let formattedLastSync = $derived.by(() => {
    if (!lastSync || lastSync === 'Never') return 'Never';
    const d = new Date(lastSync);
    return Number.isNaN(d.getTime()) ? lastSync : d.toLocaleString();
  });
</script>

{#snippet logoutIcon()}
  <Icon icon={LogOut} size="sm" />
{/snippet}
{#snippet refreshIcon()}
  <Icon icon={RefreshCw} size="sm" />
{/snippet}
{#snippet downloadIcon()}
  <Icon icon={Download} size="sm" />
{/snippet}
{#snippet systemThemeIcon()}
  <Icon icon={Monitor} size="sm" />
{/snippet}
{#snippet lightThemeIcon()}
  <Icon icon={Sun} size="sm" />
{/snippet}
{#snippet darkThemeIcon()}
  <Icon icon={Moon} size="sm" />
{/snippet}

{#snippet header()}
  <!-- Header and content share both padding={6} and scrollbar-gutter: stable so
       classic-scrollbar platforms never shift centered 800px content left of
       the header title. -->
  <LayoutHeader hasDivider padding={6} class="sunder-settings-header">
    <VStack gap={0.5}>
      <Heading level={1}>Settings</Heading>
      <Text type="supporting" color="secondary">
        Manage your profile, appearance, display currency, background sync, and local backups.
      </Text>
    </VStack>
  </LayoutHeader>
{/snippet}

{#snippet content()}
  <!-- No Section wrappers here: plain stacks keep every heading pixel-aligned
       with the header title, with full padding={6} breathing room below the
       header divider. -->
  <LayoutContent padding={6} label="Settings" class="sunder-settings-content">
    <VStack gap={6}>
      <VStack gap={4}>
        <Heading level={2}>Profile</Heading>
        <HStack gap={4} vAlign="center" wrap="wrap">
          <Avatar name={$authStore.user?.username || 'User'} size="lg" tooltip={false} />
          <StackItem size="fill">
            <MetadataList orientation="horizontal">
              <MetadataListItem label="Username">
                <Text weight="medium">{$authStore.user?.username || 'Unknown'}</Text>
              </MetadataListItem>
              <MetadataListItem label="Email">
                <Text weight="medium">{$authStore.user?.email || 'Unknown'}</Text>
              </MetadataListItem>
            </MetadataList>
          </StackItem>
        </HStack>
      </VStack>

      <Divider />

      <VStack gap={4}>
        <Heading level={2}>Appearance</Heading>
        <HStack gap={6} wrap="wrap" vAlign="center" hAlign="between">
          <StackItem size="fill">
            <VStack gap={1}>
              <Text weight="medium">Theme</Text>
              <Text type="supporting" color="secondary">
                Follow your operating system setting or pin Sunder to light or dark mode (Alt+T).
              </Text>
            </VStack>
          </StackItem>
          <SegmentedControl
            label="Theme mode"
            value={$themeStore}
            onChange={(v) => themeStore.set(v as ThemeMode)}
          >
            <SegmentedControlItem value="system" label="System" icon={systemThemeIcon} />
            <SegmentedControlItem value="light" label="Light" icon={lightThemeIcon} />
            <SegmentedControlItem value="dark" label="Dark" icon={darkThemeIcon} />
          </SegmentedControl>
        </HStack>
      </VStack>

      <Divider />

      <VStack gap={4}>
        <Heading level={2}>Currency</Heading>
        <HStack gap={6} wrap="wrap" vAlign="center" hAlign="between">
          <StackItem size="fill">
            <VStack gap={1}>
              <Text weight="medium">Default display currency</Text>
              <Text type="supporting" color="secondary">
                Totals, balances, and spending analytics convert entries to this currency using
                historical rates on each transaction's date.
              </Text>
            </VStack>
          </StackItem>
          <div class="sunder-settings-control">
            <CurrencySelect
              bind:value={selectedDefaultCurrency}
              disabled={savingCurrency}
              onchange={handleDefaultCurrencyChange}
            />
          </div>
        </HStack>
        <HStack hAlign="between" vAlign="center" wrap="wrap" gap={6}>
          <StackItem size="fill">
            <Text type="supporting" color="secondary">
              {currencyStore.currencies.length} active server currencies ·
              {currencyStore.sortedDates.length} cached exchange rate {currencyStore.sortedDates
                .length === 1
                ? 'day'
                : 'days'}
            </Text>
          </StackItem>
          <Button
            label={refreshingRates || currencyStore.syncingRates
              ? 'Refreshing...'
              : 'Refresh rates'}
            variant="secondary"
            size="sm"
            icon={refreshIcon}
            isLoading={refreshingRates || currencyStore.syncingRates}
            onclick={handleRefreshCurrencyData}
          />
        </HStack>
      </VStack>

      <Divider />

      <VStack gap={4}>
        <Heading level={2}>Sync & offline</Heading>
        <Switch
          label="Background auto-sync"
          description="Automatically back up local changes to your server in the background."
          value={syncState.autoSyncEnabled}
          onChange={handleAutoSyncToggle}
        />
        {#if syncState.autoSyncEnabled}
          <HStack gap={6} wrap="wrap" vAlign="center" hAlign="between">
            <StackItem size="fill">
              <VStack gap={1}>
                <Text weight="medium">Sync interval</Text>
                <Text type="supporting" color="secondary">
                  How often local changes are backed up while auto-sync is on.
                </Text>
              </VStack>
            </StackItem>
            <div class="sunder-settings-control">
              <Select
                value={syncState.autoSyncInterval}
                options={intervalOptions}
                label="Sync interval"
                onchange={handleIntervalChange}
              />
            </div>
          </HStack>
        {/if}
        <HStack hAlign="between" vAlign="center" wrap="wrap" gap={6}>
          <StackItem size="fill">
            <Text type="supporting" color="secondary">Last synced: {formattedLastSync}</Text>
          </StackItem>
          <Button
            label={syncing ? 'Syncing...' : 'Sync now'}
            variant="primary"
            size="sm"
            icon={refreshIcon}
            isLoading={syncing}
            onclick={handleSync}
          />
        </HStack>
      </VStack>

      <Divider />

      <VStack gap={4}>
        <Heading level={2}>Backup & import</Heading>
        <HStack hAlign="between" vAlign="center" wrap="wrap" gap={6}>
          <StackItem size="fill">
            <VStack gap={1}>
              <Text weight="medium">Export data</Text>
              <Text type="supporting" color="secondary">
                Download your full ledger as a portable Sunder JSON backup.
              </Text>
            </VStack>
          </StackItem>
          <Button
            label="Download backup"
            variant="secondary"
            size="sm"
            icon={downloadIcon}
            onclick={downloadSunderBackup}
          />
        </HStack>
        <FileInput
          label="Import Sunder backup"
          description="Restore accounts, categories, and entries from a Sunder JSON file."
          accept=".json"
          value={sunderFile}
          onChange={handleSunderImport}
        />
        <FileInput
          label="Import Bagels DB"
          description="Migrate your data from a Bagels SQLite database file."
          accept=".db,.sqlite,.sqlite3"
          value={bagelsFile}
          onChange={handleBagelsImport}
        />
      </VStack>

      <Divider />

      <VStack gap={4}>
        <Heading level={2}>Sign out</Heading>
        <HStack hAlign="between" vAlign="center" wrap="wrap" gap={6}>
          <StackItem size="fill">
            <VStack gap={1}>
              <Text weight="medium">End session on this device</Text>
              <Text type="supporting" color="secondary">
                Clears the local offline database on this browser before ending your session.
              </Text>
            </VStack>
          </StackItem>
          <Button
            label="Sign out"
            variant="destructive"
            size="sm"
            icon={logoutIcon}
            onclick={handleLogoutClick}
          />
        </HStack>
      </VStack>

      <HStack hAlign="center" paddingBlockStart={2}>
        <Text type="supporting" color="secondary">Sunder v1.0.0</Text>
      </HStack>
    </VStack>
  </LayoutContent>
{/snippet}

{#snippet logoutModalFooter()}
  <HStack gap={3} wrap="wrap" hAlign="end" width="100%">
    <Button label="Cancel" variant="ghost" onclick={() => (logoutModalOpen = false)} />
    <Button label="Discard & sign out" variant="destructive" onclick={executeLogout} />
    <Button
      label={syncing ? 'Syncing...' : 'Sync & sign out'}
      variant="primary"
      icon={refreshIcon}
      isLoading={syncing}
      onclick={handleLogoutAndSync}
    />
  </HStack>
{/snippet}

<Layout height="fill" contentWidth={800} {header} {content} />

<Modal bind:open={logoutModalOpen} title="Unsynced Changes Detected" footer={logoutModalFooter}>
  <Text>
    You have {unsyncedCount} unsynced {unsyncedCount === 1 ? 'change' : 'changes'}. If you sign out
    now without syncing, these changes will be permanently lost from this device.
  </Text>
</Modal>

<style>
  /* Reserve identical scrollbar gutter on header & scrolling content so
     centered 800px columns align to the pixel on classic-scrollbar OSes. */
  :global(.sunder-settings-header) {
    overflow-y: auto;
    scrollbar-gutter: stable;
  }

  :global(.sunder-settings-content) {
    scrollbar-gutter: stable;
  }

  .sunder-settings-control {
    width: min(100%, 260px);
    flex: 0 1 260px;
    min-width: 0;
  }
</style>
