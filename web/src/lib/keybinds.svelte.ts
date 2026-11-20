import { untrack } from 'svelte';

export type KeybindCommand = {
  id: string;
  name: string;
  keys: string[]; // e.g. ['Alt+n', 'a'] or ['Alt+ ']
  global: boolean;
  action: () => void;
};

let commands: KeybindCommand[] = $state([]);
let helpOpen = $state(false);

export const keybinds = {
  get commands() {
    return commands;
  },
  get helpOpen() {
    return helpOpen;
  },
  set helpOpen(value: boolean) {
    helpOpen = value;
  },
  toggleHelp() {
    helpOpen = !helpOpen;
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
