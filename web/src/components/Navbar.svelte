<script lang="ts">
  import { Button, HStack, TopNav, TopNavHeading, TopNavItem } from '@astryx-sv/core';
  import BrandMark from '$components/BrandMark.svelte';
  import ThemeButton from '$components/ThemeButton.svelte';
  import { authStore } from '$lib/auth';
</script>

{#snippet brandLogo()}
  <BrandMark size={24} />
{/snippet}

{#snippet brand()}
  <TopNavHeading heading="Sunder" headingHref="/" logo={brandLogo} />
{/snippet}

{#snippet navLinks()}
  <TopNavItem label="Features" href="/#features" />
  <TopNavItem label="Platforms" href="/#experience" />
{/snippet}

{#snippet actions()}
  <HStack gap={2} vAlign="center">
    <ThemeButton size="sm" variant="secondary" />
    {#if $authStore.isAuthenticated && $authStore.user}
      <Button label={$authStore.user.username} variant="primary" size="sm" href="/dashboard" />
    {:else}
      <Button label="Sign in" variant="secondary" size="sm" href="/auth/sign-in" />
      <Button label="Get started" variant="primary" size="sm" href="/auth/sign-up" />
    {/if}
  </HStack>
{/snippet}

<TopNav label="Main navigation" heading={brand} startContent={navLinks} endContent={actions} />
