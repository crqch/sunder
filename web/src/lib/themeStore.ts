import { writable } from 'svelte/store';

export type ThemeMode = 'system' | 'light' | 'dark';

const CYCLE_ORDER: ThemeMode[] = ['system', 'light', 'dark'];

function isThemeMode(value: string | null): value is ThemeMode {
  return value === 'system' || value === 'light' || value === 'dark';
}

function createThemeStore() {
  const getInitialTheme = (): ThemeMode => {
    if (typeof window !== 'undefined') {
      try {
        const saved = localStorage.getItem('theme');
        if (isThemeMode(saved)) return saved;
      } catch {
        // Ignore storage errors
      }
      const attr = document.documentElement.getAttribute('data-theme');
      if (attr === 'dark' || attr === 'light') return attr;
    }
    return 'system';
  };

  let currentValue: ThemeMode = getInitialTheme();
  const { subscribe, set } = writable<ThemeMode>(currentValue);

  const applyTheme = (value: ThemeMode) => {
    if (typeof window !== 'undefined') {
      try {
        localStorage.setItem('theme', value);
      } catch {
        // Ignore storage errors
      }
      if (value === 'system') {
        document.documentElement.removeAttribute('data-theme');
      } else {
        document.documentElement.setAttribute('data-theme', value);
      }
    }
  };

  const internalSet = (value: ThemeMode) => {
    currentValue = value;
    applyTheme(value);
    set(value);
  };

  let isTransitioning = false;

  const transitionTo = (next: ThemeMode) => {
    if (next === currentValue || isTransitioning) return;

    if (typeof document === 'undefined' || !document.startViewTransition) {
      internalSet(next);
      return;
    }

    isTransitioning = true;
    document.documentElement.classList.add('theme-transitioning');

    const transition = document.startViewTransition(() => {
      internalSet(next);
    });

    transition.ready
      .then(() => {
        const style = document.createElement('style');
        const animationNameOut = `theme-flip-out-${Date.now()}`;
        const animationNameIn = `theme-flip-in-${Date.now()}`;
        style.innerHTML = `
          @keyframes ${animationNameOut} {
            0% { transform: scale(1); filter: blur(0px) saturate(1); opacity: 1; }
            100% { transform: scale(0.95); filter: blur(20px) saturate(0.5); opacity: 0; }
          }
          @keyframes ${animationNameIn} {
            0% { transform: scale(1.05); filter: blur(40px) saturate(2) brightness(1.2); opacity: 0; }
            100% { transform: scale(1); filter: blur(0px) saturate(1) brightness(1); opacity: 1; }
          }
          ::view-transition-old(root) {
            animation: ${animationNameOut} 400ms cubic-bezier(0.2, 0.8, 0.2, 1) forwards !important;
            mix-blend-mode: normal !important;
          }
          ::view-transition-new(root) {
            animation: ${animationNameIn} 400ms cubic-bezier(0.2, 0.8, 0.2, 1) forwards !important;
            mix-blend-mode: normal !important;
          }
        `;
        document.head.appendChild(style);

        transition.finished
          .then(() => {
            style.remove();
            document.documentElement.classList.remove('theme-transitioning');
            isTransitioning = false;
          })
          .catch(() => {
            style.remove();
            document.documentElement.classList.remove('theme-transitioning');
            isTransitioning = false;
          });
      })
      .catch(() => {
        document.documentElement.classList.remove('theme-transitioning');
        isTransitioning = false;
      });
  };

  return {
    subscribe,
    set: (value: ThemeMode) => transitionTo(value),
    toggle: (_e?: MouseEvent | { clientX: number; clientY: number }) => {
      const next = CYCLE_ORDER[(CYCLE_ORDER.indexOf(currentValue) + 1) % CYCLE_ORDER.length];
      transitionTo(next);
    }
  };
}

export const themeStore = createThemeStore();
