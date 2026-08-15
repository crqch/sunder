<script lang="ts">
  import { keybinds } from '$lib/keybinds.svelte';

  let buffer: string[] = [];
  let bufferTimer: ReturnType<typeof setTimeout> | null = null;

  function handleKeydown(e: KeyboardEvent) {
    if (['INPUT', 'TEXTAREA', 'SELECT'].includes((e.target as HTMLElement).tagName)) {
      return; // ignore when typing
    }

    if (['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) return;

    let parts = [];
    if (e.ctrlKey) parts.push('ctrl');
    if (e.altKey) parts.push('alt');
    // For shift, we don't always add it if it's already encoded in the character (like '?')
    // but we'll add it if it's a letter. Let's just always push shift if it's there.
    // Actually, "Alt+?" is better checked directly.
    let key = e.key === ' ' ? 'space' : e.key.toLowerCase();

    // special handling for "?"
    if (key === '?') {
      if (e.altKey) parts = ['alt', '?'];
      else parts = ['?'];
    } else {
      if (e.shiftKey) parts.push('shift');
      parts.push(key);
    }

    let stroke = parts.join('+');
    buffer.push(stroke);

    if (bufferTimer) clearTimeout(bufferTimer);
    bufferTimer = setTimeout(() => {
      buffer = [];
    }, 1500);

    // try matching commands
    let matched = false;
    for (let cmd of keybinds.commands) {
      if (cmd.keys.length <= buffer.length) {
        let slice = buffer.slice(buffer.length - cmd.keys.length);
        // compare slice with cmd.keys (normalized)
        let cmdKeysNorm = cmd.keys.map((k) => k.toLowerCase());
        if (slice.join(',') === cmdKeysNorm.join(',')) {
          e.preventDefault();
          buffer = [];
          cmd.action();
          matched = true;
          break;
        }
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />
