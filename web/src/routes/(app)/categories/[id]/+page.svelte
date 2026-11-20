<script lang="ts">
  import {
    AlertDialog,
    Badge,
    BreadcrumbItem,
    Breadcrumbs,
    Button,
    Card,
    Divider,
    EmptyState,
    FormLayout,
    Grid,
    Heading,
    HStack,
    Icon,
    Layout,
    LayoutContent,
    LayoutHeader,
    List,
    ListItem,
    Stack,
    Text,
    TextArea,
    TextInput,
    VStack
  } from '@astryx-sv/core';
  import { Check, Pencil, Plus, Trash2, X } from '@lucide/svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { liveQuery } from 'dexie';
  import { db } from '$lib/db';
  import type { EntryCategory, AccountEntry, Account } from '$lib/types';
  import { toast } from '$lib/toast.svelte';
  import Modal from '$components/Modal.svelte';
  import EntryForm from '$components/EntryForm.svelte';
  import ColorPicker from '$components/ColorPicker.svelte';
  import CategoryDot from '$components/CategoryDot.svelte';
  import { currencyStore, normalizeCurrencyCode } from '$lib/currency.svelte';

  let id = $derived($page.params.id || '');

  let category = $state<EntryCategory | null>(null);
  let entries = $state<AccountEntry[]>([]);
  let accounts = $state<Account[]>([]);

  let editing = $state(false);
  let editName = $state('');
  let editDescription = $state('');
  let editColor = $state('');

  let createEntryModalOpen = $state(false);
  let deleteConfirmOpen = $state(false);

  $effect(() => {
    const subCat = liveQuery(() => db.entry_categories.get(id)).subscribe({
      next: (v) => {
        if (v && v.deleted_at) {
          category = null;
        } else {
          category = v || null;
          if (category && !editing) {
            editName = category.title;
            editDescription = category.description || '';
            editColor = category.color;
          }
        }
      }
    });
    const subEnt = liveQuery(() =>
      db.account_entries
        .where('category_id')
        .equals(id)
        .filter((e) => !e.deleted_at)
        .reverse()
        .sortBy('date')
    ).subscribe({ next: (v) => (entries = v) });
    const subAcc = liveQuery(() => db.accounts.toArray()).subscribe({
      next: (v) => (accounts = v)
    });

    return () => {
      subCat.unsubscribe();
      subEnt.unsubscribe();
      subAcc.unsubscribe();
    };
  });

  function requestDelete() {
    if (!category) return;
    if (entries.length > 0) {
      toast.error(`Cannot delete category while it has ${entries.length} active entries.`);
      return;
    }
    deleteConfirmOpen = true;
  }

  async function handleDelete() {
    if (!category) return;
    const now = new Date().toISOString();
    await db.entry_categories.update(id, {
      deleted_at: now,
      updated_at: now
    });
    deleteConfirmOpen = false;
    goto('/categories');
  }

  async function handleSaveEdit(e: SubmitEvent) {
    e.preventDefault();
    if (!category || !editName.trim()) return;
    await db.entry_categories.update(id, {
      title: editName.trim(),
      description: editDescription.trim() || null,
      color: editColor,
      updated_at: new Date().toISOString()
    });
    editing = false;
    toast.success('Category updated');
  }

  function cancelEdit() {
    editing = false;
    if (category) {
      editName = category.title;
      editDescription = category.description || '';
      editColor = category.color;
    }
  }

  function getAccountName(accId: string) {
    return accounts.find((a) => a.id === accId)?.name || 'Unknown';
  }

  let defaultCurrency = $derived(currencyStore.defaultCurrencyCode);
  let categoryTotal = $derived(
    entries.reduce((sum, e) => sum + currencyStore.convertEntryToDefault(e), 0)
  );
  let averageEntry = $derived(entries.length > 0 ? categoryTotal / entries.length : 0);
</script>

