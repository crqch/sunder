<script lang="ts">
  import {
    AppShell,
    Avatar,
    Button,
    Center,
    HStack,
    Icon,
    IconButton,
    NavIcon,
    SideNav,
    SideNavHeading,
    SideNavItem,
    SideNavSection,
    Spinner,
    StackItem,
    Text,
    Token,
    VStack
  } from '@astryx-sv/core';
  import { Home, Keyboard, List, Plus, Settings, ShieldCheck, Tags, Wallet } from '@lucide/svelte';
  import { authStore } from '$lib/auth';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import favicon from '$lib/assets/favicon.svg';
  import BrandMark from '$components/BrandMark.svelte';
  import SyncIndicator from '$components/SyncIndicator.svelte';
  import ThemeButton from '$components/ThemeButton.svelte';
  import { currencyStore } from '$lib/currency.svelte';
  import AccountForm from '$components/AccountForm.svelte';
  import CategoryForm from '$components/CategoryForm.svelte';
  import EntryForm from '$components/EntryForm.svelte';
  import Modal from '$components/Modal.svelte';
  import { syncState } from '$lib/syncState.svelte';
  import { keybinds } from '$lib/keybinds.svelte';
  import { modals } from '$lib/modals.svelte';

  let { children } = $props();

  $effect(() => {
    if (!$authStore.loading && !$authStore.isAuthenticated) {
      goto('/auth/sign-in');
    }
  });

  let currencyInitialized = false;
  $effect(() => {
    if ($authStore.isAuthenticated) {
      if ($authStore.user?.default_currency_code) {
        currencyStore.setDefaultCurrencyLocal($authStore.user.default_currency_code);
      }
      if (!currencyInitialized) {
        currencyInitialized = true;
        currencyStore.syncAllCurrencyData();
      }
    } else {
      currencyInitialized = false;
    }
  });

  let currentPath = $derived($page.url.pathname);

  // Desktop sidebar state (width + collapsed) managed directly here so:
  // 1. Astryx SideNav stays mounted across collapse toggles (no <nav> remount,
  //    allowing true CSS width/opacity transitions),
  // 2. Persisted collapsed state can never desync with Astryx useResizable into
  //    a 0px invisible sidebar on page reload,
  // 3. Mobile drawer (<= 768px) never inherits desktop collapsed UI.
  const SIDENAV_WIDTH_KEY = 'sunder_sidenav_width';
  const SIDENAV_COLLAPSED_KEY = 'sunder_sidenav_collapsed';
  const LEGACY_ASTRYX_KEY = 'astryx-resizable:sunder-sidenav';
  const SIDENAV_DEFAULT_WIDTH = 256;
  const SIDENAV_MIN_WIDTH = 220;
  const SIDENAV_MAX_WIDTH = 360;
  const SIDENAV_COLLAPSE_THRESHOLD = 160;

  function clampSideNavWidth(raw: number): number {
    if (!Number.isFinite(raw)) return SIDENAV_DEFAULT_WIDTH;
    return Math.round(Math.min(SIDENAV_MAX_WIDTH, Math.max(SIDENAV_MIN_WIDTH, raw)));
  }

  function readInitialSideNavState(): { width: number; collapsed: boolean } {
    if (typeof window === 'undefined') {
      return { width: SIDENAV_DEFAULT_WIDTH, collapsed: false };
    }
    try {
      // Clean up legacy Astryx resizable key (which could trap SideNav at 0px on reload)
      // while preserving any valid width the user previously chose.
      const legacyRaw = localStorage.getItem(LEGACY_ASTRYX_KEY);
      if (legacyRaw) {
        localStorage.removeItem(LEGACY_ASTRYX_KEY);
        const parsed = JSON.parse(legacyRaw);
        if (parsed && typeof parsed === 'object') {
          const migratedWidth = clampSideNavWidth(Number(parsed.size));
          if (!localStorage.getItem(SIDENAV_WIDTH_KEY)) {
            localStorage.setItem(SIDENAV_WIDTH_KEY, String(migratedWidth));
          }
          // Intentionally reset stuck legacy collapsed state to expanded so any
          // user who previously hit the 0px invisible-sidebar bug immediately
          // recovers their visible sidebar on reload.
        }
      }
      const savedWidth = Number(localStorage.getItem(SIDENAV_WIDTH_KEY));
      const width =
        localStorage.getItem(SIDENAV_WIDTH_KEY) !== null
          ? clampSideNavWidth(savedWidth)
          : SIDENAV_DEFAULT_WIDTH;
      const collapsed = localStorage.getItem(SIDENAV_COLLAPSED_KEY) === 'true';
      return { width, collapsed };
    } catch {
      return { width: SIDENAV_DEFAULT_WIDTH, collapsed: false };
    }
  }

  const initialSideNav = readInitialSideNavState();
  let sideNavWidth = $state(initialSideNav.width);
  let sideNavCollapsed = $state(initialSideNav.collapsed);
  let isResizingSideNav = $state(false);
  let isDesktop = $state(
    typeof window !== 'undefined' ? window.matchMedia('(min-width: 769px)').matches : true
  );

  // Only apply collapsed rail presentation on desktop; mobile drawer always renders full UI.
  let effectiveCollapsed = $derived(isDesktop && sideNavCollapsed);

  $effect(() => {
    if (typeof window === 'undefined') return;
    const mq = window.matchMedia('(min-width: 769px)');
    isDesktop = mq.matches;
    const onChange = (e: MediaQueryListEvent) => {
      isDesktop = e.matches;
    };
    mq.addEventListener('change', onChange);
    return () => mq.removeEventListener('change', onChange);
  });

  function setSideNavCollapsed(next: boolean) {
    sideNavCollapsed = next;
    if (typeof window !== 'undefined') {
      localStorage.setItem(SIDENAV_COLLAPSED_KEY, String(next));
    }
  }

  function setSideNavWidth(next: number) {
    sideNavWidth = clampSideNavWidth(next);
    if (typeof window !== 'undefined') {
      localStorage.setItem(SIDENAV_WIDTH_KEY, String(sideNavWidth));
    }
  }

  function handleResizePointerDown(e: PointerEvent) {
    if (e.button !== 0 || !isDesktop || effectiveCollapsed) return;
    e.preventDefault();
    const startX = e.clientX;
    const startWidth = sideNavWidth;
    isResizingSideNav = true;
    document.body.style.userSelect = 'none';
    document.body.style.cursor = 'col-resize';

    let pendingCollapse = false;

    const onMove = (ev: PointerEvent) => {
      const raw = startWidth + (ev.clientX - startX);
      if (raw < SIDENAV_COLLAPSE_THRESHOLD) {
        pendingCollapse = true;
      } else {
        pendingCollapse = false;
        sideNavWidth = clampSideNavWidth(raw);
      }
    };

    const cleanup = () => {
      window.removeEventListener('pointermove', onMove);
      window.removeEventListener('pointerup', onUp);
      window.removeEventListener('pointercancel', onUp);
      isResizingSideNav = false;
      document.body.style.userSelect = '';
      document.body.style.cursor = '';
    };

    const onUp = () => {
      cleanup();
      if (pendingCollapse) {
        setSideNavCollapsed(true);
      } else {
        setSideNavWidth(sideNavWidth);
      }
    };

    window.addEventListener('pointermove', onMove);
    window.addEventListener('pointerup', onUp);
    window.addEventListener('pointercancel', onUp);
  }

  function handleResizeKeyDown(e: KeyboardEvent) {
    if (e.key === 'ArrowLeft') {
      e.preventDefault();
      setSideNavWidth(sideNavWidth - 16);
    } else if (e.key === 'ArrowRight') {
      e.preventDefault();
      setSideNavWidth(sideNavWidth + 16);
    } else if (e.key === 'Home') {
      e.preventDefault();
      setSideNavWidth(SIDENAV_MIN_WIDTH);
    } else if (e.key === 'End') {
      e.preventDefault();
      setSideNavWidth(SIDENAV_MAX_WIDTH);
    }
  }

  // Keep AppShell's banner landmark absent when sync is idle/minimized while
  // giving the slide-out transition time to finish before unmounting the slot.
  let shouldShowSyncBanner = $derived(!syncState.minimized && syncState.status !== 'idle');
  let bannerSlotMounted = $state(false);

  $effect(() => {
    if (shouldShowSyncBanner) {
      bannerSlotMounted = true;
      return;
    }
    if (!bannerSlotMounted) return;
    const timer = setTimeout(() => {
      bannerSlotMounted = false;
    }, 240);
    return () => clearTimeout(timer);
  });

  const workspaceItems = [
    { name: 'Dashboard', href: '/dashboard', icon: Home },
    { name: 'Entries', href: '/entries', icon: List },
    { name: 'Accounts', href: '/accounts', icon: Wallet },
    { name: 'Categories', href: '/categories', icon: Tags }
  ];

  let systemItems = $derived.by(() => {
    const items = [{ name: 'Settings', href: '/settings', icon: Settings }];
    if ($authStore.user?.flags?.includes('is_admin')) {
      items.push({ name: 'Admin', href: '/admin', icon: ShieldCheck });
    }
    return items;
  });

  let accountDirty = $state(false);
  let categoryDirty = $state(false);
  let entryDirty = $state(false);

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
        action: () => goto('/dashboard')
      }),
      keybinds.register({
        id: 'app.tab2',
        name: 'Entries Tab',
        keys: ['alt+2'],
        global: false,
        action: () => goto('/entries')
      }),
      keybinds.register({
        id: 'app.tab_entries_alt',
        name: 'Entries Tab',
        keys: ['alt+e'],
        global: false,
        action: () => goto('/entries')
      }),
      keybinds.register({
        id: 'app.tab3',
        name: 'Accounts Tab',
        keys: ['alt+3'],
        global: false,
        action: () => goto('/accounts')
      }),
      keybinds.register({
        id: 'app.tab_accounts_alt',
        name: 'Accounts Tab',
        keys: ['alt+a'],
        global: false,
        action: () => goto('/accounts')
      }),
      keybinds.register({
        id: 'app.tab4',
        name: 'Categories Tab',
        keys: ['alt+4'],
        global: false,
        action: () => goto('/categories')
      }),
      keybinds.register({
        id: 'app.tab_categories_alt',
        name: 'Categories Tab',
        keys: ['alt+c'],
        global: false,
        action: () => goto('/categories')
      }),
      keybinds.register({
        id: 'app.tab5',
        name: 'Settings Tab',
        keys: ['alt+5'],
        global: false,
        action: () => goto('/settings')
      })
    ];
    return () => unregs.forEach((u) => u());
  });
