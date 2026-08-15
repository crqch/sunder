<script lang="ts">
  import favicon from '$lib/assets/favicon.svg';
  import { SiGithub } from '@icons-pack/svelte-simple-icons';
  import { ArrowUpRight } from '@lucide/svelte';
</script>

<footer class="border-border bg-card text-card-foreground w-full border-t transition-colors">
  <div class="mx-auto flex max-w-7xl flex-col gap-12 px-4 py-12 sm:px-6 lg:px-8 lg:py-16">
    <div class="grid grid-cols-1 gap-8 lg:grid-cols-4">
      <!-- Brand column -->
      <div class="flex flex-col gap-4 lg:col-span-2">
        <a href="/" class="group inline-flex w-fit items-center gap-3">
          <img
            src={favicon}
            alt="Sunder Logo"
            class="size-8 scale-[.98] object-scale-down transition-transform duration-500 group-hover:rotate-180 dark:invert"
          />
          <span class="text-foreground text-xl font-black tracking-widest">SUNDER</span>
        </a>
        <p class="text-muted-foreground max-w-sm text-sm leading-relaxed">
          Local-first personal finance and expense tracking. Total control over your data with
          instant offline responsiveness and multi-device sync.
        </p>
      </div>

      <!-- Navigation Column 1: Product -->
      <div class="flex flex-col gap-3">
        <h4 class="text-foreground text-sm font-bold tracking-wider uppercase">Product</h4>
        <ul class="text-muted-foreground flex flex-col gap-2 text-sm">
          <li>
            <a href="#features" class="hover:text-foreground transition-colors">Features</a>
          </li>
          <li>
            <a href="#experience" class="hover:text-foreground transition-colors"
              >Multi-Device Apps</a
            >
          </li>
          <li>
            <a href="#demo" class="hover:text-foreground transition-colors">Interactive Demo</a>
          </li>
          <li>
            <a href="/auth/sign-in" class="hover:text-foreground transition-colors">Sign In</a>
          </li>
        </ul>
      </div>

      <!-- Navigation Column 2: Access -->
      <div class="flex flex-col gap-3">
        <h4 class="text-foreground text-sm font-bold tracking-wider uppercase">Access</h4>
        <ul class="text-muted-foreground flex flex-col gap-2 text-sm">
          <li>
            <a
              href="/auth/sign-up"
              class="hover:text-foreground flex items-center gap-1 transition-colors"
            >
              <span>Get an Invite</span>
              <ArrowUpRight class="size-3" />
            </a>
          </li>
          <li>
            <a
              href="https://github.com/crqch/sunder"
              target="_blank"
              rel="noopener noreferrer"
              class="hover:text-foreground flex items-center gap-1 transition-colors"
            >
              <SiGithub class="size-3.5" />
              <span>Source Code</span>
            </a>
          </li>
        </ul>
      </div>
    </div>

    <!-- Bottom Bar -->
    <div
      class="border-border/60 text-muted-foreground flex flex-col items-center justify-between gap-4 border-t pt-8 text-xs sm:flex-row"
    >
      <p>© {new Date().getFullYear()} Sunder. Open source under MIT.</p>
      <div class="flex items-center gap-6">
        <a href="#privacy" class="hover:text-foreground transition-colors">Privacy</a>
      </div>
    </div>
  </div>
</footer>
