export function tooltip(node: HTMLElement, params: { text: string; keys?: string[] }) {
  let tooltipNode: HTMLElement | null = null;

  function renderTooltip() {
    tooltipNode = document.createElement('div');
    tooltipNode.className =
      'fixed z-[200] pointer-events-none bg-background/90 text-foreground border border-foreground/20 px-2 py-1.5 rounded-md text-xs shadow-md flex items-center gap-2 backdrop-blur-sm transition-opacity duration-200 opacity-0';

    let textSpan = document.createElement('span');
    textSpan.textContent = params.text;
    textSpan.className = 'font-medium tracking-wide';
    tooltipNode.appendChild(textSpan);

    if (params.keys && params.keys.length > 0) {
      let keysDiv = document.createElement('div');
      keysDiv.className = 'flex items-center gap-1 ml-2';
      for (let k of params.keys) {
        let kbd = document.createElement('kbd');
        kbd.className =
          'border border-border rounded bg-muted/80 font-mono text-[10px] px-1.5 py-0.5 text-muted-foreground uppercase min-w-[18px] text-center shadow-sm';
        kbd.textContent = k;
        keysDiv.appendChild(kbd);
      }
      tooltipNode.appendChild(keysDiv);
    }
    document.body.appendChild(tooltipNode);
  }

  let showTimer: ReturnType<typeof setTimeout>;

  function handleMouseEnter() {
    showTimer = setTimeout(() => {
      if (!tooltipNode) renderTooltip();
      if (tooltipNode) {
        tooltipNode.style.display = 'flex';
        // calculate position
        const rect = node.getBoundingClientRect();
        let top = rect.top - tooltipNode.offsetHeight - 8;
        let left = rect.left + rect.width / 2 - tooltipNode.offsetWidth / 2;

        if (top < 0) top = rect.bottom + 8;
        if (left < 0) left = 8;
        if (left + tooltipNode.offsetWidth > window.innerWidth) {
          left = window.innerWidth - tooltipNode.offsetWidth - 8;
        }

        tooltipNode.style.top = `${top}px`;
        tooltipNode.style.left = `${left}px`;

        // fade in
        setTimeout(() => {
          if (tooltipNode) tooltipNode.style.opacity = '1';
        }, 10);
      }
    }, 500); // 500ms delay before showing
  }

  function handleMouseLeave() {
    clearTimeout(showTimer);
    if (tooltipNode) {
      tooltipNode.style.opacity = '0';
      setTimeout(() => {
        if (tooltipNode && tooltipNode.style.opacity === '0') {
          tooltipNode.style.display = 'none';
        }
      }, 200);
    }
  }

  node.addEventListener('mouseenter', handleMouseEnter);
  node.addEventListener('mouseleave', handleMouseLeave);
  // hide on click too
  node.addEventListener('click', handleMouseLeave);

  return {
    update(newParams: { text: string; keys?: string[] }) {
      params = newParams;
    },
    destroy() {
      node.removeEventListener('mouseenter', handleMouseEnter);
      node.removeEventListener('mouseleave', handleMouseLeave);
      node.removeEventListener('click', handleMouseLeave);
      if (tooltipNode && tooltipNode.parentNode) {
        tooltipNode.parentNode.removeChild(tooltipNode);
      }
    }
  };
}
