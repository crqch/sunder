<script lang="ts">
  import './layout.css';
  import { LayerProvider, Theme } from '@astryx-sv/core';
  import { neutralTheme } from '@astryx-sv/theme-neutral';
  import { themeStore } from '$lib/themeStore';
  import { onMount } from 'svelte';
  import { initAuth } from '$lib/auth';
  import KeybindListener from '$components/KeybindListener.svelte';
  import KeybindsModal from '$components/KeybindsModal.svelte';
  import ToastBridge from '$components/ToastBridge.svelte';
  import { keybinds } from '$lib/keybinds.svelte';

  $effect(() => {
    let unregTheme = keybinds.register({
      id: 'global.theme',
      name: 'Toggle Theme',
      keys: ['alt+t'],
      global: true,
      action: () => themeStore.toggle()
    });

    let unregHelp = keybinds.register({
      id: 'global.help',
      name: 'Show Keyboard Shortcuts',
      keys: ['alt+?'],
      global: true,
      action: () => keybinds.toggleHelp()
    });

    let unregHelpAlt = keybinds.register({
      id: 'global.help_alt',
      name: 'Show Keyboard Shortcuts',
      keys: ['alt+/'],
      global: true,
      action: () => keybinds.toggleHelp()
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

<Theme theme={neutralTheme} mode={$themeStore}>
  <LayerProvider toast={{ position: 'topEnd', maxVisible: 4 }}>
    <ToastBridge />
    <KeybindListener />
    <KeybindsModal bind:open={keybinds.helpOpen} />
    {@render children()}
  </LayerProvider>
</Theme>
