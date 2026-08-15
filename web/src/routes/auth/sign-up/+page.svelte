<script lang="ts">
  import { register } from '$lib/auth';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';

  let invite = $state($page.url.searchParams.get('invite') || '');
  let email = $state('');
  let username = $state('');
  let password = $state('');
  let error = $state('');
  let success = $state('');
  let loading = $state(false);

  async function handleSubmit(e: Event) {
    e.preventDefault();

    if (!invite || !email || !username || !password) {
      error = 'Please fill in all fields';
      return;
    }

    error = '';
    success = '';
    loading = true;

    try {
      await register(invite, email, username, password);
      success = 'Registration successful! You can now sign in.';

      // Redirect to sign in after a short delay
      setTimeout(() => {
        goto('/auth/sign-in');
      }, 2000);
    } catch (err: any) {
      // Handle specific error codes if available
      if (err.error_code) {
        switch (err.error_code) {
          case 'INVALID_INVITE':
            error = 'This invite code is invalid or does not exist.';
            break;
          case 'USED_INVITE':
            error = 'This invite code has already been used.';
            break;
          case 'VALIDATION_FAILED':
            error = err.message || 'Please check your input details.';
            break;
          default:
            error = err.message || 'Registration failed.';
        }
      } else {
        error = err.message || 'Registration failed. Please try again.';
      }
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head>
  <title>Sign Up - Sunder</title>
</svelte:head>

<div class="bg-card border-border border p-8 shadow-xl">
  <h2 class="text-card-foreground mb-6 text-2xl font-bold">Sign Up</h2>

  <form onsubmit={handleSubmit} class="flex flex-col gap-4">
    {#if error}
      <div
        class="bg-destructive/10 border-destructive text-destructive border p-3 font-mono text-sm"
      >
        {error}
      </div>
    {/if}

    {#if success}
      <div class="bg-primary/10 border-primary text-primary border p-3 font-mono text-sm">
        {success}
      </div>
    {/if}

    <div class="flex flex-col gap-2">
      <label for="invite" class="text-card-foreground font-mono text-sm">Invite Code</label>
      <input
        type="text"
        id="invite"
        bind:value={invite}
        class="bg-background border-border focus:ring-ring text-foreground w-full border p-2 font-mono focus:ring-2 focus:outline-none"
        disabled={loading || success !== ''}
      />
    </div>

    <div class="flex flex-col gap-2">
      <label for="email" class="text-card-foreground font-mono text-sm">Email</label>
      <input
        type="email"
        id="email"
        bind:value={email}
        class="bg-background border-border focus:ring-ring text-foreground w-full border p-2 font-mono focus:ring-2 focus:outline-none"
        disabled={loading || success !== ''}
        autocomplete="email"
      />
    </div>

    <div class="flex flex-col gap-2">
      <label for="username" class="text-card-foreground font-mono text-sm">Username</label>
      <input
        type="text"
        id="username"
        bind:value={username}
        class="bg-background border-border focus:ring-ring text-foreground w-full border p-2 font-mono focus:ring-2 focus:outline-none"
        disabled={loading || success !== ''}
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
        disabled={loading || success !== ''}
        autocomplete="new-password"
      />
    </div>

    <button
      type="submit"
      class="btn mt-4 w-full !rounded-none"
      disabled={loading || success !== ''}
    >
      {loading ? 'CREATING ACCOUNT...' : 'SIGN UP'}
    </button>
  </form>

  <div class="border-border text-muted-foreground mt-6 border-t pt-6 text-center font-mono text-sm">
    Already have an account?
    <a href="/auth/sign-in" class="text-primary underline-offset-4 hover:underline">Sign in</a>
  </div>
</div>
