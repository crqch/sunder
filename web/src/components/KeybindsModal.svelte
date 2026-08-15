<script lang="ts">
  import Modal from '$components/Modal.svelte';
  import { keybinds } from '$lib/keybinds.svelte';

  let { open = $bindable(false) } = $props<{ open: boolean }>();

  let groupedCommands = $derived(() => {
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
  <div class="space-y-4 pt-2">
    {#each groupedCommands() as group}
      <div class="border-border/50 flex items-center justify-between border-b pb-3 last:border-0">
        <span class="text-sm font-medium">{group.name}</span>
        <div class="flex flex-wrap items-center justify-end gap-2">
          {#each group.bindings as keysList, bIndex}
            <div class="flex items-center gap-2">
              {#each keysList as chord, i}
                <div class="flex items-center gap-1">
                  {#each chord.split('+') as key}
                    <kbd
                      class="border-border bg-muted/80 text-muted-foreground rounded border px-1.5 py-0.5 font-mono text-[11px] uppercase shadow-sm"
                      >{key}</kbd
                    >
                  {/each}
                </div>
                {#if i < keysList.length - 1}
                  <span class="text-muted-foreground px-1 text-xs font-medium">then</span>
                {/if}
              {/each}
            </div>
            {#if bIndex < group.bindings.length - 1}
              <span class="text-muted-foreground px-1 text-xs font-medium">or</span>
            {/if}
          {/each}
        </div>
      </div>
    {/each}
    {#if keybinds.commands.length === 0}
      <p class="text-muted-foreground text-center text-sm">No keybinds registered.</p>
    {/if}
  </div>
</Modal>
