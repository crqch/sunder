<script lang="ts">
  import { login } from '$lib/auth';
  import { syncAll } from '$lib/sync';
  import { goto } from '$app/navigation';

  let loginField = $state('');
  let password = $state('');
  let error = $state('');
  let loading = $state(false);

  async function handleSubmit(e: Event) {
    e.preventDefault();

    if (!loginField || !password) {
      error = 'Please fill in all fields';
      return;
    }

    error = '';
    loading = true;

    try {
      await login(loginField, password);
      try {
        await syncAll();
      } catch (syncErr) {
        console.error('Initial sync after login failed:', syncErr);
      }
      goto('/dashboard');
    } catch (err: any) {
      error = err.message || 'Login failed. Please try again.';
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head>
  <title>Sign In - Sunder</title>
</svelte:head>

<div class="bg-card border-border border p-8 shadow-xl">
  <h2 class="text-card-foreground mb-6 text-2xl font-bold">Sign In</h2>

  <form onsubmit={handleSubmit} class="flex flex-col gap-4">
    {#if error}
      <div
        class="bg-destructive/10 border-destructive text-destructive border p-3 font-mono text-sm"
      >
        {error}
      </div>
    {/if}

    <div class="flex flex-col gap-2">
      <label for="login" class="text-card-foreground font-mono text-sm">Email or Username</label>
      <input
        type="text"
        id="login"
        bind:value={loginField}
        class="bg-background border-border focus:ring-ring text-foreground w-full border p-2 font-mono focus:ring-2 focus:outline-none"
        disabled={loading}
        autocomplete="username"
      />
    </div>

    <div class="flex flex-col gap-2">
      <label for="password" class="text-card-foreground font-mono text-sm">Password</label>
      <input
        type="password"
        id="password"
        bind:value={password}
        class="bg-background border-border focus:ring-ring text-foreground w-full border p-2 font-mono focus:ring-2 focus:outline-none"
        disabled={loading}
        autocomplete="current-password"
      />
    </div>

    <button type="submit" class="btn mt-4 w-full !rounded-none" disabled={loading}>
      {loading ? 'SIGNING IN...' : 'SIGN IN'}
    </button>
  </form>

  <div class="border-border text-muted-foreground mt-6 border-t pt-6 text-center font-mono text-sm">
    Don't have an account?
    <a href="/auth/sign-up" class="text-primary underline-offset-4 hover:underline">Sign up</a>
  </div>
</div>
