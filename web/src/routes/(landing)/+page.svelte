<script lang="ts">
  import {
    Wallet,
    RefreshCw,
    ShieldCheck,
    Smartphone,
    ArrowRight,
    CheckCircle2,
    TrendingUp,
    PieChart,
    Database,
    Lock,
    Sparkles,
    Key,
    Layers,
    Zap,
    Tag,
    Globe,
    Cpu,
    Check,
    Laptop
  } from '@lucide/svelte';
  import { authStore } from '$lib/auth';

  // State for interactive demo tab on landing page
  let activeTab = $state<'ledger' | 'accounts' | 'sync'>('ledger');

  // Demo mock data based on Sunder finance schemas
  const mockEntries = [
    {
      id: 1,
      title: 'Weekly Grocery Shopping',
      category: 'Groceries',
      color: 'bg-emerald-500/10 text-emerald-500 border-emerald-500/20',
      account: 'Checking Account',
      amount: -142.8,
      date: 'Today, 14:20',
      location: 'Whole Foods Market'
    },
    {
      id: 2,
      title: 'Monthly Salary Deposit',
      category: 'Income',
      color: 'bg-blue-500/10 text-blue-500 border-blue-500/20',
      account: 'Main Savings',
      amount: 4250.0,
      date: 'Yesterday, 09:00',
      location: 'Employer Direct Deposit'
    },
    {
      id: 3,
      title: 'Cloud Storage & Hosting',
      category: 'Subscriptions',
      color: 'bg-purple-500/10 text-purple-500 border-purple-500/20',
      account: 'Credit Card',
      amount: -34.99,
      date: 'Aug 12, 18:45',
      location: 'Digital Workspace'
    },
    {
      id: 4,
      title: 'Coffee & Morning Snack',
      category: 'Dining',
      color: 'bg-amber-500/10 text-amber-500 border-amber-500/20',
      account: 'Checking Account',
      amount: -12.5,
      date: 'Aug 11, 08:30',
      location: 'Artisan Cafe'
    }
  ];

  const mockAccounts = [
    {
      name: 'Checking Account',
      balance: 3420.5,
      type: 'Liquid Cash',
      entriesCount: 148,
      change: '+12.4%'
    },
    {
      name: 'Main Savings',
      balance: 18500.0,
      type: 'High Yield',
      entriesCount: 24,
      change: '+4.2%'
    },
    {
      name: 'Credit Card',
      balance: -450.25,
      type: 'Revolving Line',
      entriesCount: 39,
      change: '-2.1%'
    }
  ];
</script>

<div
  class="mx-auto flex w-full max-w-7xl flex-col items-center gap-20 px-4 py-12 sm:px-6 lg:px-8 lg:py-16"
