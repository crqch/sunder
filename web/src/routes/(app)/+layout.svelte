<script lang="ts">
  import '../layout.css';
  import { authStore } from '$lib/auth';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { Home, Wallet, Tags, Settings, List, Menu, X, ShieldCheck } from '@lucide/svelte';
  import favicon from '$lib/assets/favicon.svg';
  import ThemeButton from '$components/ThemeButton.svelte';
  import SyncIndicator from '$components/SyncIndicator.svelte';
  import { themeStore } from '$lib/themeStore';
  import { Toaster } from 'svelte-sonner';

  let { children } = $props();

  $effect(() => {
    if (!$authStore.loading && !$authStore.isAuthenticated) {
      goto('/auth/sign-in');
    }
  });

  let currentPath = $derived($page.url.pathname);

  let navItems = $derived.by(() => {
    const items = [
      { name: 'Dashboard', href: '/dashboard', icon: Home },
      { name: 'Entries', href: '/entries', icon: List },
      { name: 'Accounts', href: '/accounts', icon: Wallet },
      { name: 'Categories', href: '/categories', icon: Tags },
      { name: 'Settings', href: '/settings', icon: Settings }
    ];
    if ($authStore.user?.flags?.includes('is_admin')) {
      items.push({ name: 'Admin', href: '/admin', icon: ShieldCheck });
    }
    return items;
  });

  let lastLogoClickTime = 0;

  import AccountForm from '$components/AccountForm.svelte';
  import CategoryForm from '$components/CategoryForm.svelte';
  import EntryForm from '$components/EntryForm.svelte';
  import Modal from '$components/Modal.svelte';
  import { keybinds } from '$lib/keybinds.svelte';
  import { tooltip } from '$lib/tooltip';
  import { modals } from '$lib/modals.svelte';

  let accountDirty = $state(false);
  let categoryDirty = $state(false);
  let entryDirty = $state(false);
  let sidebarOpen = $state(false);

  $effect(() => {
    let unregs = [
      keybinds.register({
        id: 'app.new_account',
        name: 'New Account',
        keys: ['alt+n', 'a'],
        global: false,
        action: () => {
          if (modals.createAccount && !accountDirty) modals.createAccount = false;
          else modals.createAccount = true;
        }
      }),
      keybinds.register({
        id: 'app.new_category',
        name: 'New Category',
        keys: ['alt+n', 'c'],
        global: false,
        action: () => {
          if (modals.createCategory && !categoryDirty) modals.createCategory = false;
          else modals.createCategory = true;
        }
      }),
      keybinds.register({
        id: 'app.new_entry_space',
        name: 'New Entry',
        keys: ['alt+space'],
        global: false,
        action: () => {
          if (modals.createEntry && !entryDirty) modals.createEntry = false;
          else modals.createEntry = true;
        }
      }),
      keybinds.register({
        id: 'app.new_entry',
        name: 'New Entry',
        keys: ['alt+n', 'e'],
        global: false,
        action: () => {
          if (modals.createEntry && !entryDirty) modals.createEntry = false;
          else modals.createEntry = true;
        }
      }),
      keybinds.register({
        id: 'app.tab1',
        name: 'Dashboard Tab',
        keys: ['alt+1'],
        global: false,
        action: () => goto(navItems[0].href)
      }),
      keybinds.register({
        id: 'app.tab2',
        name: 'Entries Tab',
        keys: ['alt+2'],
        global: false,
        action: () => goto(navItems[1].href)
      }),
      keybinds.register({
        id: 'app.tab_entries_alt',
        name: 'Entries Tab',
        keys: ['alt+e'],
        global: false,
        action: () => goto(navItems[1].href)
      }),
      keybinds.register({
        id: 'app.tab3',
        name: 'Accounts Tab',
        keys: ['alt+3'],
        global: false,
        action: () => goto(navItems[2].href)
      }),
      keybinds.register({
        id: 'app.tab_accounts_alt',
        name: 'Accounts Tab',
        keys: ['alt+a'],
        global: false,
        action: () => goto(navItems[2].href)
      }),
      keybinds.register({
        id: 'app.tab4',
        name: 'Categories Tab',
        keys: ['alt+4'],
        global: false,
        action: () => goto(navItems[3].href)
      }),
      keybinds.register({
        id: 'app.tab_categories_alt',
        name: 'Categories Tab',
        keys: ['alt+c'],
        global: false,
        action: () => goto(navItems[3].href)
      }),
      keybinds.register({
        id: 'app.tab5',
        name: 'Settings Tab',
        keys: ['alt+5'],
        global: false,
        action: () => goto(navItems[4].href)
      })
    ];
    return () => unregs.forEach((u) => u());
  });
</script>

<svelte:head>
  <title>Sunder</title>
  <link rel="icon" href={favicon} />
