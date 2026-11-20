<script lang="ts">
  import { HStack, List, ListItem, Text, Kbd } from '@astryx-sv/core';
  import Modal from '$components/Modal.svelte';
  import { keybinds } from '$lib/keybinds.svelte';

  let { open = $bindable(false) }: { open: boolean } = $props();

  let groupedCommands = $derived.by(() => {
    const groups: { name: string; bindings: string[][] }[] = [];
    const map = new Map<string, number>();

    for (const cmd of keybinds.commands) {
      if (!map.has(cmd.name)) {
        map.set(cmd.name, groups.length);
        groups.push({ name: cmd.name, bindings: [cmd.keys] });
      } else {
        const index = map.get(cmd.name)!;
        groups[index].bindings.push(cmd.keys);
      }
    }
    return groups;
  });
</script>

<Modal bind:open title="Keyboard Shortcuts">
  {#if keybinds.commands.length === 0}
    <Text type="supporting" color="secondary">No keybinds registered.</Text>
  {:else}
    <List hasDividers edge="flush">
      {#each groupedCommands as group (group.name)}
        {#snippet groupKeys()}
          <HStack gap={2} vAlign="center" wrap="wrap" hAlign="end">
            {#each group.bindings as keysList, bIndex}
              <HStack gap={1} vAlign="center">
                {#each keysList as chord, i}
                  <Kbd keys={chord} />
                  {#if i < keysList.length - 1}
                    <Text type="supporting" color="secondary">then</Text>
                  {/if}
                {/each}
              </HStack>
              {#if bIndex < group.bindings.length - 1}
                <Text type="supporting" color="secondary">or</Text>
              {/if}
            {/each}
          </HStack>
        {/snippet}
        <ListItem label={group.name} endContent={groupKeys} />
      {/each}
    </List>
  {/if}
</Modal>
