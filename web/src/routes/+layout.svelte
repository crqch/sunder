<script lang="ts">
  import { themeStore } from '$lib/themeStore';
  import { onMount } from 'svelte';
  import { initAuth } from '$lib/auth';
  import KeybindListener from '$components/KeybindListener.svelte';
  import KeybindsModal from '$components/KeybindsModal.svelte';
  import { keybinds } from '$lib/keybinds.svelte';

  let showKeybinds = $state(false);

  $effect(() => {
    let unregTheme = keybinds.register({
      id: 'global.theme',
      name: 'Toggle Theme',
      keys: ['alt+t'],
      global: true,
      action: () => {
        themeStore.set($themeStore === 'dark' ? 'light' : 'dark');
      }
    });

    let unregHelp = keybinds.register({
      id: 'global.help',
      name: 'Show Keyboard Shortcuts',
      keys: ['alt+?'],
      global: true,
      action: () => {
        showKeybinds = !showKeybinds;
      }
    });

    let unregHelpAlt = keybinds.register({
      id: 'global.help_alt',
      name: 'Show Keyboard Shortcuts',
      keys: ['alt+/'],
      global: true,
      action: () => {
        showKeybinds = !showKeybinds;
      }
    });

    return () => {
      unregTheme();
      unregHelp();
      unregHelpAlt();
    };
  });

  onMount(() => {
    initAuth();
  });

  let { children } = $props();
</script>

<KeybindListener />
<KeybindsModal bind:open={showKeybinds} />

<div class="text-foreground bg-background dark">
  {@render children()}
</div>
