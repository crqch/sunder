<script lang="ts">
  import { onMount } from 'svelte';
  import { Button, Icon, IconButton } from '@astryx-sv/core';
  import { Monitor, Moon, Sun } from '@lucide/svelte';
  import { themeStore, type ThemeMode } from '$lib/themeStore';

  let {
    showText = false,
    size = 'sm',
    variant = 'secondary'
  }: {
    showText?: boolean;
    size?: 'sm' | 'md' | 'lg';
    variant?: 'secondary' | 'ghost';
  } = $props();

  // Defer reading persisted client mode until after hydration so SSR and initial
  // client DOM render the same icon component without corrupting SVG children.
  let mounted = $state(false);
  onMount(() => {
    mounted = true;
  });

  let displayMode = $derived<ThemeMode>(mounted ? $themeStore : 'system');

  let modeLabel = $derived(
    displayMode === 'dark' ? 'Dark' : displayMode === 'light' ? 'Light' : 'System'
  );

  let tooltipLabel = $derived(
    displayMode === 'system'
      ? 'Theme: System (click for light · Alt+T)'
      : displayMode === 'light'
        ? 'Theme: Light (click for dark · Alt+T)'
        : 'Theme: Dark (click for system · Alt+T)'
  );
</script>

{#snippet modeIcon()}
  {#if displayMode === 'dark'}
    <Icon icon={Moon} size="sm" />
  {:else if displayMode === 'light'}
    <Icon icon={Sun} size="sm" />
  {:else}
    <Icon icon={Monitor} size="sm" />
  {/if}
{/snippet}

{#if showText}
  <Button
    label={modeLabel}
    {variant}
    {size}
    icon={modeIcon}
    tooltip={tooltipLabel}
    onclick={(e) => themeStore.toggle(e)}
  />
{:else}
  <IconButton
    label={tooltipLabel}
    {variant}
    {size}
    icon={modeIcon}
    tooltip={tooltipLabel}
    onclick={(e) => themeStore.toggle(e)}
  />
{/if}
