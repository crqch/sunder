<script lang="ts">
  import { Selector } from '@astryx-sv/core';

  let {
    value = $bindable(),
    options,
    placeholder = 'Select...',
    label = 'Select',
    showLabel = false,
    size = 'md',
    onchange
  }: {
    value: string | number;
    options: { value: string | number; label: string }[];
    placeholder?: string;
    label?: string;
    showLabel?: boolean;
    size?: 'sm' | 'md' | 'lg';
    onchange?: (value: string | number) => void;
  } = $props();

  let selectorOptions = $derived(options.map((o) => ({ value: String(o.value), label: o.label })));

  function handleChange(next: string) {
    const match = options.find((o) => String(o.value) === next);
    if (match) {
      value = match.value;
      onchange?.(match.value);
    }
  }
</script>

<Selector
  {label}
  isLabelHidden={!showLabel}
  {placeholder}
  {size}
  width="100%"
  options={selectorOptions}
  value={String(value ?? '')}
  onChange={handleChange}
/>
