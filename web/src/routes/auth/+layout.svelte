<script lang="ts">
  import {
    Button,
    Center,
    Heading,
    HStack,
    Icon,
    Layout,
    LayoutContent,
    LayoutHeader,
    Stack,
    VStack
  } from '@astryx-sv/core';
  import { ArrowLeft } from '@lucide/svelte';
  import BrandMark from '$components/BrandMark.svelte';
  import ThemeButton from '$components/ThemeButton.svelte';
  import { authStore } from '$lib/auth';
  import { goto } from '$app/navigation';

  let { children } = $props();

  $effect(() => {
    if ($authStore.isAuthenticated) {
      goto('/dashboard');
    }
  });
</script>

{#snippet backIcon()}
  <Icon icon={ArrowLeft} size="sm" />
{/snippet}

{#snippet header()}
  <LayoutHeader padding={4}>
    <HStack hAlign="between" vAlign="center">
      <Button label="Back to home" variant="ghost" size="sm" href="/" icon={backIcon} />
      <ThemeButton size="sm" variant="ghost" />
    </HStack>
  </LayoutHeader>
{/snippet}

<Stack minHeight="100dvh">
  <Layout height="fill" {header}>
    <LayoutContent padding={6}>
      <Center axis="both" minHeight="100%">
        <VStack gap={6} hAlign="center" width="100%" maxWidth={420}>
          <VStack gap={2} hAlign="center">
            <BrandMark size={40} />
            <Heading level={1}>Sunder</Heading>
          </VStack>

          <Stack width="100%">
            {@render children()}
          </Stack>
        </VStack>
      </Center>
    </LayoutContent>
  </Layout>
</Stack>
