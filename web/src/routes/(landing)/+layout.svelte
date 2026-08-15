<script lang="ts">
  import '../layout.css';
  import favicon from '$lib/assets/favicon.svg';
  import Navbar from '$components/Navbar.svelte';
  import Footer from '$components/Footer.svelte';

  let { children } = $props();
</script>

<svelte:head>
  <title>Sunder - Local-First Personal Finance & Expense Sync</title>
  <link rel="icon" href={favicon} />
</svelte:head>

<div class="flex min-h-screen flex-col">
  <Navbar />
  <main class="flex w-full flex-1 flex-col items-center">
    {@render children()}
  </main>
  <Footer />
</div>
