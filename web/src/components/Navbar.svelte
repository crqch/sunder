<script lang="ts">
  import ThemeButton from '$components/ThemeButton.svelte';
  import favicon from '$lib/assets/favicon.svg';
</script>

<div
  class="bg-popover/90 text-secondary-foreground border-border/40 sticky top-0 z-50 flex h-20 w-full border-b backdrop-blur-md"
>
  <div class="mx-auto flex w-full max-w-7xl items-center justify-between px-4 sm:px-6 lg:px-8">
    <div class="flex items-center gap-8">
      <a href="/" class="group inline-flex items-center gap-3">
        <img
          src={favicon}
          alt="Logo"
          class="size-8 scale-[.98] object-scale-down transition-transform duration-500 group-hover:rotate-180 dark:invert"
        />
        <b class="text-lg font-bold tracking-widest">SUNDER</b>
      </a>

      <nav class="text-muted-foreground hidden items-center gap-6 text-sm font-medium md:flex">
        <a href="#features" class="hover:text-foreground transition-colors">Features</a>
        <a href="#demo" class="hover:text-foreground transition-colors">Interactive Demo</a>
        <a href="#experience" class="hover:text-foreground transition-colors">Apps & Platform</a>
      </nav>
    </div>

    <div class="flex items-center gap-3">
      <ThemeButton />
      <a href="/auth/sign-in" class="btn btn-outline btn-sm hidden sm:inline-flex">Sign in</a>
      <a href="/auth/sign-up" class="btn btn-sm">Get started</a>
    </div>
  </div>
</div>
