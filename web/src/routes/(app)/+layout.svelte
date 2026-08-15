<script lang="ts">
  import '../layout.css';
  import { authStore } from '$lib/auth';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { Home, Wallet, Tags, Settings } from '@lucide/svelte';
  import favicon from '$lib/assets/favicon.svg';
  import ThemeButton from '$components/ThemeButton.svelte';
  import { Toaster } from 'svelte-sonner';

  let { children } = $props();

  $effect(() => {
    if (!$authStore.loading && !$authStore.isAuthenticated) {
      goto('/auth/sign-in');
    }
  });

  let currentPath = $derived($page.url.pathname);

  const navItems = [
    { name: 'Dashboard', href: '/dashboard', icon: Home },
    { name: 'Accounts', href: '/accounts', icon: Wallet },
    { name: 'Categories', href: '/categories', icon: Tags },
    { name: 'Settings', href: '/settings', icon: Settings }
  ];

  let lastLogoClickTime = 0;
</script>

<svelte:head>
  <title>Sunder</title>
  <link rel="icon" href={favicon} />
</svelte:head>

<Toaster richColors position="top-right" />

{#if $authStore.loading || !$authStore.isAuthenticated}
  <div class="bg-background text-foreground flex min-h-screen items-center justify-center">
    <p class="animate-pulse font-bold tracking-widest uppercase">Loading...</p>
  </div>
{:else}
  <div class="bg-background text-foreground flex h-screen overflow-hidden font-sans">
    <aside
      class="border-border bg-card flex w-16 flex-col border-r transition-all duration-300 md:w-64"
    >
      <a
        href="/dashboard"
        onclick={(e) => {
          const now = Date.now();
          if (now - lastLogoClickTime < 3000) {
            e.preventDefault();
            goto('/');
          }
          lastLogoClickTime = now;
        }}
        class="border-border group flex h-16 items-center justify-center border-b md:justify-start md:px-6"
      >
        <img
          src={favicon}
          alt="Sunder"
          class="size-6 transition-transform duration-500 group-hover:rotate-180 dark:invert"
        />
        <span class="ml-3 hidden text-lg font-semibold tracking-tight md:block">Sunder</span>
      </a>
      <nav class="flex flex-1 flex-col gap-1 overflow-y-auto px-2 py-4 md:px-3">
        {#each navItems as item}
          <a
            href={item.href}
            class="flex items-center justify-center gap-3 rounded-md border p-3 transition-all md:justify-start md:px-3 md:py-2 {currentPath.startsWith(
              item.href
            )
              ? 'bg-primary/10 text-primary border-primary/20'
              : 'text-muted-foreground hover:bg-muted hover:border-border hover:text-foreground border-transparent'}"
          >
            <svelte:component this={item.icon} size={18} class="shrink-0" />
            <span class="hidden text-sm font-medium md:block">{item.name}</span>
          </a>
        {/each}
      </nav>
      <div class="border-border flex justify-center border-t p-3 md:justify-start">
        <ThemeButton class="flex w-full items-center justify-center gap-4" />
      </div>
    </aside>

    <main class="bg-background flex-1 overflow-y-auto">
      <div class="mx-auto max-w-5xl p-6 md:p-10">
        {@render children()}
      </div>
    </main>
  </div>
{/if}