</script>

<svelte:head>
  <title>Sunder</title>
  <link rel="icon" href={favicon} />
</svelte:head>

{#snippet brandMark()}
  <BrandMark size={20} />
{/snippet}

{#snippet logoIcon()}
  <NavIcon icon={brandMark} />
{/snippet}

{#snippet plusIcon()}
  <Icon icon={Plus} size="sm" />
{/snippet}

{#snippet keyboardIcon()}
  <Icon icon={Keyboard} size="sm" />
{/snippet}

{#snippet syncBanner()}
  <SyncIndicator mode="banner" />
{/snippet}

{#if $authStore.loading || !$authStore.isAuthenticated}
  <Center minHeight="100dvh">
    <Spinner size="xl" label="Loading Sunder..." />
  </Center>
{:else}
  <SyncIndicator mode="poller" />

  <!--
    Responsive contract:
      > 768px   SideNav 256px (resizable 220-360, collapsible) | content
      <= 768px  nav collapses into AppShell's mobile drawer
  -->
  <AppShell contentPadding={0} height="fill" banner={bannerSlotMounted ? syncBanner : undefined}>
    {#snippet sideNav()}
      <div class="sunder-sidenav-shell" data-resizing={isResizingSideNav ? '' : undefined}>
        <SideNav
          class={`sunder-sidenav${effectiveCollapsed ? ' is-collapsed' : ''}${isResizingSideNav ? ' is-resizing' : ''}`}
          style={isDesktop && !effectiveCollapsed ? `width: ${sideNavWidth}px;` : undefined}
          collapsible={{
            isCollapsed: effectiveCollapsed,
            onCollapsedChange: (next) => setSideNavCollapsed(next)
          }}
        >
          {#snippet header()}
            <SideNavHeading heading="Sunder" headingHref="/dashboard" icon={logoIcon} />
          {/snippet}

          {#snippet topContent()}
            <div class="sunder-sidenav-top-content">
              {#if effectiveCollapsed}
                <IconButton
                  class="sunder-sidenav-new-entry"
                  label="New entry"
                  variant="primary"
                  size="sm"
                  icon={plusIcon}
                  tooltip="New entry (Alt+Space)"
                  onclick={() => (modals.createEntry = true)}
                />
              {:else}
                <Button
                  class="sunder-sidenav-new-entry"
                  label="New entry"
                  variant="primary"
                  width="100%"
                  icon={plusIcon}
                  tooltip="New entry (Alt+Space)"
                  onclick={() => (modals.createEntry = true)}
                />
              {/if}
            </div>
          {/snippet}

          <SideNavSection title="Workspace">
            {#each workspaceItems as item (item.href)}
              {#snippet itemIcon()}
                <Icon icon={item.icon} size="md" />
              {/snippet}
              <SideNavItem
                class="sunder-nav-item"
                label={item.name}
                href={item.href}
                isSelected={currentPath.startsWith(item.href)}
                icon={itemIcon}
              />
            {/each}
          </SideNavSection>

          <SideNavSection title="System">
            {#each systemItems as item (item.href)}
              {#snippet itemIcon()}
                <Icon icon={item.icon} size="md" />
              {/snippet}
              <SideNavItem
                class="sunder-nav-item"
                label={item.name}
                href={item.href}
                isSelected={currentPath.startsWith(item.href)}
                icon={itemIcon}
              />
            {/each}
          </SideNavSection>

          {#snippet footer()}
            <VStack class="sunder-sidenav-footer" gap={2} padding={3}>
              <SyncIndicator mode="sidebar" compact={effectiveCollapsed} />
              {#if $authStore.user}
                <HStack
                  class="sunder-sidenav-user"
                  gap={2}
                  vAlign="center"
                  hAlign="between"
                  width="100%"
                >
                  <HStack class="sunder-sidenav-user-main" gap={2} vAlign="center">
                    <Avatar
                      class="sunder-sidenav-avatar"
                      name={$authStore.user.username}
                      size="sm"
                      tooltip={false}
                    />
                    <StackItem class="sunder-sidenav-user-item" size="fill">
                      <VStack class="sunder-sidenav-user-text" gap={0}>
                        <Text size="sm" weight="medium" maxLines={1}>
                          {$authStore.user.username}
                        </Text>
                        <Text type="supporting" color="secondary" maxLines={1}>
                          {$authStore.user.email}
                        </Text>
                      </VStack>
                    </StackItem>
                  </HStack>
                  <Token
                    class="sunder-sidenav-currency"
                    label={currencyStore.defaultCurrencyCode}
                    size="sm"
                  />
                </HStack>
              {/if}
            </VStack>
          {/snippet}

          {#snippet footerIcons()}
            <ThemeButton variant="ghost" size="md" />
            <IconButton
              label="Keyboard shortcuts"
              variant="ghost"
              icon={keyboardIcon}
              tooltip="Keyboard shortcuts (Alt+?)"
              onclick={() => keybinds.toggleHelp()}
            />
          {/snippet}
        </SideNav>

        {#if isDesktop && !effectiveCollapsed}
          <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
          <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
          <div
            class="sunder-sidenav-resize-handle"
            role="separator"
            aria-orientation="vertical"
            aria-label="Resize sidebar"
            aria-valuemin={SIDENAV_MIN_WIDTH}
            aria-valuemax={SIDENAV_MAX_WIDTH}
            aria-valuenow={sideNavWidth}
            tabindex="0"
            data-resizing={isResizingSideNav ? '' : undefined}
            onpointerdown={handleResizePointerDown}
            ondblclick={() => setSideNavWidth(SIDENAV_DEFAULT_WIDTH)}
            onkeydown={handleResizeKeyDown}
          ></div>
        {/if}
      </div>
    {/snippet}

    {@render children()}
  </AppShell>

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
