<script lang="ts">
  import {
    DateInput,
    DateRangeInput,
    type DateRange,
    type DateRangePreset,
    type ISODateString
  } from '@astryx-sv/core';

  let {
    mode = $bindable('range'),
    startDate = $bindable(null),
    endDate = $bindable(null),
    label = 'Date',
    showLabel = false
  }: {
    mode?: 'single' | 'range';
    startDate?: Date | null;
    endDate?: Date | null;
    label?: string;
    showLabel?: boolean;
  } = $props();

  function toISO(d: Date | null): ISODateString | undefined {
    if (!d) return undefined;
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${d.getFullYear()}-${m}-${day}` as ISODateString;
  }

  function fromISO(value: string): Date {
    const [y, m, d] = value.split('-').map(Number);
    return new Date(y, m - 1, d);
  }

  function monthsAgo(months: number): DateRange {
    const end = new Date();
    const start = new Date();
    start.setMonth(start.getMonth() - months);
    return { start: toISO(start)!, end: toISO(end)! };
  }

  const presets: DateRangePreset[] = [
    { label: '1M', getRange: () => monthsAgo(1) },
    { label: '3M', getRange: () => monthsAgo(3) },
    { label: '6M', getRange: () => monthsAgo(6) },
    { label: '1Y', getRange: () => monthsAgo(12) }
  ];

  let rangeValue = $derived<DateRange | null>(
    startDate && endDate ? { start: toISO(startDate)!, end: toISO(endDate)! } : null
  );

  function handleSingleChange(value: string | undefined) {
    if (!value) {
      startDate = null;
      endDate = null;
      return;
    }
    const picked = fromISO(value);
    startDate = picked;
    endDate = picked;
  }

  function handleRangeChange(value: DateRange | null) {
    if (!value) {
      startDate = null;
      endDate = null;
      return;
    }
    startDate = fromISO(value.start);
    endDate = fromISO(value.end);
  }
</script>

{#if mode === 'single'}
  <DateInput
    {label}
    isLabelHidden={!showLabel}
    width="100%"
    value={toISO(startDate)}
    onChange={handleSingleChange}
    hasClear
    format="date"
  />
{:else}
  <DateRangeInput
    {label}
    isLabelHidden={!showLabel}
    width="100%"
    value={rangeValue}
    onChange={handleRangeChange}
    {presets}
    numberOfMonths={1}
  />
{/if}
