<script lang="ts">
  import { AppShell } from '@astryx-sv/core';
  import favicon from '$lib/assets/favicon.svg';
  import Navbar from '$components/Navbar.svelte';
  import Footer from '$components/Footer.svelte';

  let { children } = $props();
</script>

<svelte:head>
  <title>Sunder - Local-First Personal Finance & Expense Sync</title>
  <link rel="icon" href={favicon} />
</svelte:head>

<AppShell height="auto" contentPadding={0}>
  {#snippet topNav()}
    <Navbar />
  {/snippet}

  {@render children()}

  <Footer />
</AppShell>
