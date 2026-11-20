<script lang="ts">
  import {
    HStack,
    Icon,
    Selector,
    Text,
    type SelectorOptionData,
    type SelectorOptionType
  } from '@astryx-sv/core';
  import { Plus } from '@lucide/svelte';
  import type { EntryCategory } from '$lib/types';
  import CategoryDot from './CategoryDot.svelte';
  import CategoryForm from './CategoryForm.svelte';
  import Modal from './Modal.svelte';

  let {
    value = $bindable(''),
    categories,
    showLabel = false,
    isRequired = false
  }: {
    value: string;
    categories: EntryCategory[];
    showLabel?: boolean;
    isRequired?: boolean;
  } = $props();

  const CREATE_VALUE = '__create_category__';
  let modalOpen = $state(false);

  let options = $derived<SelectorOptionType[]>([
    ...categories.map((c) => ({ value: c.id, label: c.title })),
    { type: 'divider' },
    { value: CREATE_VALUE, label: 'Create new category', icon: createIcon }
  ]);

  let categoryById = $derived(new Map(categories.map((c) => [c.id, c])));

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

{#snippet categoryRow(option: SelectorOptionData)}
  {@const category = categoryById.get(option.value)}
  <HStack gap={2} vAlign="center">
    {#if category}
      <CategoryDot color={category.color} size="sm" />
    {/if}
    <Text type="body">{option.label}</Text>
  </HStack>
{/snippet}

<Selector
  label="Category"
  isLabelHidden={!showLabel}
  {isRequired}
  placeholder="Select category..."
  width="100%"
  {options}
  {value}
  onChange={handleChange}
  renderOption={categoryRow}
  renderValue={categoryRow}
/>

<Modal bind:open={modalOpen} title="Create New Category">
  <CategoryForm
    onsuccess={(id) => {
      value = id;
      modalOpen = false;
    }}
    oncancel={() => (modalOpen = false)}
  />
</Modal>
