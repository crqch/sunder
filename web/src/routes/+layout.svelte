<script lang="ts">
  import { themeStore } from '$lib/themeStore';
  import { onMount } from 'svelte';
  import { initAuth } from '$lib/auth';

  onMount(() => {
    initAuth();
  });

  let { children } = $props();
</script>

<div class="text-foreground bg-background dark">
  {@render children()}
</div>
