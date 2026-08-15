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

  const internalSet = (value: Theme) => {
    applyTheme(value);
    set(value);
  };

  let isTransitioning = false;

  return {
    subscribe,
    set: internalSet,
    toggle: (e?: MouseEvent | { clientX: number; clientY: number }) => {
      if (isTransitioning) return;

      let next: Theme;
      update((current) => {
        next = current === 'light' ? 'dark' : 'light';
        return current; // Don't change state yet
      });

      if (!document.startViewTransition) {
        internalSet(next!);
        return;
      }

      isTransitioning = true;

      document.documentElement.classList.add('theme-transitioning');

      const transition = document.startViewTransition(() => {
        internalSet(next!);
      });

      transition.ready.then(() => {
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
      });
    }
  };
}

export const themeStore = createThemeStore();
