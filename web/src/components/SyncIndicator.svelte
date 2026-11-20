<script lang="ts">
  import { onMount } from 'svelte';
  import { cubicOut } from 'svelte/easing';
  import { prefersReducedMotion } from 'svelte/motion';
  import { fade, scale, slide } from 'svelte/transition';
  import {
    AlertDialog,
    Badge,
    Banner,
    Button,
    HStack,
    Heading,
    Icon,
    IconButton,
    List,
    ListItem,
    Spinner,
    StatusDot,
    Text,
    VStack
  } from '@astryx-sv/core';
  import { ChevronDown, Maximize2, Minimize2, RefreshCw } from '@lucide/svelte';
  import { getUnsyncedChanges, syncAll, clearLocalDatabase } from '$lib/sync';
  import { syncState } from '$lib/syncState.svelte';

  let {
    mode = 'banner',
    compact = false
  }: { mode?: 'poller' | 'banner' | 'sidebar'; compact?: boolean } = $props();

  let showDetails = $state(false);
  let discardConfirmOpen = $state(false);
  let lastAutoSyncTime = Date.now();
  const detailsId = $props.id();

  // Svelte skips `in:` transitions for blocks created during a component's first
  // render, and this component remounts every time the banner slot appears — so
  // the shell's intro never played. Flipping a post-mount flag creates the shell
  // through a real state change instead, which lets the slide intro run.
  let bannerEntered = $state(false);
  $effect(() => {
    bannerEntered = true;
  });

  /** Svelte transitions get a zero clock when the user asks for less motion. */
  function motionMs(duration: number): number {
    return prefersReducedMotion.current ? 0 : duration;
  }

  let unsyncedCount = $derived(syncState.pendingCount);
  let unsyncedData = $derived(syncState.unsyncedData);
  let isPillVisible = $derived(syncState.minimized && syncState.status !== 'idle');

  function toggleMinimize() {
    syncState.minimized = !syncState.minimized;
    localStorage.setItem('sunder_sync_minimized', syncState.minimized.toString());
  }

  // Periodically check for unsynced changes and run auto-sync
  onMount(() => {
    // Only run the polling interval in the dedicated poller instance so we don't
    // duplicate work and polling stays active even when the banner slot unmounts.
    if (mode !== 'poller') return;

    const poll = async () => {
      if (syncState.status === 'syncing') return;
      try {
        const changes = await getUnsyncedChanges();
        syncState.unsyncedData = changes;
        const count = changes.accounts.length + changes.categories.length + changes.entries.length;
        syncState.pendingCount = count;

        if (count > 0 && syncState.status === 'idle') {
          syncState.status = 'unsynced';
        } else if (count === 0 && syncState.status === 'unsynced') {
          syncState.status = 'idle';
          showDetails = false;
        }

        if (syncState.autoSyncEnabled && count > 0) {
          const now = Date.now();
          if (now - lastAutoSyncTime >= syncState.autoSyncInterval * 60 * 1000) {
            lastAutoSyncTime = now;
            await handleManualSync();
          }
        }
      } catch {
        // ignore transient IndexedDB reads
      }
    };

    poll();
    const interval = setInterval(poll, 2000);
    return () => clearInterval(interval);
  });

  async function handleManualSync() {
    if (syncState.status === 'syncing') return;

    syncState.status = 'syncing';
    try {
      await syncAll();
      syncState.status = 'success';
      syncState.pendingCount = 0;
      syncState.unsyncedData = { accounts: [], categories: [], entries: [] };
      syncState.lastSyncedAt =
        localStorage.getItem('sunder_last_sync_timestamp') || new Date().toISOString();
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
    syncState.status = 'syncing';
    try {
      await clearLocalDatabase();
      await syncAll();
      syncState.status = 'idle';
      syncState.pendingCount = 0;
      syncState.unsyncedData = { accounts: [], categories: [], entries: [] };
      syncState.lastSyncedAt =
        localStorage.getItem('sunder_last_sync_timestamp') || new Date().toISOString();
      showDetails = false;
    } catch (e) {
      console.error(e);
      syncState.status = 'error';
    }
    discardConfirmOpen = false;
  }

  function lastSyncTimestamp(): string {
    if (typeof window === 'undefined') return '1970-01-01';
    return (
      syncState.lastSyncedAt || localStorage.getItem('sunder_last_sync_timestamp') || '1970-01-01'
    );
  }

  let statusLabelText = $derived(
    syncState.status === 'syncing'
      ? 'Syncing...'
      : syncState.status === 'success'
        ? 'Synced'
        : syncState.status === 'error'
          ? 'Sync failed'
          : 'Unsynced'
  );

  let bannerStatus = $derived<'info' | 'success' | 'error' | 'warning'>(
    syncState.status === 'syncing'
      ? 'info'
      : syncState.status === 'success'
        ? 'success'
        : syncState.status === 'error'
          ? 'error'
          : 'warning'
  );

  let bannerTitle = $derived(
    syncState.status === 'syncing'
      ? 'Syncing your data...'
      : syncState.status === 'success'
        ? 'All changes synced'
        : syncState.status === 'error'
          ? 'Sync failed — check your connection or retry'
          : `${unsyncedCount} unsynced ${unsyncedCount === 1 ? 'change' : 'changes'}`
  );
</script>

{#snippet statusMark()}
  <span class="sunder-sync-mark" data-status={syncState.status}>
    {#key syncState.status}
      <span
        class="sunder-sync-mark-swap"
        in:scale={{ duration: motionMs(180), start: 0.65, easing: cubicOut }}
        out:fade={{ duration: motionMs(110) }}
      >
        {#if syncState.status === 'syncing'}
          <Spinner size="sm" />
        {:else if syncState.status === 'success'}
          <StatusDot variant="success" label="Synced" />
        {:else if syncState.status === 'error'}
          <StatusDot variant="error" label="Sync failed" />
        {:else if syncState.status === 'unsynced'}
          <StatusDot variant="warning" label="Unsynced changes" isPulsing />
        {:else}
          <StatusDot variant="success" label="All changes synced" />
        {/if}
      </span>
    {/key}
  </span>
{/snippet}

{#snippet syncIcon()}
  <Icon icon={RefreshCw} size="sm" />
{/snippet}

{#snippet expandIcon()}
  <Icon icon={Maximize2} size="sm" />
{/snippet}

{#snippet minimizeIcon()}
  <Icon icon={Minimize2} size="sm" />
{/snippet}

{#snippet chevronIcon()}
  <span class="sunder-sync-chevron" data-open={showDetails ? '' : undefined}>
    <Icon icon={ChevronDown} size="sm" />
  </span>
{/snippet}

{#snippet bannerActions()}
  <HStack class="sunder-sync-actions" gap={2} vAlign="center">
    {#if syncState.status === 'unsynced' || syncState.status === 'error'}
      <Button
        class="sunder-sync-action"
        label={syncState.status === 'error' ? 'Retry sync' : 'Sync now'}
        variant="primary"
        size="sm"
        icon={syncIcon}
        onclick={handleManualSync}
      />
    {/if}
    <IconButton
      class="sunder-sync-minimize"
      label="Minimize sync banner"
      variant="ghost"
      size="sm"
      icon={minimizeIcon}
      tooltip="Minimize to sidebar"
      onclick={toggleMinimize}
    />
    {#if syncState.status === 'unsynced'}
      <IconButton
        class="sunder-sync-toggle"
        label={showDetails ? 'Collapse pending changes' : 'Expand pending changes'}
        variant="ghost"
        size="sm"
        icon={chevronIcon}
        tooltip={showDetails ? 'Collapse pending changes' : 'Expand pending changes'}
        aria-expanded={showDetails}
        aria-controls={showDetails ? detailsId : undefined}
        onclick={() => (showDetails = !showDetails)}
      />
    {/if}
  </HStack>
{/snippet}

{#snippet changeItem(item: any, labelField: string, baseHref: string)}
  {@const isDeleted = !!item.deleted_at}
  {@const isCreated = item.created_at > lastSyncTimestamp()}
  {#snippet changeBadge()}
    {#if isDeleted}
      <Badge variant="error" label="Deleted" />
    {:else if isCreated}
      <Badge variant="success" label="Created" />
    {:else}
      <Badge variant="warning" label="Updated" />
    {/if}
  {/snippet}
  {#snippet itemLabel()}
    {#if isDeleted}
      <Text color="secondary" hasStrikethrough>{item[labelField]}</Text>
    {:else}
      {item[labelField]}
    {/if}
  {/snippet}
  <ListItem
    label={itemLabel}
    startContent={changeBadge}
    href={isDeleted ? undefined : `${baseHref}/${item.id}`}
    onclick={() => {
      showDetails = false;
    }}
  />
{/snippet}

{#snippet pendingDetails()}
  <VStack gap={4}>
    <HStack hAlign="between" vAlign="center" gap={3}>
      <Heading level={4}>Pending changes</Heading>
      <Button
        class="sunder-sync-discard"
        label="Discard all"
        variant="destructive"
        size="sm"
        onclick={() => (discardConfirmOpen = true)}
      />
    </HStack>

    {#if unsyncedData.accounts.length > 0}
      <VStack gap={1}>
        <Text type="supporting" color="secondary" weight="semibold">Accounts</Text>
        <List density="compact" edge="flush">
          {#each unsyncedData.accounts as item (item.id)}
            {@render changeItem(item, 'name', '/accounts')}
          {/each}
        </List>
      </VStack>
    {/if}
    {#if unsyncedData.categories.length > 0}
      <VStack gap={1}>
        <Text type="supporting" color="secondary" weight="semibold">Categories</Text>
        <List density="compact" edge="flush">
          {#each unsyncedData.categories as item (item.id)}
            {@render changeItem(item, 'title', '/categories')}
          {/each}
        </List>
      </VStack>
    {/if}
    {#if unsyncedData.entries.length > 0}
      <VStack gap={1}>
        <Text type="supporting" color="secondary" weight="semibold">Entries</Text>
        <List density="compact" edge="flush">
          {#each unsyncedData.entries as item (item.id)}
            {@render changeItem(item, 'title', '/entries')}
          {/each}
        </List>
      </VStack>
    {/if}
  </VStack>
{/snippet}

{#if mode === 'poller'}
  <!-- Headless sync state poller; renders no DOM or landmark. -->
{:else if mode === 'sidebar'}
  {#if compact}
    {#if isPillVisible}
      <div
        class="sunder-sync-compact"
        data-status={syncState.status}
        in:fade={{ duration: motionMs(180) }}
        out:fade={{ duration: motionMs(120) }}
      >
        <IconButton
          class="sunder-sync-compact-button"
          label={statusLabelText}
          variant="ghost"
          size="sm"
          icon={statusMark}
          tooltip="Expand sync banner"
          onclick={toggleMinimize}
        />
      </div>
    {/if}
  {:else if isPillVisible}
    <div
      class="sunder-sync-pill"
      data-status={syncState.status}
      in:slide={{ duration: motionMs(240), easing: cubicOut }}
      out:slide={{ duration: motionMs(180), easing: cubicOut }}
    >
      <div
        class="sunder-sync-pill-inner"
        in:fade={{ duration: motionMs(180), delay: motionMs(30) }}
        out:fade={{ duration: motionMs(120) }}
      >
        <HStack class="sunder-sync-pill-row" gap={1} vAlign="center" hAlign="between" width="100%">
          <Button
            class="sunder-sync-pill-button"
            label={statusLabelText}
            variant="ghost"
            size="sm"
            icon={statusMark}
            tooltip="Click to sync now"
            onclick={handleManualSync}
          />
          <IconButton
            class="sunder-sync-pill-expand"
            label="Expand sync banner"
            variant="ghost"
            size="sm"
            icon={expandIcon}
            tooltip="Expand sync banner"
            onclick={toggleMinimize}
          />
        </HStack>
      </div>
    </div>
  {/if}
{:else if bannerEntered && !syncState.minimized && syncState.status !== 'idle'}
  <div
    class="sunder-sync-banner-shell"
    in:slide={{ duration: motionMs(260), easing: cubicOut }}
    out:slide={{ duration: motionMs(180), easing: cubicOut }}
  >
    <div
      class="sunder-sync-banner-inner"
      in:fade={{ duration: motionMs(200), delay: motionMs(30) }}
      out:fade={{ duration: motionMs(120) }}
    >
      <Banner
        class="sunder-sync-banner"
        container="section"
        status={bannerStatus}
        title={bannerTitle}
        icon={statusMark}
        endContent={bannerActions}
        collapsible={false}
      />
      <!--
        The disclosure region lives outside Banner so its padding and borders
        ride the same slide transition as its height. Inside Banner's content
        wrapper they popped in/out instantly at the ends of the slide (jitter).
      -->
      {#if syncState.status === 'unsynced' && showDetails}
        <div
          class="sunder-sync-details"
          id={detailsId}
          in:slide={{ duration: motionMs(220), easing: cubicOut }}
          out:slide={{ duration: motionMs(160), easing: cubicOut }}
        >
          {@render pendingDetails()}
        </div>
      {/if}
    </div>
  </div>
{/if}

{#if mode === 'banner'}
  <AlertDialog
    isOpen={discardConfirmOpen}
    onOpenChange={(next) => (discardConfirmOpen = next)}
    title="Discard all unsynced changes?"
    description="This permanently deletes every local change that has not been synced, then pulls fresh data from the server. This cannot be undone."
    actionLabel="Discard all"
    actionVariant="destructive"
    onAction={handleDiscardAll}
  />
{/if}
