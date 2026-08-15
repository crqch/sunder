<script lang="ts">
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';

  onMount(() => {
    goto('/auth/sign-in', { replaceState: true });
  });
</script>
