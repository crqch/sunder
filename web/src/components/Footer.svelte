<script lang="ts">
  import {
    Center,
    Divider,
    Grid,
    GridSpan,
    HStack,
    Icon,
    Link,
    Section,
    Stack,
    Text,
    VStack
  } from '@astryx-sv/core';
  import { SiGithub } from '@icons-pack/svelte-simple-icons';
  import { ArrowUpRight } from '@lucide/svelte';
  import BrandMark from '$components/BrandMark.svelte';
</script>

{#snippet githubIcon()}
  <SiGithub size={14} />
{/snippet}
{#snippet arrowIcon()}
  <Icon icon={ArrowUpRight} size="sm" />
{/snippet}

<Section variant="section" dividers={['top']} padding={0}>
  <Center axis="horizontal" width="100%">
    <VStack gap={8} padding={8} width="100%" maxWidth={1280}>
      <Grid columns={{ minWidth: 220, max: 4 }} gap={8}>
        <GridSpan columns={2}>
          <VStack gap={3}>
            <Link href="/" label="Sunder home">
              <HStack gap={2} vAlign="center">
                <BrandMark size={24} />
                <Text weight="bold" size="lg">Sunder</Text>
              </HStack>
            </Link>
            <Stack maxWidth={380}>
              <Text type="supporting" color="secondary">
                Local-first personal finance and expense tracking. Full ownership of your ledger
                with instant offline responsiveness and multi-device sync.
              </Text>
            </Stack>
          </VStack>
        </GridSpan>

        <VStack gap={2}>
          <Text type="label" weight="semibold">Product</Text>
          <VStack gap={1.5}>
            <Link href="/#features" type="supporting" color="secondary">Features</Link>
            <Link href="/#experience" type="supporting" color="secondary">Platforms</Link>
            <Link href="/auth/sign-in" type="supporting" color="secondary">Sign in</Link>
          </VStack>
        </VStack>

        <VStack gap={2}>
          <Text type="label" weight="semibold">Open Source</Text>
          <VStack gap={1.5}>
            <Link href="/auth/sign-up" type="supporting" color="secondary">
              <HStack gap={1} vAlign="center">Request access {@render arrowIcon()}</HStack>
            </Link>
            <Link
              href="https://github.com/crqch/sunder"
              isExternalLink
              type="supporting"
              color="secondary"
            >
              <HStack gap={1.5} vAlign="center">{@render githubIcon()} GitHub repository</HStack>
            </Link>
          </VStack>
        </VStack>
      </Grid>

      <Divider />

      <HStack hAlign="between" vAlign="center" wrap="wrap" gap={3}>
        <Text type="supporting" color="secondary">
          © {new Date().getFullYear()} Sunder. Open source under MIT.
        </Text>
        <Text type="supporting" color="secondary">Local-first · Self-hostable</Text>
      </HStack>
    </VStack>
  </Center>
</Section>
