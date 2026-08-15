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
</script>

<div
  class="mx-auto flex w-full max-w-7xl flex-col items-center gap-20 px-4 py-12 sm:px-6 lg:px-8 lg:py-16"
>
  <!-- HERO SECTION -->
  <section
    class="relative flex w-full flex-col items-center gap-8 overflow-hidden pt-12 pb-8 text-center"
  >
    <!-- Glowing Orbs Background -->
    <div
      class="pointer-events-none absolute top-1/3 left-1/2 -z-10 -translate-x-1/2 -translate-y-1/2"
    >
      <div
        class="bg-primary/20 absolute top-[-10rem] left-[-20rem] h-[30rem] w-[30rem] animate-pulse rounded-full opacity-70 mix-blend-screen blur-[100px]"
      ></div>
      <div
        class="absolute top-[5rem] left-[5rem] h-[25rem] w-[25rem] rounded-full bg-emerald-500/10 opacity-60 mix-blend-screen blur-[120px]"
      ></div>
    </div>
    <!-- Badge -->
    <div
      class="border-primary/30 bg-primary/10 text-primary hover:bg-primary/20 inline-flex items-center gap-2 rounded-full border px-4 py-1.5 text-xs font-semibold transition-all"
    >
      <Zap class="fill-primary size-3.5" />
      <span>Local-First Engine • Instant Offline Access • Multi-Device Sync</span>
    </div>

    <!-- Main Headline -->
    <div class="relative z-10 flex max-w-4xl flex-col items-center gap-6">
      <h1 class="text-5xl font-black tracking-tight drop-shadow-sm sm:text-6xl lg:text-7xl/tight">
        Master your wealth at the <span class="text-primary drop-shadow-md">speed of thought</span>
      </h1>
      <p class="text-muted-foreground max-w-2xl text-lg leading-relaxed font-medium sm:text-xl">
        Sunder delivers instant financial clarity with zero loading delays. Command your accounts,
        log expenses offline, and experience frictionless sync across all your devices.
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
      class="shadow-primary/20 hover:shadow-primary/30 group relative mt-10 w-full max-w-5xl overflow-hidden rounded-3xl border border-white/10 bg-black/40 p-4 shadow-2xl backdrop-blur-2xl transition-all duration-700 sm:p-6 lg:p-8"
    >
      <!-- Glassmorphism shine effect -->
      <div
        class="absolute inset-0 bg-gradient-to-br from-white/5 to-transparent opacity-0 transition-opacity duration-1000 group-hover:opacity-100"
      ></div>
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
        class="group border-border/50 bg-card/30 hover:border-primary/50 hover:bg-card/50 flex flex-col gap-4 rounded-3xl border p-8 shadow-sm backdrop-blur-md transition-all duration-500 hover:-translate-y-1 hover:shadow-xl"
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
        class="group border-border/50 bg-card/30 hover:border-primary/50 hover:bg-card/50 flex flex-col gap-4 rounded-3xl border p-8 shadow-sm backdrop-blur-md transition-all duration-500 hover:-translate-y-1 hover:shadow-xl"
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
        class="group border-border/50 bg-card/30 hover:border-primary/50 hover:bg-card/50 flex flex-col gap-4 rounded-3xl border p-8 shadow-sm backdrop-blur-md transition-all duration-500 hover:-translate-y-1 hover:shadow-xl"
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
        class="group border-border/50 bg-card/30 hover:border-primary/50 hover:bg-card/50 flex flex-col gap-4 rounded-3xl border p-8 shadow-sm backdrop-blur-md transition-all duration-500 hover:-translate-y-1 hover:shadow-xl"
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
</div>
