<script lang="ts">
  import { initToast } from '$lib/toast.svelte';

  // Must mount inside <LayerProvider> so useToast() binds to the provider context.
  initToast();
</script>
