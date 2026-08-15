<script lang="ts">
  import { onMount } from 'svelte';
  import { fly, slide } from 'svelte/transition';
  import {
    RefreshCw,
    CheckCircle2,
    AlertCircle,
    CircleDashed,
    ChevronDown,
    ChevronUp,
    Save,
    Maximize2
  } from '@lucide/svelte';
  import { getUnsyncedChanges, syncAll, clearLocalDatabase } from '$lib/sync';
  import { syncState } from '$lib/syncState.svelte';

  let { mode = 'bar' } = $props<{ mode?: 'bar' | 'sidebar' }>();

  let unsyncedCount = $state(0);
  let unsyncedData = $state<{ accounts: any[]; categories: any[]; entries: any[] }>({
    accounts: [],
    categories: [],
    entries: []
  });
  let showDetails = $state(false);
  let lastAutoSyncTime = Date.now();

  function toggleMinimize() {
    syncState.minimized = !syncState.minimized;
    localStorage.setItem('sunder_sync_minimized', syncState.minimized.toString());
  }

  // Periodically check for unsynced changes and run auto-sync
  onMount(() => {
    // Only run the interval in the bar instance so we don't duplicate work
    if (mode === 'sidebar') return;

    const interval = setInterval(async () => {
      if (syncState.status === 'syncing') return;
      try {
        const changes = await getUnsyncedChanges();
        unsyncedData = changes;
        unsyncedCount =
          changes.accounts.length + changes.categories.length + changes.entries.length;

        if (unsyncedCount > 0 && syncState.status === 'idle') {
          syncState.status = 'unsynced';
        } else if (unsyncedCount === 0 && syncState.status === 'unsynced') {
          syncState.status = 'idle';
          showDetails = false;
        }

        // Auto-sync logic
        if (syncState.autoSyncEnabled && unsyncedCount > 0) {
          const now = Date.now();
          if (now - lastAutoSyncTime >= syncState.autoSyncInterval * 60 * 1000) {
            lastAutoSyncTime = now;
            await handleManualSync();
          }
        }
      } catch (e) {
        // ignore
      }
    }, 2000);

    return () => clearInterval(interval);
  });

  async function handleManualSync() {
    if (syncState.status === 'syncing') return;

    syncState.status = 'syncing';
    try {
      await syncAll();
      syncState.status = 'success';
      unsyncedCount = 0;
      showDetails = false;
      setTimeout(() => {
        if (syncState.status === 'success') {
          syncState.status = 'idle';
        }
      }, 3000);
    } catch (e) {
      console.error(e);
      syncState.status = 'error';
    }
  }

  async function handleDiscardAll() {
    if (confirm('Are you sure you want to discard all unsynced changes? This cannot be undone.')) {
      syncState.status = 'syncing';
      try {
        await clearLocalDatabase();
        await syncAll(); // pulls fresh data
        syncState.status = 'idle';
        unsyncedCount = 0;
        showDetails = false;
      } catch (e) {
        console.error(e);
        syncState.status = 'error';
      }
    }
  }
</script>

