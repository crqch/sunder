<script lang="ts">
  import { themeStore } from '$lib/themeStore';
  import { Moon, Sun } from '@lucide/svelte';
  import { tooltip } from '$lib/tooltip';

  let {
    class: className,
    showText = false,
    responsiveText = false
  }: { class?: string; showText?: boolean; responsiveText?: boolean } = $props();
</script>

<button
  type="button"
  onclick={(e) => themeStore.toggle(e)}
  class="btn btn-outline flex items-center justify-center gap-2 overflow-hidden transition-all {className}"
  use:tooltip={{ text: 'Toggle Theme', keys: ['Alt', 'T'] }}
  aria-label="Toggle theme"
>
  <div class="relative flex size-4 shrink-0 items-center justify-center">
    <Sun
      class="absolute scale-100 rotate-0 opacity-100 transition-all duration-500 dark:scale-0 dark:rotate-90 dark:opacity-0"
      size={16}
    />
    <Moon
      class="absolute scale-0 -rotate-90 opacity-0 transition-all duration-500 dark:scale-100 dark:rotate-0 dark:opacity-100"
      size={16}
    />
  </div>

  {#if showText || responsiveText}
    <span
      class="font-medium whitespace-nowrap {responsiveText
        ? 'hidden sm:inline-block'
        : 'inline-block'}"
    >
      <span class="dark:hidden">Light mode</span>
      <span class="hidden dark:inline">Dark mode</span>
    </span>
  {/if}
</button>
