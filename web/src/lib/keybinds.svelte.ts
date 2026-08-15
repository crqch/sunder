import { untrack } from 'svelte';

export type KeybindCommand = {
  id: string;
  name: string;
  keys: string[]; // e.g. ['Alt+n', 'a'] or ['Alt+ ']
  global: boolean;
  action: () => void;
};

let commands: KeybindCommand[] = $state([]);

export const keybinds = {
  get commands() {
    return commands;
  },
  register(cmd: KeybindCommand) {
    if (!cmd) return () => {};
    untrack(() => {
      commands.push(cmd);
    });
    return () => {
      commands = commands.filter((c) => c && c.id !== cmd.id);
    };
  }
};
