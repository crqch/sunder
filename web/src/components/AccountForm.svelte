<script lang="ts">
  import { Button, FormLayout, HStack, Icon, TextInput } from '@astryx-sv/core';
  import { Save } from '@lucide/svelte';
  import { createId } from '@paralleldrive/cuid2';
  import { db } from '$lib/db';
  import CurrencySelect from '$components/CurrencySelect.svelte';
  import { normalizeCurrencyCode } from '$lib/currency.svelte';

  let {
    onsuccess,
    oncancel,
    isDirty = $bindable(false)
  }: {
    onsuccess: (id: string) => void;
    oncancel?: () => void;
    isDirty?: boolean;
  } = $props();

  let name = $state('');
  let currencyCode = $state('');

  $effect(() => {
    isDirty = name.trim() !== '' || currencyCode !== '';
  });

  async function saveAccount(e: SubmitEvent) {
    e.preventDefault();
    if (!name.trim()) return;

    const now = new Date().toISOString();
    const id = createId();
    const newAccount = {
      id,
      name: name.trim(),
      currency_code: currencyCode ? normalizeCurrencyCode(currencyCode) : null,
      deleted_at: null,
      created_at: now,
      updated_at: now
    };

    await db.accounts.add(newAccount);
    onsuccess(id);
  }
</script>

{#snippet saveIcon()}
  <Icon icon={Save} size="sm" />
{/snippet}

<form onsubmit={saveAccount}>
  <FormLayout defaultOptionality="optional">
    <TextInput
      label="Account name"
      placeholder="e.g. Checking, Savings, Cash"
      value={name}
      onChange={(v) => (name = v)}
      isRequired
      hasAutoFocus
    />
    <CurrencySelect
      bind:value={currencyCode}
      label="Default currency"
      allowEmpty={true}
      emptyLabel="Use account owner's default currency"
    />
    <HStack gap={3} hAlign="end" paddingBlockStart={2}>
      {#if oncancel}
        <Button label="Cancel" variant="secondary" onclick={oncancel} />
      {/if}
      <Button label="Create account" type="submit" variant="primary" icon={saveIcon} />
    </HStack>
  </FormLayout>
</form>
