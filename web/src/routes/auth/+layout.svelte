<script lang="ts">
  import '../layout.css';
  import favicon from '$lib/assets/favicon.svg';
  import { authStore } from '$lib/auth';
  import { goto } from '$app/navigation';

  let { children } = $props();

  $effect(() => {
    if ($authStore.isAuthenticated) {
      goto('/dashboard');
    }
  });
</script>

<div class="flex min-h-screen flex-col items-center justify-center p-4">
  <div class="mb-8 flex flex-col items-center gap-4">
    <img src={favicon} alt="Sunder Logo" class="h-16 w-16 dark:invert" />
    <h1 class="text-foreground font-mono text-3xl font-bold tracking-widest uppercase">Sunder</h1>
  </div>

  <div class="w-full max-w-md">
    {@render children()}
  </div>
</div>
