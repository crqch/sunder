<script lang="ts">
  import { Stack } from '@astryx-sv/core';

  let { color, size = 'md' }: { color: string; size?: 'sm' | 'md' | 'lg' } = $props();

  const sizeVar = {
    sm: 'var(--spacing-2)',
    md: 'var(--spacing-3)',
    lg: 'var(--spacing-5)'
  };
</script>

<!-- User-defined category color: data, not theme — carried via CSS variables. -->
<Stack class="sunder-category-dot" style="--dot-color: {color}; --dot-size: {sizeVar[size]}" />
