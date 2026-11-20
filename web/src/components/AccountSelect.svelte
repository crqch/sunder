<script lang="ts">
  import { Icon, Selector, type SelectorOptionType } from '@astryx-sv/core';
  import { Plus } from '@lucide/svelte';
  import type { Account } from '$lib/types';
  import AccountForm from './AccountForm.svelte';
  import Modal from './Modal.svelte';

  let {
    value = $bindable(''),
    accounts,
    showLabel = false,
    isRequired = false
  }: {
    value: string;
    accounts: Account[];
    showLabel?: boolean;
    isRequired?: boolean;
  } = $props();

  const CREATE_VALUE = '__create_account__';
  let modalOpen = $state(false);

  let options = $derived<SelectorOptionType[]>([
    ...accounts.map((a) => ({ value: a.id, label: a.name })),
    { type: 'divider' },
    { value: CREATE_VALUE, label: 'Create new account', icon: createIcon }
  ]);

  function handleChange(next: string | null) {
    if (next === CREATE_VALUE) {
      modalOpen = true;
      return;
    }
    value = next ?? '';
  }
</script>

{#snippet createIcon()}
  <Icon icon={Plus} size="sm" />
{/snippet}

<Selector
  label="Account"
  isLabelHidden={!showLabel}
  {isRequired}
  placeholder="Select account..."
  width="100%"
  {options}
  {value}
  onChange={handleChange}
/>

<Modal bind:open={modalOpen} title="Create New Account">
  <AccountForm
    onsuccess={(id) => {
      value = id;
      modalOpen = false;
    }}
    oncancel={() => (modalOpen = false)}
  />
</Modal>
