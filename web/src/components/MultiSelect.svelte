<script lang="ts">
  import { HStack, MultiSelector, Text, type MultiSelectorOptionData } from '@astryx-sv/core';
  import CategoryDot from './CategoryDot.svelte';

  let {
    value = $bindable<string[]>([]),
    options,
    placeholder = 'Select options...',
    label = 'Select',
    showLabel = false
  }: {
    value: string[];
    options: { id: string; name: string; color?: string }[];
    placeholder?: string;
    label?: string;
    showLabel?: boolean;
  } = $props();

  let selectorOptions = $derived(options.map((o) => ({ value: String(o.id), label: o.name })));

  let colorById = $derived(new Map(options.map((o) => [String(o.id), o.color])));

  function handleChange(next: string[]) {
    value = next;
  }
</script>

{#snippet renderOption(option: MultiSelectorOptionData)}
  {@const color = colorById.get(String(option.value))}
  <HStack gap={2} vAlign="center">
    {#if color}
      <CategoryDot {color} size="sm" />
    {/if}
    <Text type="body">{option.label}</Text>
  </HStack>
{/snippet}

<MultiSelector
  {label}
  isLabelHidden={!showLabel}
  {placeholder}
  width="100%"
  options={selectorOptions}
  {value}
  onChange={handleChange}
  {renderOption}
  hasSearch={options.length > 8}
  triggerDisplay="labels"
  formatValue={(items) => (items.length === 1 ? items[0].label : `${items.length} selected`)}
/>
