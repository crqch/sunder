<script lang="ts">
  import { Button, FormLayout, HStack, Icon, TextArea, TextInput } from '@astryx-sv/core';
  import { Save } from '@lucide/svelte';
  import { createId } from '@paralleldrive/cuid2';
  import { db } from '$lib/db';
  import ColorPicker from './ColorPicker.svelte';

  let {
    onsuccess,
    oncancel,
    isDirty = $bindable(false)
  }: {
    onsuccess: (id: string) => void;
    oncancel?: () => void;
    isDirty?: boolean;
  } = $props();

  let title = $state('');
  let description = $state('');
  let color = $state('#ef4444');

  $effect(() => {
    isDirty = title.trim() !== '' || description.trim() !== '' || color !== '#ef4444';
  });

  async function saveCategory(e: SubmitEvent) {
    e.preventDefault();
    if (!title.trim()) return;

    const now = new Date().toISOString();
    const id = createId();
    const newCategory = {
      id,
      title,
      description: description.trim() || null,
      color,
      deleted_at: null,
      created_at: now,
      updated_at: now
    };

    await db.entry_categories.add(newCategory);
    onsuccess(id);
  }
</script>

{#snippet saveIcon()}
  <Icon icon={Save} size="sm" />
{/snippet}

<form onsubmit={saveCategory}>
  <FormLayout defaultOptionality="optional">
    <TextInput
      label="Category title"
      placeholder="e.g., Groceries, Rent, Utilities..."
      value={title}
      onChange={(v) => (title = v)}
      isRequired
      hasAutoFocus
    />
    <TextArea
      label="Description"
      placeholder="Brief description..."
      rows={2}
      value={description}
      onChange={(v) => (description = v)}
    />
    <ColorPicker bind:value={color} />
    <HStack gap={3} hAlign="end" paddingBlockStart={2}>
      {#if oncancel}
        <Button label="Cancel" variant="secondary" onclick={oncancel} />
      {/if}
      <Button label="Create category" type="submit" variant="primary" icon={saveIcon} />
    </HStack>
  </FormLayout>
</form>
