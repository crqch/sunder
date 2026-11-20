import { useToast } from '@astryx-sv/core';

/**
 * App-wide toast facade over Astryx's useToast.
 * `initToast()` is called by `<ToastBridge />` inside `<LayerProvider>` so
 * toasts render inside the configured provider viewport rather than mounting
 * a detached fallback container.
 */
type ShowToast = ReturnType<typeof useToast>;

let show: ShowToast | null = null;
let dismissLoading: (() => void) | null = null;

export function initToast() {
  show = useToast();
}

export const toast = {
  success: (body: string) => {
    show?.({ body, type: 'info' });
  },
  info: (body: string) => {
    show?.({ body, type: 'info' });
  },
  warning: (body: string) => {
    show?.({ body, type: 'info' });
  },
  error: (body: string) => {
    show?.({ body, type: 'error' });
  },
  loading: (body: string) => {
    dismissLoading?.();
    dismissLoading =
      show?.({
        body,
        type: 'info',
        isAutoHide: false,
        uniqueID: 'sunder-toast-loading'
      }) ?? null;
  },
  dismiss: () => {
    dismissLoading?.();
    dismissLoading = null;
  }
};
