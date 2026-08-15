<script lang="ts">
  import type { Account } from '$lib/types';
  import { ChevronDown, Plus } from '@lucide/svelte';
  import { fly } from 'svelte/transition';
  import AccountForm from './AccountForm.svelte';
  import Modal from './Modal.svelte';

  let { value = $bindable(0), accounts } = $props<{
    value: number;
    accounts: Account[];
  }>();

  let open = $state(false);
  let modalOpen = $state(false);

  let containerRef: HTMLDivElement;

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      open = false;
    }
  }

  function toggleOpen() {
    open = !open;
  }

  function selectOption(id: number) {
    value = id;
    open = false;
  }

  function openModal() {
    open = false;
    modalOpen = true;
  }

  function onCreated(newId: number) {
    value = newId;
    modalOpen = false;
  }

  $effect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (open && containerRef && !containerRef.contains(event.target as Node)) {
        open = false;
      }
    }
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  });

  let selectedName = $derived(accounts.find((a) => a.id === value)?.name || 'Select Account...');
</script>

<div
  class="relative"
  bind:this={containerRef}
  onkeydown={handleKeydown}
  role="combobox"
  aria-expanded={open}
  aria-haspopup="listbox"
  aria-controls="account-list"
>
  <button
    type="button"
    onclick={toggleOpen}
    class="bg-background border-border/50 focus:ring-primary/50 focus:border-primary flex w-full items-center justify-between rounded-lg border p-2.5 text-sm transition-all focus:ring-2 focus:outline-none"
  >
    <span>{selectedName}</span>
    <ChevronDown
      size={16}
      class="text-muted-foreground transition-transform {open ? 'rotate-180' : ''}"
    />
  </button>

  {#if open}
    <div
      class="bg-card border-border/50 absolute z-10 mt-1 flex w-full origin-top flex-col overflow-hidden rounded-lg border shadow-lg"
      id="account-list"
      role="listbox"
      transition:fly={{ y: -10, duration: 200 }}
      style="perspective: 1000px;"
    >
      <div class="max-h-60 overflow-y-auto p-1">
        {#each accounts as acc}
          <button
            type="button"
            role="option"
            aria-selected={value === acc.id}
            onclick={() => selectOption(acc.id)}
            class="hover:bg-muted focus:bg-muted w-full rounded-md px-3 py-2 text-left text-sm transition-colors focus:outline-none {value ===
            acc.id
              ? 'bg-primary/10 text-primary font-medium'
              : 'text-foreground'}"
          >
            {acc.name}
          </button>
        {/each}
        {#if accounts.length === 0}
          <div class="text-muted-foreground px-3 py-2 text-center text-sm">No accounts</div>
        {/if}
      </div>
      <div class="border-border/50 bg-muted/30 border-t p-1">
        <button
          type="button"
          onclick={openModal}
          class="text-primary hover:bg-primary/10 focus:bg-primary/10 flex w-full items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors focus:outline-none"
        >
          <Plus size={16} /> Create new account
        </button>
      </div>
    </div>
  {/if}
</div>

<Modal bind:open={modalOpen} title="Create New Account">
  <AccountForm onsuccess={onCreated} oncancel={() => (modalOpen = false)} />
</Modal>