</svelte:head>

<Toaster richColors position="top-right" theme={$themeStore} />

{#if $authStore.loading || !$authStore.isAuthenticated}
  <div class="bg-background text-foreground flex min-h-screen items-center justify-center">
    <p class="animate-pulse font-bold tracking-widest uppercase">Loading...</p>
  </div>
{:else}
  <div class="bg-background text-foreground relative flex h-screen overflow-hidden font-sans">
    <aside
      class="border-border bg-card fixed inset-y-0 left-0 z-50 flex
             w-full flex-col border-r transition-all duration-300 md:relative md:w-64
             {sidebarOpen ? 'translate-x-0' : '-translate-x-full md:translate-x-0'}"
    >
      <div class="border-border flex h-16 items-center justify-between border-b px-6">
        <a
          href="/dashboard"
          onclick={(e) => {
            const now = Date.now();
            if (now - lastLogoClickTime < 3000) {
              e.preventDefault();
              goto('/');
            }
            lastLogoClickTime = now;
            sidebarOpen = false;
          }}
          class="group flex items-center"
        >
          <img
            src={favicon}
            alt="Sunder"
            class="size-6 transition-transform duration-500 group-hover:rotate-180 dark:invert"
          />
          <span class="ml-3 text-lg font-semibold tracking-tight">Sunder</span>
        </a>
        <button
          class="text-muted-foreground hover:text-foreground -mr-2 p-2 md:hidden"
          onclick={() => (sidebarOpen = false)}
        >
          <X size={20} />
        </button>
      </div>

      <nav class="flex-1 space-y-1.5 overflow-y-auto px-4 py-4 md:px-3">
        {#each navItems as item, i}
          <a
            href={item.href}
            class="flex items-center gap-3 rounded-lg px-3 py-2.5 transition-all {currentPath.startsWith(
              item.href
            )
              ? 'bg-primary/10 text-primary font-semibold'
              : 'text-muted-foreground hover:bg-muted hover:text-foreground font-medium'}"
            use:tooltip={{ text: item.name, keys: ['Alt', String(i + 1)] }}
            onclick={() => (sidebarOpen = false)}
          >
            <svelte:component this={item.icon} size={20} class="shrink-0 md:h-[18px] md:w-[18px]" />
            <span class="text-base font-medium md:text-sm">{item.name}</span>
          </a>
        {/each}
      </nav>
      <div class="border-border flex flex-col justify-start border-t p-4 md:p-3">
        <SyncIndicator mode="sidebar" />
        <ThemeButton class="flex w-full items-center justify-center gap-3 py-2" showText={true} />
      </div>
    </aside>

    <div class="relative flex-1 overflow-hidden">
      <header
        class="bg-card border-border flex items-center justify-between border-b px-4 py-3 md:hidden"
      >
        <div class="flex items-center gap-3">
          <img src={favicon} alt="Sunder Logo" class="h-8 w-8" />
          <span class="text-xl font-bold tracking-tight">Sunder</span>
        </div>
        <button
          onclick={() => (sidebarOpen = true)}
          class="text-muted-foreground hover:bg-muted rounded-md p-1"
        >
          <Menu size={24} />
        </button>
      </header>

      <main class="bg-background/50 relative h-full overflow-y-auto">
        <div class="mx-auto h-full max-w-7xl p-4 pb-20 md:p-8 md:pb-24">
          {@render children()}
        </div>
      </main>
    </div>
  </div>

  <SyncIndicator />

  <Modal bind:open={modals.createAccount} isDirty={accountDirty} title="Create New Account">
    <AccountForm
      bind:isDirty={accountDirty}
      onsuccess={(id) => {
        modals.createAccount = false;
        goto(`/accounts/${id}`);
      }}
      oncancel={() => (modals.createAccount = false)}
    />
  </Modal>

  <Modal bind:open={modals.createCategory} isDirty={categoryDirty} title="Create New Category">
    <CategoryForm
      bind:isDirty={categoryDirty}
      onsuccess={(id) => {
        modals.createCategory = false;
        goto(`/categories/${id}`);
      }}
      oncancel={() => (modals.createCategory = false)}
    />
  </Modal>

  <Modal bind:open={modals.createEntry} isDirty={entryDirty} title="Create New Entry">
    <EntryForm
      bind:isDirty={entryDirty}
      initialAccountId={$page.route.id === '/(app)/accounts/[id]' ? $page.params.id : undefined}
      initialCategoryId={$page.route.id === '/(app)/categories/[id]' ? $page.params.id : undefined}
      onsuccess={(id) => {
        modals.createEntry = false;
        goto(`/entries/${id}`);
      }}
      oncancel={() => (modals.createEntry = false)}
    />
  </Modal>
{/if}