{#snippet editIcon()}
  <Icon icon={Pencil} size="sm" />
{/snippet}
{#snippet deleteIcon()}
  <Icon icon={Trash2} size="sm" />
{/snippet}
{#snippet addIcon()}
  <Icon icon={Plus} size="sm" />
{/snippet}
{#snippet saveIcon()}
  <Icon icon={Check} size="sm" />
{/snippet}
{#snippet cancelIcon()}
  <Icon icon={X} size="sm" />
{/snippet}
{#snippet notFoundActions()}
  <Button label="Back to categories" href="/categories" />
{/snippet}

{#snippet header()}
  <LayoutHeader hasDivider padding={5}>
    <VStack gap={2}>
      <Breadcrumbs variant="supporting">
        <BreadcrumbItem href="/categories">Categories</BreadcrumbItem>
        <BreadcrumbItem isCurrent>{category?.title ?? 'Category'}</BreadcrumbItem>
      </Breadcrumbs>

      {#if category}
        <HStack hAlign="between" vAlign="center" wrap="wrap" gap={3}>
          <HStack gap={3} vAlign="center">
            <CategoryDot color={category.color} size="lg" />
            <VStack gap={0.5}>
              <Heading level={1} maxLines={1}>{category.title}</Heading>
              {#if category.description}
                <Text type="supporting" color="secondary">{category.description}</Text>
              {/if}
            </VStack>
          </HStack>

          {#if !editing}
            <HStack gap={2} vAlign="center">
              <Button
                label="Add entry"
                variant="primary"
                size="sm"
                icon={addIcon}
                onclick={() => (createEntryModalOpen = true)}
              />
              <Button
                label="Edit"
                variant="secondary"
                size="sm"
                icon={editIcon}
                onclick={() => (editing = true)}
              />
              <Button
                label="Delete"
                variant="destructive"
                size="sm"
                icon={deleteIcon}
                onclick={requestDelete}
              />
            </HStack>
          {/if}
        </HStack>
      {/if}
    </VStack>
  </LayoutHeader>
{/snippet}

{#snippet content()}
  <LayoutContent padding={6} label="Category details">
    {#if !category}
      <EmptyState title="Category not found or deleted" actions={notFoundActions} />
    {:else}
      <VStack gap={5}>
        {#if editing}
          <Card padding={5}>
            <form onsubmit={handleSaveEdit}>
              <FormLayout>
                <Heading level={3}>Edit category</Heading>
                <TextInput
                  label="Category title"
                  value={editName}
                  onChange={(v) => (editName = v)}
                  isRequired
                  hasAutoFocus
                />
                <TextArea
                  label="Description"
                  rows={2}
                  value={editDescription}
                  onChange={(v) => (editDescription = v)}
                />
                <ColorPicker bind:value={editColor} />
                <HStack gap={2} hAlign="end">
                  <Button
                    label="Cancel"
                    variant="secondary"
                    icon={cancelIcon}
                    onclick={cancelEdit}
                  />
                  <Button label="Save changes" type="submit" variant="primary" icon={saveIcon} />
                </HStack>
              </FormLayout>
            </form>
          </Card>
        {/if}

        <Grid columns={{ minWidth: 200, max: 3 }} gap={4}>
          <Card padding={4}>
            <VStack gap={1}>
              <Text type="supporting" color="secondary">Net total ({defaultCurrency})</Text>
              <Text
                size="xl"
                weight="semibold"
                hasTabularNumbers
                class={categoryTotal >= 0 ? 'sunder-positive' : 'sunder-negative'}
              >
                {currencyStore.formatAmount(categoryTotal, defaultCurrency, { showPlus: true })}
              </Text>
            </VStack>
          </Card>

          <Card padding={4}>
            <VStack gap={1}>
              <Text type="supporting" color="secondary">Transactions</Text>
              <Text size="xl" weight="semibold" hasTabularNumbers>{entries.length}</Text>
            </VStack>
          </Card>

          <Card padding={4}>
            <VStack gap={1}>
              <Text type="supporting" color="secondary">Average per entry</Text>
              <Text
                size="xl"
                weight="semibold"
                hasTabularNumbers
                class={entries.length === 0
                  ? undefined
                  : averageEntry >= 0
                    ? 'sunder-positive'
                    : 'sunder-negative'}
              >
                {entries.length === 0
                  ? '—'
                  : currencyStore.formatAmount(averageEntry, defaultCurrency, { showPlus: true })}
              </Text>
            </VStack>
          </Card>
        </Grid>

        <Card padding={0}>
          <VStack gap={0}>
            <HStack padding={4} hAlign="between" vAlign="center" wrap="wrap" gap={3}>
              <HStack gap={2} vAlign="center">
                <Heading level={3}>Transactions</Heading>
                <Badge label={String(entries.length)} />
              </HStack>
              <Button
                label="Add entry"
                variant="secondary"
                size="sm"
                icon={addIcon}
                onclick={() => (createEntryModalOpen = true)}
              />
            </HStack>
            <Divider />

            {#if entries.length === 0}
              <Stack padding={6}>
                <EmptyState
                  isCompact
                  title="No transactions for this category yet"
                  description="Add an entry tagged with this category to see it here."
                />
              </Stack>
            {:else}
              <List hasDividers edge="bleed">
                {#each entries as entry (entry.id)}
                  {@const entryCode = normalizeCurrencyCode(entry.currency_code)}
                  {@const isForeign = entryCode !== defaultCurrency}
                  {@const converted = isForeign
                    ? currencyStore.tryConvert(
                        entry.amount,
                        entryCode,
                        defaultCurrency,
                        entry.date || entry.created_at
                      )
                    : null}
                  {#snippet entryEnd()}
                    <VStack gap={0} hAlign="end">
                      <Text
                        weight="semibold"
                        hasTabularNumbers
                        class={entry.amount >= 0 ? 'sunder-positive' : 'sunder-negative'}
                      >
                        {currencyStore.formatAmount(entry.amount, entryCode, { showPlus: true })}
                      </Text>
                      {#if converted?.converted}
                        <Text type="supporting" color="secondary" hasTabularNumbers>
                          ≈ {currencyStore.formatAmount(converted.amount, defaultCurrency, {
                            showPlus: true
                          })}
                        </Text>
                      {/if}
                      <Text type="supporting" color="secondary">
                        {new Date(entry.date || entry.created_at).toLocaleDateString()}
                      </Text>
                    </VStack>
                  {/snippet}
                  <ListItem
                    label={entry.title}
                    description="{getAccountName(entry.account_id)}{entry.location
                      ? ` · ${entry.location}`
                      : ''}"
                    href="/entries/{entry.id}"
                    endContent={entryEnd}
                  />
                {/each}
              </List>
            {/if}
          </VStack>
        </Card>
      </VStack>
    {/if}
  </LayoutContent>
{/snippet}

<Layout height="fill" contentWidth={960} {header} {content} />

<Modal bind:open={createEntryModalOpen} title="New Entry">
  <EntryForm
    initialCategoryId={id}
    onsuccess={() => {
      createEntryModalOpen = false;
      toast.success('Entry created');
    }}
    oncancel={() => (createEntryModalOpen = false)}
  />
</Modal>

<AlertDialog
  isOpen={deleteConfirmOpen}
  onOpenChange={(next) => (deleteConfirmOpen = next)}
  title="Delete {category?.title ?? 'category'}?"
  description="The category will be marked as deleted and disappear from your lists."
  actionLabel="Delete category"
  actionVariant="destructive"
  onAction={handleDelete}
/>
