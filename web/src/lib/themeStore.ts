import { writable } from 'svelte/store';

type Theme = 'light' | 'dark';

function createThemeStore() {
  const getInitialTheme = (): Theme => {
    if (typeof window !== 'undefined') {
      const attr = document.documentElement.getAttribute('data-theme');
      if (attr === 'dark' || attr === 'light') return attr;
    }
    return 'light';
  };

  const { subscribe, set, update } = writable<Theme>(getInitialTheme());

  const applyTheme = (value: Theme) => {
    if (typeof window !== 'undefined') {
      localStorage.setItem('theme', value);
      document.documentElement.setAttribute('data-theme', value);
    }
  };

  return {
    subscribe,
    set: (value: Theme) => {
      applyTheme(value);
      set(value);
    },
    toggle: () => {
      update((current) => {
        const next = current === 'light' ? 'dark' : 'light';
        applyTheme(next);
        return next;
      });
    }
  };
}

export const themeStore = createThemeStore();
