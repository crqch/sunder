<script lang="ts">
  import {
    Selector,
    Text,
    Token,
    type SelectorOptionData,
    type SelectorOptionType
  } from '@astryx-sv/core';
  import { currencyStore } from '$lib/currency.svelte';
  import type { Currency } from '$lib/types';

  let {
    value = $bindable<string>(''),
    currencies = undefined,
    allowEmpty = false,
    emptyLabel = 'Use default currency',
    placeholder = 'Select currency...',
    disabled = false,
    label = 'Currency',
    showLabel = false,
    isRequired = false,
    size = 'md',
    onchange
  }: {
    value: string;
    currencies?: Currency[];
    allowEmpty?: boolean;
    emptyLabel?: string;
    placeholder?: string;
    disabled?: boolean;
    label?: string;
    showLabel?: boolean;
    isRequired?: boolean;
    size?: 'sm' | 'md' | 'lg';
    onchange?: (code: string) => void;
  } = $props();

  let availableCurrencies = $derived<Currency[]>(currencies ?? currencyStore.currencies);

  // Normalize incoming codes to uppercase so Astryx Selector's strict `===`
  // option matching never falls back to the placeholder for lowercase inputs.
  let normalizedValue = $derived(value ? value.trim().toUpperCase() : '');

  // The selected code must stay visible even before the currency list finishes
  // loading (or when it is not part of the active list): synthesize a fallback
  // option so the trigger never falls back to the placeholder for a real value.
  let selectedIsKnown = $derived(
    normalizedValue !== '' &&
      availableCurrencies.some((c) => c.code.toUpperCase() === normalizedValue)
  );

  let options = $derived<SelectorOptionType[]>([
    ...(allowEmpty ? [{ value: '', label: emptyLabel }] : []),
    ...availableCurrencies.map((c) => {
      const code = c.code.toUpperCase();
      return {
        value: code,
        label: `${code} ${c.symbol} ${c.name}`
      };
    }),
    ...(normalizedValue !== '' && !selectedIsKnown
      ? [{ value: normalizedValue, label: normalizedValue }]
      : [])
  ]);

  let currencyByCode = $derived(new Map(availableCurrencies.map((c) => [c.code.toUpperCase(), c])));

  function handleChange(next: string | null) {
    const clean = next ? next.trim().toUpperCase() : '';
    value = clean;
    onchange?.(clean);
  }
</script>

{#snippet currencyRow(option: SelectorOptionData)}
  {@const currency = currencyByCode.get(option.value.toUpperCase())}
  {#if currency}
    <span class="sunder-currency-row">
      <Token label={currency.code.toUpperCase()} size="sm" />
      <span class="sunder-currency-symbol">
        <Text type="supporting" color="secondary">{currency.symbol}</Text>
      </span>
      <span class="sunder-currency-name">
        <Text type="body" maxLines={1}>{currency.name}</Text>
      </span>
    </span>
  {:else if option.value}
    <Token label={option.value.toUpperCase()} size="sm" />
  {:else}
    <Text type="body" color="secondary">{option.label}</Text>
  {/if}
{/snippet}

<Selector
  {label}
  isLabelHidden={!showLabel}
  {isRequired}
  placeholder={allowEmpty && !normalizedValue ? emptyLabel : placeholder}
  width="100%"
  {size}
  {options}
  value={normalizedValue}
  onChange={handleChange}
  isDisabled={disabled}
  hasSearch
  searchPlaceholder="Search code, name, symbol..."
  renderOption={currencyRow}
  renderValue={currencyRow}
/>

<style>
  .sunder-currency-row {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-2);
    min-width: 0;
    max-width: 100%;
    width: 100%;
  }

  .sunder-currency-symbol {
    flex: 0 0 auto;
  }

  .sunder-currency-name {
    min-width: 0;
    flex: 1 1 auto;
    overflow: hidden;
  }
</style>
