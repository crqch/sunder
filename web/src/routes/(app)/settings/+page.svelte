<script lang="ts">
  import { authStore, logout } from '$lib/auth';
  import { syncAll, getUnsyncedChanges, clearLocalDatabase } from '$lib/sync';
  import { syncState } from '$lib/syncState.svelte';
  import { LogOut, RefreshCw } from '@lucide/svelte';
  import { toast } from 'svelte-sonner';
  import Modal from '$components/Modal.svelte';

  let lastSync = $state(
    typeof window !== 'undefined'
      ? localStorage.getItem('sunder_last_sync_timestamp') || 'Never'
      : 'Never'
  );
  let syncing = $state(false);

  let logoutModalOpen = $state(false);
  let unsyncedCount = $state(0);

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
      // Fallback: just try to logout anyway if something is catastrophically broken
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

  function handleAutoSyncToggle(e: Event) {
    const checked = (e.target as HTMLInputElement).checked;
    syncState.autoSyncEnabled = checked;
    localStorage.setItem('sunder_auto_sync', checked.toString());
  }

  function handleIntervalChange(e: Event) {
    const val = parseInt((e.target as HTMLSelectElement).value);
    syncState.autoSyncInterval = val;
    localStorage.setItem('sunder_auto_sync_interval', val.toString());
  }
</script>

<div class="space-y-8 font-sans">
  <div class="border-border/50 flex items-center justify-between border-b pb-4">
    <h1 class="text-2xl font-semibold tracking-tight">Settings</h1>
  </div>

  <div class="bg-card border-border/50 space-y-4 rounded-xl border p-6 shadow-sm">
    <h2 class="border-border/50 mb-4 border-b pb-2 text-lg font-semibold tracking-tight">
      Account
    </h2>
    <div class="grid grid-cols-1 gap-6 sm:grid-cols-2">
      <div>
        <p class="text-muted-foreground text-sm font-medium">Username</p>
        <p class="text-lg font-medium">{$authStore.user?.username || 'Unknown'}</p>
      </div>
      <div>
        <p class="text-muted-foreground text-sm font-medium">Email</p>
        <p class="text-lg font-medium">{$authStore.user?.email || 'Unknown'}</p>
      </div>
    </div>

    <div class="pt-4">
      <button onclick={handleLogoutClick} class="btn btn-destructive btn-sm gap-2">
        <LogOut size={14} /> Logout
      </button>
    </div>
  </div>

  <div class="bg-card border-border/50 space-y-6 rounded-xl border p-6 shadow-sm">
    <h2 class="border-border/50 border-b pb-2 text-lg font-semibold tracking-tight">Data & Sync</h2>

    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <h3 class="font-medium">Background Auto-Sync</h3>
          <p class="text-muted-foreground text-sm">Automatically back up your changes.</p>
        </div>
        <label class="relative inline-flex cursor-pointer items-center">
          <input
            type="checkbox"
            checked={syncState.autoSyncEnabled}
            onchange={handleAutoSyncToggle}
            class="peer sr-only"
          />
          <div
            class="peer bg-muted peer-focus:ring-primary/50 peer-checked:bg-primary h-6 w-11 rounded-full peer-focus:ring-2 peer-focus:outline-none after:absolute after:top-[2px] after:left-[2px] after:h-5 after:w-5 after:rounded-full after:border after:border-gray-300 after:bg-white after:transition-all after:content-[''] peer-checked:after:translate-x-full peer-checked:after:border-white"
          ></div>
        </label>
      </div>

      {#if syncState.autoSyncEnabled}
        <div class="flex items-center justify-between">
          <label for="sync-interval" class="text-muted-foreground text-sm font-medium"
            >Sync Interval</label
          >
          <select
            id="sync-interval"
            value={syncState.autoSyncInterval}
            onchange={handleIntervalChange}
            class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary rounded-md border p-1 text-sm focus:ring-2 focus:outline-none"
          >
            <option value={1}>Every 1 minute</option>
            <option value={5}>Every 5 minutes</option>
            <option value={15}>Every 15 minutes</option>
            <option value={60}>Every 1 hour</option>
          </select>
        </div>
      {/if}
    </div>

    <div class="border-border/50 border-t pt-4">
      <p class="text-muted-foreground mb-4 text-sm font-medium">Last synced: {lastSync}</p>
      <button onclick={handleSync} disabled={syncing} class="btn btn-sm gap-2 disabled:opacity-50">
        <RefreshCw size={14} class={syncing ? 'animate-spin' : ''} />
        {syncing ? 'Syncing...' : 'Sync Now'}
      </button>
    </div>
  </div>

  <div class="text-muted-foreground mt-12 text-center text-xs font-medium">
    <p>Sunder v1.0.0</p>
  </div>
</div>

<Modal bind:open={logoutModalOpen} title="Unsynced Changes Detected">
  <div class="space-y-4">
    <p class="text-sm">
      You have <strong class="text-foreground font-semibold"
        >{unsyncedCount} unsynced changes</strong
      >. If you log out now without syncing, these changes will be permanently lost from your
      device.
    </p>
    <div class="flex flex-col gap-2 pt-2 sm:flex-row">
      <button onclick={handleLogoutAndSync} disabled={syncing} class="btn flex-1 gap-2">
        {#if syncing}
          <RefreshCw size={14} class="animate-spin" /> Syncing...
        {:else}
          <RefreshCw size={14} /> Sync & Logout
        {/if}
      </button>
      <button
        onclick={executeLogout}
        class="btn-outline border-destructive/50 hover:bg-destructive/10 text-destructive flex-1"
      >
        Discard & Logout
      </button>
    </div>
    <button
      onclick={() => (logoutModalOpen = false)}
      class="btn-outline text-muted-foreground w-full"
    >
      Cancel
    </button>
  </div>
</Modal>
