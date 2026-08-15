<script lang="ts">
  import { authenticatedFetch } from '$lib/auth';
  import { onMount } from 'svelte';
  import { Trash2, KeyRound, Plus, ShieldCheck, Eye, EyeOff, Pencil } from '@lucide/svelte';
  import { toast } from 'svelte-sonner';
  import Modal from '$components/Modal.svelte';

  let users = $state<any[]>([]);
  let invites = $state<any[]>([]);
  let loading = $state(true);

  let editingUser = $state<any>(null);
  let editUsername = $state('');
  let editEmail = $state('');
  let editFlags = $state('');
  let editPassword = $state('');
  let editModalOpen = $state(false);
  let showPassword = $state(false);

  let newInviteToken = $state('');

  async function loadData() {
    try {
      const [uRes, iRes] = await Promise.all([
        authenticatedFetch('/admin/users'),
        authenticatedFetch('/admin/invites')
      ]);
      const uData = await uRes.json();
      const iData = await iRes.json();
      users = uData.users || [];
      invites = iData.invites || [];
    } catch (e) {
      toast.error('Failed to load admin data');
    } finally {
      loading = false;
    }
  }

  onMount(loadData);

  async function deleteUser(id: string) {
    if (!confirm('Are you sure you want to delete this user?')) return;
    const res = await authenticatedFetch(`/admin/users/${id}`, { method: 'DELETE' });
    if (res.ok) {
      toast.success('User deleted');
      users = users.filter((u) => u.id !== id);
    } else {
      toast.error('Failed to delete user');
    }
  }

  async function saveUser() {
    if (!editingUser) return;

    // Parse flags from comma separated string
    const flagsArray = editFlags
      .split(',')
      .map((f) => f.trim())
      .filter((f) => f);

    const payload: any = {
      username: editUsername,
      email: editEmail,
      flags: flagsArray
    };

    if (editPassword) {
      payload.pass = editPassword;
    }

    const res = await authenticatedFetch(`/admin/users/${editingUser.id}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload)
    });
    if (res.ok) {
      const data = await res.json();
      toast.success('User updated');
      users = users.map((u) => (u.id === editingUser.id ? data.user : u));
      editModalOpen = false;
    } else {
      toast.error('Failed to update user');
    }
  }

  async function createInvite() {
    if (!newInviteToken) return;
    const res = await authenticatedFetch(`/admin/invites`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ token: newInviteToken })
    });
    if (res.ok) {
      toast.success('Invite created');
      const data = await res.json();
      invites = [...invites, data];
      newInviteToken = '';
    } else {
      toast.error('Failed to create invite');
    }
  }

  async function deleteInvite(id: string) {
    if (!confirm('Delete this invite?')) return;
    const res = await authenticatedFetch(`/admin/invites/${id}`, { method: 'DELETE' });
    if (res.ok) {
      toast.success('Invite deleted');
      invites = invites.filter((i) => i.id !== id);
    } else {
      toast.error('Failed to delete invite');
    }
  }
</script>

<div class="mx-auto flex max-w-5xl flex-col gap-8 py-6">
  <div class="border-border flex items-center gap-3 border-b pb-4">
    <ShieldCheck class="text-primary size-8" />
    <div>
      <h1 class="text-3xl font-bold tracking-tight">Admin Dashboard</h1>
      <p class="text-muted-foreground text-sm">Manage users and invite codes</p>
    </div>
  </div>

  {#if loading}
    <div class="text-muted-foreground animate-pulse py-10 text-center">Loading admin data...</div>
  {:else}
    <!-- Users Section -->
    <section class="flex flex-col gap-4">
      <h2 class="text-2xl font-bold">Users</h2>
      <div class="border-border bg-card overflow-hidden rounded-xl border">
        <table class="w-full text-left text-sm">
          <thead class="bg-muted/50 border-border border-b">
            <tr>
              <th class="p-3 font-semibold">ID</th>
              <th class="p-3 font-semibold">Username</th>
              <th class="p-3 font-semibold">Email</th>
              <th class="p-3 font-semibold">Flags</th>
              <th class="p-3 text-right font-semibold">Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each users as user}
              <tr class="border-border/50 hover:bg-muted/20 border-b last:border-0">
                <td class="p-3 font-mono text-xs">{user.id}</td>
                <td class="p-3 font-medium">{user.username}</td>
                <td class="text-muted-foreground p-3">{user.email}</td>
                <td class="p-3">
                  {#if user.flags && user.flags.length > 0}
                    <div class="flex gap-1">
                      {#each user.flags as flag}
                        <span
                          class="bg-primary/10 text-primary rounded px-2 py-0.5 text-xs font-medium"
                          >{flag}</span
                        >
                      {/each}
                    </div>
                  {:else}
                    <span class="text-muted-foreground text-xs italic">none</span>
                  {/if}
                </td>
                <td class="p-3 text-right">
                  <button
                    class="btn btn-outline mr-2 h-auto p-1.5 text-xs"
                    onclick={() => {
                      editingUser = user;
                      editUsername = user.username;
                      editEmail = user.email;
                      editFlags = user.flags ? user.flags.join(', ') : '';
                      editPassword = '';
                      editModalOpen = true;
                      showPassword = false;
                    }}
                    title="Edit User"
                  >
                    <Pencil class="size-4" />
                  </button>
                  <button
                    class="btn bg-destructive text-destructive-foreground h-auto p-1.5 text-xs"
                    onclick={() => deleteUser(user.id)}
                    title="Delete User"
                  >
                    <Trash2 class="size-4" />
                  </button>
                </td>
              </tr>
            {/each}
            {#if users.length === 0}
              <tr
                ><td colspan="5" class="text-muted-foreground p-6 text-center">No users found.</td
                ></tr
              >
            {/if}
          </tbody>
        </table>
      </div>
    </section>

    <!-- Invites Section -->
    <section class="flex flex-col gap-4">
      <h2 class="text-2xl font-bold">Invite Codes</h2>
      <div class="flex gap-2">
        <input
          type="text"
          class="input flex-1"
          placeholder="New invite code (e.g. BETA-2024)"
          bind:value={newInviteToken}
        />
        <button
          class="btn bg-primary text-primary-foreground px-4"
          onclick={createInvite}
          disabled={!newInviteToken}
        >
          <Plus class="mr-2 size-4" /> Create Invite
        </button>
      </div>

      <div class="border-border bg-card overflow-hidden rounded-xl border">
        <table class="w-full text-left text-sm">
          <thead class="bg-muted/50 border-border border-b">
            <tr>
              <th class="p-3 font-semibold">ID</th>
              <th class="p-3 font-semibold">Token</th>
              <th class="p-3 font-semibold">Status</th>
              <th class="p-3 text-right font-semibold">Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each invites as invite}
              <tr class="border-border/50 hover:bg-muted/20 border-b last:border-0">
                <td class="p-3 font-mono text-xs">{invite.id}</td>
                <td class="p-3 font-mono font-medium">{invite.token}</td>
                <td class="p-3">
                  {#if invite.used_by}
                    <span
                      class="bg-muted text-muted-foreground rounded px-2 py-0.5 text-xs font-medium"
                      >Used by {invite.used_by}</span
                    >
                  {:else}
                    <span
                      class="rounded bg-emerald-500/10 px-2 py-0.5 text-xs font-medium text-emerald-500"
                      >Available</span
                    >
                  {/if}
                </td>
                <td class="p-3 text-right">
                  <button
                    class="btn bg-destructive text-destructive-foreground h-auto p-1.5 text-xs"
                    onclick={() => deleteInvite(invite.id)}
                    title="Delete Invite"
                  >
                    <Trash2 class="size-4" />
                  </button>
                </td>
              </tr>
            {/each}
            {#if invites.length === 0}
              <tr
                ><td colspan="4" class="text-muted-foreground p-6 text-center"
                  >No invite codes found.</td
                ></tr
              >
            {/if}
          </tbody>
        </table>
      </div>
    </section>
  {/if}
</div>

<Modal bind:open={editModalOpen} title="Edit User">
  {#if editingUser}
    <div class="flex flex-col gap-4">
      <p class="text-muted-foreground -mt-4 text-sm">
        Editing <span class="text-foreground font-bold">{editingUser.username}</span>
      </p>

      <label class="flex flex-col gap-1.5 text-sm font-medium">
        Username
        <input type="text" class="input" bind:value={editUsername} />
      </label>

      <label class="flex flex-col gap-1.5 text-sm font-medium">
        Email
        <input type="email" class="input" bind:value={editEmail} />
      </label>

      <label class="flex flex-col gap-1.5 text-sm font-medium">
        Flags (comma separated)
        <input
          type="text"
          class="input"
          bind:value={editFlags}
          placeholder="e.g. is_admin, beta_tester"
        />
      </label>

      <label class="relative mt-2 flex flex-col gap-1.5 text-sm font-medium">
        New Password <span class="text-muted-foreground text-xs font-normal"
          >(Leave blank to keep current)</span
        >
        <div class="relative">
          <input
            type={showPassword ? 'text' : 'password'}
            class="input w-full pr-10"
            bind:value={editPassword}
          />
          <button
            type="button"
            class="text-muted-foreground hover:text-foreground absolute top-1/2 right-2 -translate-y-1/2 p-1"
            onclick={() => (showPassword = !showPassword)}
            title={showPassword ? 'Hide password' : 'Show password'}
          >
            {#if showPassword}
              <EyeOff class="size-4" />
            {:else}
              <Eye class="size-4" />
            {/if}
          </button>
        </div>
      </label>
      <div class="border-border mt-2 flex justify-end gap-3 border-t pt-4">
        <button
          class="btn btn-outline px-4 py-2"
          onclick={() => {
            editModalOpen = false;
          }}>Cancel</button
        >
        <button class="btn bg-primary text-primary-foreground px-4 py-2" onclick={saveUser}
          >Save Changes</button
        >
      </div>
    </div>
  {/if}
</Modal>