{#if mode === 'sidebar'}
  {#if syncState.minimized && syncState.status !== 'idle'}
    <div transition:slide={{ duration: 250 }} class="flex items-center gap-2 pb-2">
      <button
        type="button"
        onclick={handleManualSync}
        class="focus-visible:ring-primary flex flex-1 items-center justify-center gap-2 rounded-md p-2 text-sm font-medium transition-colors focus:outline-none focus-visible:ring-2
          {syncState.status === 'error'
          ? 'text-destructive hover:bg-destructive/10'
          : syncState.status === 'success'
            ? 'text-emerald-500 hover:bg-emerald-500/10'
            : syncState.status === 'unsynced'
              ? 'text-amber-500 hover:bg-amber-500/10'
              : 'text-muted-foreground hover:bg-muted'}"
        title="Sync Status"
      >
        {#if syncState.status === 'syncing'}
          <RefreshCw size={16} class="text-primary animate-spin" />
          <span>Syncing...</span>
        {:else if syncState.status === 'success'}
          <CheckCircle2 size={16} />
          <span>Synced!</span>
        {:else if syncState.status === 'error'}
          <AlertCircle size={16} />
          <span>Failed</span>
        {:else if syncState.status === 'unsynced'}
          <div class="relative flex h-4 w-4 items-center justify-center">
            <div
              class="absolute h-full w-full animate-ping rounded-full bg-amber-500 opacity-20"
            ></div>
            <CircleDashed size={16} class="relative z-10 text-amber-500" />
          </div>
          <span>Unsynced</span>
        {/if}
      </button>
      <button
        onclick={toggleMinimize}
        class="text-muted-foreground hover:bg-muted rounded-md p-2 transition-colors"
        title="Expand Sync Bar"
      >
        <Maximize2 size={16} />
      </button>
    </div>
  {/if}
{:else}
  {#if !syncState.minimized && syncState.status !== 'idle'}
    <div
      transition:fly={{ y: 50, duration: 300 }}
      class="bg-card border-border/50 fixed right-0 bottom-0 left-0 z-50 flex flex-col border-t shadow-[0_-4px_10px_rgba(0,0,0,0.05)] md:left-64"
    >
      <!-- Header Bar -->
      <div class="flex h-14 items-center justify-between px-6">
        <div
          class="flex cursor-pointer items-center gap-3 select-none"
          onclick={() => {
            if (syncState.status === 'unsynced') showDetails = !showDetails;
          }}
        >
          {#if syncState.status === 'syncing'}
            <RefreshCw size={18} class="text-primary animate-spin" />
            <span class="font-medium">Syncing your data...</span>
          {:else if syncState.status === 'success'}
            <CheckCircle2 size={18} class="text-emerald-500" />
            <span class="font-medium text-emerald-500">All changes synced!</span>
          {:else if syncState.status === 'error'}
            <AlertCircle size={18} class="text-destructive" />
            <span class="text-destructive font-medium">Sync Failed. Retrying soon...</span>
          {:else if syncState.status === 'unsynced'}
            <div class="relative flex h-5 w-5 items-center justify-center">
              <div
                class="absolute h-full w-full animate-ping rounded-full bg-amber-500 opacity-20"
              ></div>
              <CircleDashed size={18} class="relative z-10 text-amber-500" />
            </div>
            <span class="text-foreground font-medium">
              {unsyncedCount} unsynced {unsyncedCount === 1 ? 'change' : 'changes'}
            </span>
            <ChevronUp
              size={16}
              class="text-muted-foreground transition-transform duration-200 {showDetails
                ? 'rotate-180'
                : ''}"
            />
          {/if}
        </div>

        <div class="flex items-center gap-2">
          {#if syncState.status === 'unsynced'}
            <button onclick={handleManualSync} class="btn btn-sm gap-2">
              <Save size={16} /> Sync Now
            </button>
          {/if}

          <button
            onclick={toggleMinimize}
            class="text-muted-foreground hover:bg-muted rounded-md p-1.5 transition-colors"
            title="Minimize to sidebar"
          >
            <ChevronDown size={16} />
          </button>
        </div>
      </div>

      <!-- Details Panel -->
      {#if showDetails && syncState.status === 'unsynced'}
        <div
          transition:slide={{ duration: 250 }}
          class="border-border/50 bg-muted/30 max-h-64 overflow-y-auto border-t px-6 py-4"
        >
          <div class="mb-4 flex items-center justify-between">
            <h4 class="text-sm font-semibold">Pending Changes</h4>
            <button
              onclick={handleDiscardAll}
              class="btn-outline border-destructive/50 hover:bg-destructive/10 text-destructive btn-sm"
            >
              Discard All
            </button>
          </div>
          <div class="space-y-3">
            {#snippet changeItem(item: any, labelField: string)}
              {@const isDeleted = !!item.deleted_at}
              {@const lastSync = localStorage.getItem('sunder_last_sync_timestamp') || '1970-01-01'}
              {@const isCreated = item.created_at > lastSync}
              <li class="flex items-center gap-2">
                {#if isDeleted}
                  <span
                    class="text-destructive bg-destructive/10 rounded px-1.5 py-0.5 text-[10px] font-bold tracking-wider uppercase"
                    >Deleted</span
                  >
                  <span class="text-muted-foreground line-through">{item[labelField]}</span>
                {:else if isCreated}
                  <span
                    class="rounded bg-emerald-500/10 px-1.5 py-0.5 text-[10px] font-bold tracking-wider text-emerald-500 uppercase"
                    >Created</span
                  >
                  <span>{item[labelField]}</span>
                {:else}
                  <span
                    class="rounded bg-amber-500/10 px-1.5 py-0.5 text-[10px] font-bold tracking-wider text-amber-500 uppercase"
                    >Updated</span
                  >
                  <span>{item[labelField]}</span>
                {/if}
              </li>
            {/snippet}

            {#if unsyncedData.accounts.length > 0}
              <div>
                <p class="text-muted-foreground mb-2 text-xs font-medium tracking-wider uppercase">
                  Accounts
                </p>
                <ul class="space-y-1.5 text-sm">
                  {#each unsyncedData.accounts as item}
                    {@render changeItem(item, 'name')}
                  {/each}
                </ul>
              </div>
            {/if}
            {#if unsyncedData.categories.length > 0}
              <div>
                <p class="text-muted-foreground mb-2 text-xs font-medium tracking-wider uppercase">
                  Categories
                </p>
                <ul class="space-y-1.5 text-sm">
                  {#each unsyncedData.categories as item}
                    {@render changeItem(item, 'title')}
                  {/each}
                </ul>
              </div>
            {/if}
            {#if unsyncedData.entries.length > 0}
              <div>
                <p class="text-muted-foreground mb-2 text-xs font-medium tracking-wider uppercase">
                  Entries
                </p>
                <ul class="space-y-1.5 text-sm">
                  {#each unsyncedData.entries as item}
                    {@render changeItem(item, 'title')}
                  {/each}
                </ul>
              </div>
            {/if}
          </div>
        </div>
      {/if}
    </div>
  {/if}
{/if}
