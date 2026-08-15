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
        const btn =
          document.getElementById('theme-toggle-btn') ||
          document.getElementById('theme-toggle-btn-mobile');
        if (btn) {
          const rect = btn.getBoundingClientRect();
          themeStore.toggle({
            clientX: rect.left + rect.width / 2,
            clientY: rect.top + rect.height / 2
          } as MouseEvent);
        } else {
          themeStore.toggle();
        }
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
