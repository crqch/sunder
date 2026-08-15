<script lang="ts">
  import { goto } from '$app/navigation';
  import { db } from '$lib/db';
  import { ArrowLeft, Save } from '@lucide/svelte';

  let name = $state('');

  async function saveAccount(e: Event) {
    e.preventDefault();
    if (!name.trim()) return;

    const now = new Date().toISOString();
    const newAccount = {
      id: Date.now(),
      name,
      deleted_at: null,
      created_at: now,
      updated_at: now
    };

    await db.accounts.add(newAccount);
    goto('/accounts');
  }
</script>

<div class="mx-auto max-w-2xl space-y-6 font-sans">
  <div class="border-border/50 flex items-center gap-4 border-b pb-4">
    <a
      href="/accounts"
      class="border-border/50 hover:bg-muted rounded-lg border p-2 transition-colors"
    >
      <ArrowLeft size={20} />
    </a>
    <h1 class="text-2xl font-semibold tracking-tight">New Account</h1>
  </div>

  <form
    onsubmit={saveAccount}
    class="bg-card border-border/50 space-y-5 rounded-xl border p-6 shadow-sm"
  >
    <div class="space-y-1.5">
      <label for="name" class="text-foreground block text-sm font-medium">Account Name</label>
      <input
        id="name"
        type="text"
        bind:value={name}
        class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary w-full rounded-lg border p-2.5 text-sm transition-all focus:ring-2 focus:outline-none"
        placeholder="e.g. Checking, Savings, Cash"
        required
        autofocus
      />
    </div>

    <button
      type="submit"
      class="btn mt-4 flex w-full items-center justify-center gap-2 rounded-lg py-3 text-sm"
    >
      <Save size={16} /> Create Account
    </button>
  </form>
</div>
