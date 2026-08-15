<script module lang="ts">
  const modalStack: { close: () => void }[] = [];

  if (typeof window !== 'undefined') {
    window.addEventListener('keydown', (e) => {
      if (e.key === 'Escape' && modalStack.length > 0) {
        const active = document.activeElement as HTMLElement;
        const isInput = ['INPUT', 'TEXTAREA', 'SELECT'].includes(active?.tagName);

        if (isInput) {
          active.blur();
          e.preventDefault();
          e.stopPropagation();
        } else {
          const topModal = modalStack[modalStack.length - 1];
          topModal.close();
          e.preventDefault();
          e.stopPropagation();
        }
      }
    });
  }
</script>

<script lang="ts">
  import { fade, fly } from 'svelte/transition';
  import { X } from '@lucide/svelte';

  let {
    open = $bindable(false),
    title,
    children
  } = $props<{
    open: boolean;
    title: string;
    children: import('svelte').Snippet;
  }>();

  let modalRef = {
    close: () => {
      open = false;
    }
  };

  $effect(() => {
    if (open) {
      modalStack.push(modalRef);
      return () => {
        const idx = modalStack.indexOf(modalRef);
        if (idx !== -1) modalStack.splice(idx, 1);
      };
    }
  });

  function portal(node: HTMLElement) {
    document.body.appendChild(node);

    // Focus the first input after a tiny delay to ensure transition starts
    setTimeout(() => {
      const firstInput = node.querySelector('input, textarea, select') as HTMLElement;
      if (firstInput) {
        firstInput.focus();
      }
    }, 10);

    return {
      destroy() {
        if (node.parentNode) {
          node.parentNode.removeChild(node);
        }
      }
    };
  }
</script>

{#if open}
  <div
    use:portal
    class="bg-background/80 fixed inset-0 z-[100] flex items-center justify-center p-4 backdrop-blur-sm sm:p-6"
    transition:fade={{ duration: 150 }}
  >
    <div
      class="bg-card border-border/50 flex max-h-[100dvh] w-full max-w-xl flex-col rounded-xl border shadow-2xl"
      transition:fly={{ y: 20, duration: 250, opacity: 0 }}
    >
      <div class="border-border/50 flex items-center justify-between border-b p-4">
        <h2 class="text-lg font-semibold tracking-tight">{title}</h2>
        <button
          type="button"
          onclick={() => (open = false)}
          class="btn-icon border-transparent p-1.5"
        >
          <X size={18} />
        </button>
      </div>
      <div class="min-h-0 flex-1 overflow-y-auto p-6">
        {@render children()}
      </div>
    </div>
  </div>
{/if}
