<script module lang="ts">
  import type { Snippet } from 'svelte';

  export const MODAL_CTX = Symbol('sunder-modal-ctx');

  export interface ModalContext {
    registerFooter: (footer: Snippet | null) => void;
  }
</script>

<script lang="ts">
  import { setContext } from 'svelte';
  import { Dialog, DialogHeader, Layout, LayoutContent, LayoutFooter } from '@astryx-sv/core';

  let {
    open = $bindable(false),
    title,
    subtitle = undefined,
    isDirty = false,
    width = 560,
    footer = undefined,
    children
  }: {
    open: boolean;
    title: string;
    subtitle?: string;
    isDirty?: boolean;
    width?: number | string;
    footer?: Snippet;
    children: Snippet;
  } = $props();

  let registeredFooter = $state<Snippet | null>(null);
  let effectiveFooter = $derived(footer ?? registeredFooter ?? undefined);
  let contentWrapperEl = $state<HTMLElement | null>(null);

  setContext<ModalContext>(MODAL_CTX, {
    registerFooter(next) {
      registeredFooter = next;
    }
  });

  // Astryx Dialog focuses the first [data-autofocus] element in DOM order, which
  // is DialogHeader's <h2>. When a form inside the modal marks a primary field
  // with [data-modal-autofocus], hand focus to that control once the dialog opens.
  $effect(() => {
    if (!open) return;
    let raf1 = 0;
    let raf2 = 0;
    raf1 = requestAnimationFrame(() => {
      raf2 = requestAnimationFrame(() => {
        if (!contentWrapperEl) return;
        const target = contentWrapperEl.querySelector<HTMLElement>(
          '[data-modal-autofocus] input:not([disabled]):not([type="hidden"]), [data-modal-autofocus] textarea:not([disabled]), input[data-modal-autofocus]:not([disabled])'
        );
        if (target && document.activeElement !== target) {
          target.focus({ preventScroll: true });
        }
      });
    });
    return () => {
      cancelAnimationFrame(raf1);
      cancelAnimationFrame(raf2);
    };
  });
</script>

{#snippet dialogHeader()}
  <DialogHeader {title} {subtitle} hasDivider onOpenChange={(next) => (open = next)} />
{/snippet}

{#snippet dialogFooter()}
  <LayoutFooter hasDivider>
    {@render effectiveFooter?.()}
  </LayoutFooter>
{/snippet}

<!--
  Padding contract: the Dialog owns the surface padding and publishes it through
  the container variables; Layout's header/content/footer regions inherit it
  (outer padding on the dialog edges, inner padding between regions). Passing
  padding={0} here would zero those vars, jamming the title and close button
  against the rounded corners — so the regions below deliberately set no
  padding of their own.
-->
<Dialog
  isOpen={open}
  onOpenChange={(next) => (open = next)}
  {width}
  purpose={isDirty ? 'form' : 'info'}
  padding={5}
>
  <Layout header={dialogHeader} footer={effectiveFooter ? dialogFooter : undefined}>
    <LayoutContent>
      <div bind:this={contentWrapperEl} class="sunder-modal-body-slot">
        {@render children()}
      </div>
    </LayoutContent>
  </Layout>
</Dialog>

<style>
  .sunder-modal-body-slot {
    min-width: 0;
  }
</style>
