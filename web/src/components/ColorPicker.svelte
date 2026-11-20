<script lang="ts">
  import { Field, Grid, HStack, Stack, StackItem, TextInput, VStack } from '@astryx-sv/core';

  let { value = $bindable('#ef4444') }: { value: string } = $props();

  const PRESET_COLORS = [
    '#ef4444',
    '#f97316',
    '#f59e0b',
    '#84cc16',
    '#22c55e',
    '#10b981',
    '#06b6d4',
    '#3b82f6',
    '#6366f1',
    '#8b5cf6',
    '#d946ef',
    '#f43f5e',
    '#000000',
    '#52525b',
    '#78716c'
  ];
</script>

<VStack gap={3}>
  <Field label="Color" inputID="category-color" isGroupLabel>
    <HStack gap={3} vAlign="center">
      <Stack as="label" class="sunder-color-picker-swatch" style="--swatch-color: {value}">
        <input
          id="category-color"
          type="color"
          bind:value
          class="sunder-color-picker-native"
          aria-label="Pick a custom color"
        />
      </Stack>
      <StackItem size="fill">
        <TextInput
          label="Hex color"
          isLabelHidden
          {value}
          onChange={(v) => (value = v)}
          placeholder="#ef4444"
        />
      </StackItem>
    </HStack>
  </Field>
  <Grid columns={8} gap={2}>
    {#each PRESET_COLORS as preset (preset)}
      <Stack
        as="button"
        {...{ type: 'button' }}
        class={value === preset
          ? 'sunder-color-swatch sunder-color-swatch-selected'
          : 'sunder-color-swatch'}
        style="--swatch-color: {preset}"
        aria-label="Select color {preset}"
        aria-pressed={value === preset}
        onclick={() => (value = preset)}
      />
    {/each}
  </Grid>
</VStack>