>
  <!-- HERO SECTION -->
  <section class="flex w-full flex-col items-center gap-8 pt-4 text-center">
    <!-- Badge -->
    <div
      class="border-primary/30 bg-primary/10 text-primary hover:bg-primary/20 inline-flex items-center gap-2 rounded-full border px-4 py-1.5 text-xs font-semibold transition-all"
    >
      <Zap class="fill-primary size-3.5" />
      <span>Local-First Engine • Instant Offline Access • Multi-Device Sync</span>
    </div>

    <!-- Main Headline -->
    <div class="flex max-w-4xl flex-col items-center gap-4">
      <h1 class="text-4xl font-extrabold tracking-tight sm:text-6xl lg:text-7xl">
        Quit guessing where your <span
          class="from-primary to-accent bg-gradient-to-r bg-clip-text text-transparent"
          >money goes</span
        >
      </h1>
      <p class="text-muted-foreground max-w-2xl text-lg leading-relaxed sm:text-xl">
        Sunder gives you instant financial clarity with zero loading delays. Manage accounts, log
        expenses offline, and automatically sync across your web and mobile devices.
      </p>
    </div>

    <!-- Action Buttons -->
    <div class="flex flex-wrap items-center justify-center gap-4 pt-2">
      {#if $authStore.isAuthenticated}
        <a
          class="btn hover:shadow-primary/25 px-6 py-3 text-base font-semibold shadow-lg transition-all"
          href="/dashboard"
        >
          <span>Go to Dashboard</span>
          <ArrowRight class="ml-2 size-4" />
        </a>
      {:else}
        <a
          class="btn hover:shadow-primary/25 px-6 py-3 text-base font-semibold shadow-lg transition-all"
          href="/auth/sign-in"
        >
          <span>Get Started Free</span>
          <ArrowRight class="ml-2 size-4" />
        </a>
        <a class="btn btn-outline px-6 py-3 text-base font-semibold" href="/auth/sign-up">
          <Key class="text-muted-foreground mr-2 size-4" />
          <span>Redeem Invite Code</span>
        </a>
      {/if}
    </div>

    <!-- Hero Visual Preview Card -->
    <div
      class="border-border/80 bg-card/60 relative mt-6 w-full max-w-5xl rounded-2xl border p-4 shadow-2xl backdrop-blur-xl sm:p-6 lg:p-8"
    >
      <div class="flex flex-col gap-6">
        <!-- Top bar mockup -->
        <div
          class="border-border/60 flex flex-wrap items-center justify-between gap-4 border-b pb-4"
        >
          <div class="flex items-center gap-3">
            <div class="flex size-3 gap-1.5">
              <span class="bg-destructive/80 size-3 rounded-full"></span>
              <span class="size-3 rounded-full bg-amber-500/80"></span>
              <span class="size-3 rounded-full bg-emerald-500/80"></span>
            </div>
            <span class="text-muted-foreground font-mono text-xs">sunder // personal finance</span>
          </div>

          <div class="flex items-center gap-2 text-xs">
            <span
              class="inline-flex items-center gap-1.5 rounded-md border border-emerald-500/20 bg-emerald-500/10 px-2.5 py-1 font-medium text-emerald-500"
            >
              <span class="size-2 animate-pulse rounded-full bg-emerald-500"></span>
              <span>Local Vault: Synced & Encrypted</span>
            </span>
          </div>
        </div>

        <!-- Dashboard Header Stats inside Mockup -->
        <div class="grid grid-cols-1 gap-4 sm:grid-cols-3">
          <div
            class="border-border/60 bg-background/50 flex flex-col gap-1 rounded-xl border p-4 text-left"
          >
            <span class="text-muted-foreground text-xs font-medium">Net Worth Overview</span>
            <span class="text-foreground text-2xl font-bold">$21,470.25</span>
            <span class="flex items-center gap-1 font-mono text-xs text-emerald-500">
              <TrendingUp class="size-3" /> +$3,807.20 this month
            </span>
          </div>

          <div
            class="border-border/60 bg-background/50 flex flex-col gap-1 rounded-xl border p-4 text-left"
          >
            <span class="text-muted-foreground text-xs font-medium">Active Accounts</span>
            <span class="text-foreground text-2xl font-bold">3 Managed</span>
            <span class="text-muted-foreground text-xs">Checking, Savings, Credit Card</span>
          </div>

          <div
            class="border-border/60 bg-background/50 flex flex-col gap-1 rounded-xl border p-4 text-left"
          >
            <span class="text-muted-foreground text-xs font-medium">Latest Entry</span>
            <span class="text-foreground text-2xl font-bold">-$142.80</span>
            <span class="text-muted-foreground text-xs">Groceries • Whole Foods</span>
          </div>
        </div>

        <!-- Ledger Mockup Rows -->
        <div class="flex flex-col gap-2 text-left">
          <div
            class="text-muted-foreground flex items-center justify-between px-2 py-1 text-xs font-semibold"
          >
            <span>Recent Activity</span>
            <span>Category & Amount</span>
          </div>
          {#each mockEntries.slice(0, 3) as entry}
            <div
              class="border-border/40 bg-background/40 hover:bg-background/80 flex items-center justify-between gap-4 rounded-lg border p-3 text-sm transition-all"
            >
              <div class="flex items-center gap-3">
                <div
                  class="bg-primary/10 text-primary flex size-9 items-center justify-center rounded-lg font-bold"
                >
                  {entry.title[0]}
                </div>
                <div class="flex flex-col">
                  <span class="text-foreground font-bold">{entry.title}</span>
                  <span class="text-muted-foreground text-xs">{entry.location} • {entry.date}</span>
                </div>
              </div>
              <div class="flex items-center gap-4">
                <span
                  class="hidden items-center rounded-md border px-2.5 py-0.5 text-xs font-medium sm:inline-flex {entry.color}"
                >
                  {entry.category}
                </span>
                <span
                  class="font-mono font-bold {entry.amount > 0
                    ? 'text-emerald-500'
                    : 'text-foreground'}"
                >
                  {entry.amount > 0 ? '+' : ''}{Math.abs(entry.amount).toFixed(2)}
                </span>
              </div>
            </div>
          {/each}
        </div>
      </div>
    </div>
  </section>

  <!-- METRICS & TRUST STATS -->
  <section class="border-border/60 grid w-full grid-cols-2 gap-6 border-y py-10 md:grid-cols-4">
    <div class="flex flex-col items-center gap-1 text-center">
      <span class="text-foreground text-3xl font-black sm:text-4xl">Instant</span>
      <span class="text-muted-foreground text-sm font-medium">Zero-Lag Offline Speed</span>
    </div>
    <div class="flex flex-col items-center gap-1 text-center">
      <span class="text-foreground text-3xl font-black sm:text-4xl">100%</span>
      <span class="text-muted-foreground text-sm font-medium">Private Data Control</span>
    </div>
    <div class="flex flex-col items-center gap-1 text-center">
      <span class="text-foreground text-3xl font-black sm:text-4xl">Seamless</span>
      <span class="text-muted-foreground text-sm font-medium">Web & Mobile Sync</span>
    </div>
    <div class="flex flex-col items-center gap-1 text-center">
      <span class="text-foreground text-3xl font-black sm:text-4xl">Exclusive</span>
      <span class="text-muted-foreground text-sm font-medium">Invite Access Protection</span>
    </div>
  </section>

  <!-- CORE FEATURES GRID SECTION -->
  <section id="features" class="flex w-full flex-col gap-12 pt-6">
    <div class="flex flex-col items-center gap-4 text-center">
      <h2 class="text-3xl font-black tracking-tight sm:text-4xl lg:text-5xl">
        Built for effortless financial clarity
      </h2>
      <p class="text-muted-foreground max-w-2xl text-base sm:text-lg">
        Traditional finance tools make you wait for slow servers and network spinners. Sunder puts
        speed and privacy back in your hands.
      </p>
    </div>

    <div class="grid grid-cols-1 gap-8 md:grid-cols-2">
      <!-- Feature 1 -->
      <div
        class="group border-border bg-card hover:border-primary/50 flex flex-col gap-4 rounded-2xl border p-6 shadow-sm transition-all hover:shadow-md"
      >
        <div
          class="bg-primary/10 text-primary flex size-12 items-center justify-center rounded-xl transition-transform group-hover:scale-110"
        >
          <Zap class="size-6" />
        </div>
        <h3 class="text-xl font-bold">Instant Local Responsiveness</h3>
        <p class="text-muted-foreground leading-relaxed">
          Log purchases and review budget balances instantly with zero latency. Sunder stores your
          data directly on your device so everything responds in milliseconds.
        </p>
      </div>

      <!-- Feature 2 -->
      <div
        class="group border-border bg-card hover:border-primary/50 flex flex-col gap-4 rounded-2xl border p-6 shadow-sm transition-all hover:shadow-md"
      >
        <div
          class="bg-primary/10 text-primary flex size-12 items-center justify-center rounded-xl transition-transform group-hover:scale-110"
        >
          <RefreshCw class="size-6" />
        </div>
        <h3 class="text-xl font-bold">Automatic Background Sync</h3>
        <p class="text-muted-foreground leading-relaxed">
          Whether you're offline on a plane or connected at home, your entries automatically
          reconcile across your devices whenever network connection is available.
        </p>
      </div>

      <!-- Feature 3 -->
      <div
        class="group border-border bg-card hover:border-primary/50 flex flex-col gap-4 rounded-2xl border p-6 shadow-sm transition-all hover:shadow-md"
      >
        <div
          class="bg-primary/10 text-primary flex size-12 items-center justify-center rounded-xl transition-transform group-hover:scale-110"
        >
          <Layers class="size-6" />
        </div>
        <h3 class="text-xl font-bold">Multi-Account & Category Management</h3>
        <p class="text-muted-foreground leading-relaxed">
          Manage cash, savings, and credit cards in unified accounts. Organize transactions with
          custom categories, color badges, and clear monthly analytics.
        </p>
      </div>

      <!-- Feature 4 -->
      <div
        class="group border-border bg-card hover:border-primary/50 flex flex-col gap-4 rounded-2xl border p-6 shadow-sm transition-all hover:shadow-md"
      >
        <div
          class="bg-primary/10 text-primary flex size-12 items-center justify-center rounded-xl transition-transform group-hover:scale-110"
        >
          <Lock class="size-6" />
        </div>
        <h3 class="text-xl font-bold">Private & Security First</h3>
        <p class="text-muted-foreground leading-relaxed">
          Protected by encrypted authentication tokens and invite-only membership. Your financial
          data belongs exclusively to you—never sold or mined by advertisers.
        </p>
      </div>
    </div>
  </section>

  <!-- INTERACTIVE DEMO / PREVIEW SECTION -->
  <section
    id="demo"
    class="border-border bg-card/40 flex w-full flex-col gap-8 rounded-3xl border p-6 backdrop-blur-md sm:p-8 lg:p-12"
  >
    <div class="flex flex-col items-center gap-4 text-center">
      <div
        class="border-accent/40 bg-accent/10 text-accent-foreground inline-flex items-center gap-1.5 rounded-full border px-3 py-1 text-xs font-semibold"
      >
        <Sparkles class="size-3.5" />
        <span>Interactive Workspace Preview</span>
      </div>
      <h2 class="text-3xl font-black sm:text-4xl">Experience Sunder in Action</h2>
      <p class="text-muted-foreground max-w-xl text-sm sm:text-base">
        Explore how Sunder organizes your financial ledger, balances, and real-time synchronization.
      </p>
    </div>

    <!-- Tabs Header -->
    <div class="border-border/60 flex justify-center border-b">
      <div class="flex gap-2 pb-2">
        <button
          onclick={() => (activeTab = 'ledger')}
          class="flex cursor-pointer items-center gap-2 rounded-lg px-4 py-2 text-sm font-semibold transition-all {activeTab ===
          'ledger'
            ? 'bg-primary text-primary-foreground shadow'
            : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'}"
        >
          <Tag class="size-4" />
          <span>Ledger Entries</span>
        </button>

        <button
          onclick={() => (activeTab = 'accounts')}
          class="flex cursor-pointer items-center gap-2 rounded-lg px-4 py-2 text-sm font-semibold transition-all {activeTab ===
          'accounts'
            ? 'bg-primary text-primary-foreground shadow'
            : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'}"
        >
          <Wallet class="size-4" />
          <span>Accounts</span>
        </button>

        <button
          onclick={() => (activeTab = 'sync')}
          class="flex cursor-pointer items-center gap-2 rounded-lg px-4 py-2 text-sm font-semibold transition-all {activeTab ===
          'sync'
            ? 'bg-primary text-primary-foreground shadow'
            : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'}"
        >
          <Globe class="size-4" />
          <span>Multi-Device Sync</span>
        </button>
      </div>
    </div>

    <!-- Tab Content -->
    <div class="w-full">
      {#if activeTab === 'ledger'}
        <div class="flex flex-col gap-3">
          <div class="flex items-center justify-between">
            <h4 class="text-lg font-bold">Transaction Ledger</h4>
            <span class="text-muted-foreground text-xs">{mockEntries.length} items logged</span>
          </div>
          <div class="grid gap-3">
            {#each mockEntries as entry}
              <div
                class="border-border bg-background hover:border-primary/40 flex flex-col justify-between gap-4 rounded-xl border p-4 shadow-sm transition-all sm:flex-row sm:items-center"
              >
                <div class="flex items-center gap-3">
                  <div
                    class="bg-secondary text-secondary-foreground flex size-10 items-center justify-center rounded-xl font-bold"
                  >
                    {entry.title[0]}
                  </div>
                  <div class="flex flex-col">
                    <span class="text-foreground font-bold">{entry.title}</span>
                    <span class="text-muted-foreground text-xs"
                      >{entry.account} • {entry.location}</span
                    >
                  </div>
                </div>

                <div class="flex items-center justify-between gap-4 sm:justify-end">
                  <span
                    class="inline-flex items-center rounded-md border px-2.5 py-1 text-xs font-semibold {entry.color}"
                  >
                    {entry.category}
                  </span>
                  <span
                    class="font-mono text-base font-extrabold {entry.amount > 0
                      ? 'text-emerald-500'
                      : 'text-foreground'}"
                  >
                    {entry.amount > 0 ? '+' : ''}{Math.abs(entry.amount).toFixed(2)}
                  </span>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {:else if activeTab === 'accounts'}
        <div class="flex flex-col gap-4">
          <div class="flex items-center justify-between">
            <h4 class="text-lg font-bold">Managed Accounts</h4>
            <span class="flex items-center gap-1 text-xs font-semibold text-emerald-500">
              <CheckCircle2 class="size-3.5" /> All Accounts Active
            </span>
          </div>
          <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
            {#each mockAccounts as acc}
              <div
                class="border-border bg-background flex flex-col justify-between gap-4 rounded-xl border p-5 shadow-sm"
              >
                <div class="flex items-center justify-between">
                  <span class="text-muted-foreground text-sm font-semibold">{acc.type}</span>
                  <span class="font-mono text-xs font-bold text-emerald-500">{acc.change}</span>
                </div>
                <div class="flex flex-col gap-1">
                  <h5 class="text-foreground text-lg font-extrabold">{acc.name}</h5>
                  <span class="text-foreground font-mono text-2xl font-black"
                    >{acc.balance.toFixed(2)}</span
                  >
                </div>
                <div
                  class="border-border/60 text-muted-foreground flex justify-between border-t pt-3 text-xs"
                >
                  <span>{acc.entriesCount} transactions logged</span>
                  <span>Active</span>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {:else}
        <div class="flex flex-col items-center gap-6 py-6 text-center">
          <div class="grid w-full max-w-4xl grid-cols-1 gap-6 md:grid-cols-3">
            <!-- Box 1 -->
            <div
              class="border-border bg-background flex flex-col items-center gap-3 rounded-2xl border p-6"
            >
              <Smartphone class="text-primary size-8" />
              <h4 class="font-bold">Mobile App</h4>
              <p class="text-muted-foreground text-xs">
                Capture receipts and log transactions instantly on the go.
              </p>
            </div>
            <!-- Box 2 -->
            <div
              class="text-primary flex flex-col items-center justify-center gap-2 text-xs font-bold"
            >
              <RefreshCw class="text-primary size-6 animate-spin" />
              <span>Real-Time Cloud Sync</span>
              <span class="text-muted-foreground text-[10px]">Encrypted Background Channel</span>
            </div>
            <!-- Box 3 -->
            <div
              class="border-border bg-background flex flex-col items-center gap-3 rounded-2xl border p-6"
            >
              <Laptop class="text-accent-foreground size-8" />
              <h4 class="font-bold">Web Dashboard</h4>
              <p class="text-muted-foreground text-xs">
                Review full budget reports, export data, and manage categories.
              </p>
            </div>
          </div>
        </div>
      {/if}
    </div>
  </section>

  <!-- APPS & PLATFORM SECTION -->
  <section id="experience" class="flex w-full flex-col gap-12 pt-4">
    <div class="flex flex-col items-center gap-4 text-center">
      <h2 class="text-3xl font-black tracking-tight sm:text-4xl">Designed for all your devices</h2>
      <p class="text-muted-foreground max-w-2xl text-base">
        Access your finances whenever and wherever you need them.
      </p>
    </div>

    <div class="grid grid-cols-1 gap-6 md:grid-cols-3">
      <div class="border-border bg-card flex flex-col gap-4 rounded-2xl border p-6">
        <div class="flex items-center gap-3">
          <Laptop class="text-primary size-6" />
          <h3 class="text-lg font-bold">Desktop & Web Dashboard</h3>
        </div>
        <p class="text-muted-foreground text-sm leading-relaxed">
          Full-featured web workspace with keyboard navigation, financial metrics, and custom
          account controls.
        </p>
        <span class="text-primary mt-auto text-xs font-semibold">Web App</span>
      </div>

      <div class="border-border bg-card flex flex-col gap-4 rounded-2xl border p-6">
        <div class="flex items-center gap-3">
          <Smartphone class="text-primary size-6" />
          <h3 class="text-lg font-bold">Native Mobile Experience</h3>
        </div>
        <p class="text-muted-foreground text-sm leading-relaxed">
          Lightweight Android app built for rapid entry and tap-to-log spending when you're out and
          about.
        </p>
        <span class="text-primary mt-auto text-xs font-semibold">Android App</span>
      </div>

      <div class="border-border bg-card flex flex-col gap-4 rounded-2xl border p-6">
        <div class="flex items-center gap-3">
          <RefreshCw class="text-primary size-6" />
          <h3 class="text-lg font-bold">Encrypted Cloud Backup</h3>
        </div>
        <p class="text-muted-foreground text-sm leading-relaxed">
          Automatic cloud synchronization that safeguards your budget history against lost or
          upgraded devices.
        </p>
        <span class="text-primary mt-auto text-xs font-semibold">Cloud Sync Engine</span>
      </div>
    </div>
  </section>

  <!-- HOW IT WORKS STEP-BY-STEP -->
  <section class="flex w-full flex-col gap-12 pt-4">
    <div class="flex flex-col items-center gap-4 text-center">
      <h2 class="text-3xl font-black tracking-tight sm:text-4xl">How Sunder Works</h2>
      <p class="text-muted-foreground max-w-xl text-base">
        Get started in under 2 minutes with complete data clarity.
      </p>
    </div>

    <div class="grid grid-cols-1 gap-8 md:grid-cols-3">
      <div
        class="border-border bg-card/60 flex flex-col items-center gap-4 rounded-2xl border p-6 text-center"
      >
        <div
          class="bg-primary text-primary-foreground flex size-12 items-center justify-center rounded-full text-lg font-black shadow"
        >
          1
        </div>
        <h3 class="text-lg font-bold">Sign In or Redeem Invite</h3>
        <p class="text-muted-foreground text-sm">
          Log in to your account or enter your exclusive pass code to unlock your private vault.
        </p>
      </div>

      <div
        class="border-border bg-card/60 flex flex-col items-center gap-4 rounded-2xl border p-6 text-center"
      >
        <div
          class="bg-primary text-primary-foreground flex size-12 items-center justify-center rounded-full text-lg font-black shadow"
        >
          2
        </div>
        <h3 class="text-lg font-bold">Set Up Accounts & Categories</h3>
        <p class="text-muted-foreground text-sm">
          Add your checking, savings, and credit lines, and customize color-coded expense
          categories.
        </p>
      </div>

      <div
        class="border-border bg-card/60 flex flex-col items-center gap-4 rounded-2xl border p-6 text-center"
      >
        <div
          class="bg-primary text-primary-foreground flex size-12 items-center justify-center rounded-full text-lg font-black shadow"
        >
          3
        </div>
        <h3 class="text-lg font-bold">Track & Sync Anywhere</h3>
        <p class="text-muted-foreground text-sm">
          Log spending with instant zero-lag feedback. Sunder keeps all your devices in sync
          effortlessly.
        </p>
      </div>
    </div>
  </section>

  <!-- FINAL CTA BANNER -->
  <section
    class="from-primary/20 via-card to-card border-primary/30 flex w-full flex-col items-center gap-8 rounded-3xl border bg-gradient-to-br p-8 text-center shadow-xl sm:p-12 lg:p-16"
  >
    <div class="flex max-w-2xl flex-col gap-4">
      <h2 class="text-3xl font-extrabold sm:text-5xl">
        Take control of your personal finances today
      </h2>
      <p class="text-muted-foreground text-base sm:text-lg">
        Join Sunder to experience zero-lag speed, private invite-only security, and effortless
        multi-account management.
      </p>
    </div>

    <div class="flex flex-wrap items-center justify-center gap-4">
      {#if $authStore.isAuthenticated}
        <a
          class="btn hover:shadow-primary/30 px-8 py-3 text-base font-semibold shadow-lg"
          href="/dashboard"
        >
          <span>Go to Dashboard</span>
          <ArrowRight class="ml-2 size-4" />
        </a>
      {:else}
        <a
          class="btn hover:shadow-primary/30 px-8 py-3 text-base font-semibold shadow-lg"
          href="/auth/sign-in"
        >
          <span>Sign In to Dashboard</span>
          <ArrowRight class="ml-2 size-4" />
        </a>
        <a class="btn btn-outline px-8 py-3 text-base font-semibold" href="/auth/sign-up">
          <span>Sign Up with Invite</span>
        </a>
      {/if}
    </div>
  </section>
</div>
