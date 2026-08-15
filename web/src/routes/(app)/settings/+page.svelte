<script lang="ts">
  import { authStore, logout } from '$lib/auth';
  import { syncAll } from '$lib/sync';
  import { LogOut, RefreshCw } from '@lucide/svelte';

  let lastSync = $state(
    typeof window !== 'undefined'
      ? localStorage.getItem('sunder_last_sync_timestamp') || 'Never'
      : 'Never'
  );
  let syncing = $state(false);

  async function handleSync() {
    syncing = true;
    try {
      await syncAll();
      lastSync = localStorage.getItem('sunder_last_sync_timestamp') || new Date().toISOString();
    } catch (err) {
      console.error('Sync failed', err);
      alert('Sync failed. Please try again.');
    } finally {
      syncing = false;
    }
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
      <button onclick={logout} class="btn btn-destructive btn-sm gap-2">
        <LogOut size={14} /> Logout
      </button>
    </div>
  </div>

  <div class="bg-card border-border/50 space-y-4 rounded-xl border p-6 shadow-sm">
    <h2 class="border-border/50 mb-4 border-b pb-2 text-lg font-semibold tracking-tight">
      Data & Sync
    </h2>
    <p class="text-muted-foreground mb-4 text-sm font-medium">Last synced: {lastSync}</p>

    <button onclick={handleSync} disabled={syncing} class="btn btn-sm gap-2 disabled:opacity-50">
      <RefreshCw size={14} class={syncing ? 'animate-spin' : ''} />
      {syncing ? 'Syncing...' : 'Sync Now'}
    </button>
  </div>

  <div class="text-muted-foreground mt-12 text-center text-xs font-medium">
    <p>Sunder v1.0.0</p>
  </div>
</div>
