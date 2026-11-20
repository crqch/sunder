<script lang="ts">
  import {
    Button,
    Card,
    Center,
    Divider,
    Grid,
    Heading,
    HStack,
    Icon,
    Layout,
    LayoutContent,
    List,
    ListItem,
    Section,
    Stack,
    StatusDot,
    Text,
    Token,
    VStack
  } from '@astryx-sv/core';
  import {
    ArrowRight,
    Globe,
    Key,
    Keyboard,
    Laptop,
    Layers,
    Lock,
    RefreshCw,
    Smartphone,
    TrendingUp,
    Zap
  } from '@lucide/svelte';
  import CategoryDot from '$components/CategoryDot.svelte';
  import { authStore } from '$lib/auth';

  const mockEntries = [
    {
      id: 1,
      title: 'Weekly Grocery Shopping',
      category: 'Groceries',
      color: '#22c55e',
      account: 'Checking Account',
      amount: -142.8,
      date: 'Today, 14:20',
      location: 'Whole Foods Market'
    },
    {
      id: 2,
      title: 'Monthly Salary Deposit',
      category: 'Income',
      color: '#3b82f6',
      account: 'Main Savings',
      amount: 4250.0,
      date: 'Yesterday, 09:00',
      location: 'Direct Deposit'
    },
    {
      id: 3,
      title: 'Cloud Storage & Hosting',
      category: 'Subscriptions',
      color: '#8b5cf6',
      account: 'Credit Card',
      amount: -34.99,
      date: 'Aug 12, 18:45',
      location: 'Digital Workspace'
    }
  ];

  const capabilities = [
    {
      icon: Zap,
      title: 'Instant local responsiveness',
      body: 'Every transaction writes directly to IndexedDB on your device. Search, filter, and calculate balances in milliseconds with zero network spinners.'
    },
    {
      icon: RefreshCw,
      title: 'Background conflict-safe sync',
      body: 'Log purchases offline on a flight or subway and reconcile automatically across devices when you reconnect—or trigger manual sync anytime.'
    },
    {
      icon: Globe,
      title: 'Native multi-currency accounting',
      body: 'Track accounts and individual entries in any currency with historical exchange-rate conversion rolled up cleanly to your home currency.'
    },
    {
      icon: Layers,
      title: 'Structured accounts & categories',
      body: 'Separate checking, savings, cash, and credit cards alongside color-coded categories, custom notes, and location tags.'
    },
    {
      icon: Keyboard,
      title: 'Keyboard-first workflow',
      body: 'Jump between views and log expenses in seconds with chorded shortcuts (Alt+Space to log, Alt+1–5 to switch views, Alt+? for reference).'
    },
    {
      icon: Lock,
      title: 'Private, invite-only & self-hostable',
      body: 'Open-source architecture with invite-only access and full JSON backup/restore. Your ledger belongs strictly to you.'
    }
  ];

  const platforms = [
    {
      icon: Laptop,
      title: 'Desktop & web workspace',
      body: 'Full ledger management, multi-facet filtering, bulk edits, analytics, and keyboard navigation built for speed.',
      tag: 'Web app'
    },
    {
      icon: Smartphone,
      title: 'Native Android companion',
      body: 'Lightweight mobile experience built for rapid on-the-go expense capture with local storage and seamless sync.',
      tag: 'Android app'
    },
    {
      icon: RefreshCw,
      title: 'Self-hosted sync server',
      body: 'Encrypted synchronization backed by your own server so your history stays safe across device upgrades.',
      tag: 'Sync engine'
    }
  ];

  const specs = [
    { value: '< 1 ms', label: 'Local read & write speed' },
    { value: '100%', label: 'Offline-first ledger' },
    { value: 'Multi-FX', label: 'Automatic rate conversion' },
    { value: 'MIT', label: 'Open source & private' }
  ];
</script>

{#snippet arrowIcon()}
  <Icon icon={ArrowRight} size="sm" />
{/snippet}
{#snippet keyIcon()}
  <Icon icon={Key} size="sm" />
{/snippet}
{#snippet trendIcon()}
  <Icon icon={TrendingUp} size="sm" color="success" />
{/snippet}

{#snippet content()}
  <LayoutContent padding={6} label="Sunder overview">
    <Center axis="horizontal" width="100%">
      <VStack gap={10} width="100%" maxWidth={1120} paddingBlock={8}>
        <!-- Hero -->
        <VStack gap={6} hAlign="center">
          <HStack gap={2} vAlign="center">
            <StatusDot variant="success" label="Local-first engine" />
            <Text type="supporting" color="secondary">
              Local-first personal finance · Instant offline access · Multi-device sync
            </Text>
          </HStack>

          <VStack gap={4} hAlign="center" maxWidth={780}>
            <Heading level={1} type="display-1" justify="center" textWrap="balance">
              Master your ledger at the speed of thought
            </Heading>
            <Text type="large" color="secondary" justify="center" textWrap="pretty">
              Sunder gives you instant financial clarity with zero loading delays. Track accounts
              across currencies, log expenses offline, and sync effortlessly across all your
              devices.
            </Text>
          </VStack>

          <HStack gap={3} wrap="wrap" hAlign="center">
            {#if $authStore.isAuthenticated}
              <Button
                label="Open dashboard"
                variant="primary"
                size="lg"
                href="/dashboard"
                endContent={arrowIcon}
              />
            {:else}
              <Button
                label="Sign in"
                variant="primary"
                size="lg"
                href="/auth/sign-in"
                endContent={arrowIcon}
              />
              <Button
                label="Redeem invite code"
                variant="secondary"
                size="lg"
                href="/auth/sign-up"
                icon={keyIcon}
              />
            {/if}
          </HStack>

          <!-- Product Preview (single container, divider-separated zones, no nested cards) -->
          <Card width="100%" maxWidth={960} padding={0} elevation="med">
            <VStack gap={0}>
              <HStack
                paddingInline={6}
                paddingBlock={3}
                hAlign="between"
                vAlign="center"
                wrap="wrap"
                gap={3}
              >
                <HStack gap={2} vAlign="center">
                  <Text type="code" color="secondary" size="sm">sunder // ledger</Text>
                  <Token label="USD" size="sm" />
                </HStack>
                <HStack gap={2} vAlign="center">
                  <StatusDot variant="success" label="Synced" />
                  <Text type="supporting" color="secondary">Local vault synced</Text>
                </HStack>
              </HStack>

              <Divider />

              <Stack padding={6}>
                <Grid columns={{ minWidth: 200, max: 3 }} gap={6}>
                  <VStack gap={1}>
                    <Text type="supporting" color="secondary">Net balance</Text>
                    <Heading level={3} class="sunder-tabular">$21,470.25</Heading>
                    <HStack gap={1} vAlign="center">
                      {@render trendIcon()}
                      <Text type="supporting" class="sunder-positive" hasTabularNumbers>
                        +$3,807.20 this month
                      </Text>
                    </HStack>
                  </VStack>

                  <VStack gap={1}>
                    <Text type="supporting" color="secondary">Monthly income</Text>
                    <Heading level={3} class="sunder-tabular sunder-positive">+$4,250.00</Heading>
                    <Text type="supporting" color="secondary">3 active accounts</Text>
                  </VStack>

                  <VStack gap={1}>
                    <Text type="supporting" color="secondary">Monthly expenses</Text>
                    <Heading level={3} class="sunder-tabular sunder-negative">-$442.80</Heading>
                    <Text type="supporting" color="secondary">89.6% savings rate</Text>
                  </VStack>
                </Grid>
              </Stack>

              <Divider />

              <VStack gap={2} paddingInline={6} paddingBlock={4}>
                <HStack hAlign="between" vAlign="center">
                  <Text type="label" weight="semibold" color="secondary">Recent transactions</Text>
                  <Text type="supporting" color="secondary">3 entries</Text>
                </HStack>
                <List hasDividers edge="flush">
                  {#each mockEntries as entry (entry.id)}
                    {#snippet entryDot()}
                      <CategoryDot color={entry.color} size="md" />
                    {/snippet}
                    {#snippet entryEnd()}
                      <HStack gap={3} vAlign="center">
                        <Token label={entry.category} size="sm" />
                        <Text
                          weight="semibold"
                          hasTabularNumbers
                          class={entry.amount > 0 ? 'sunder-positive' : 'sunder-negative'}
                        >
                          {entry.amount > 0 ? '+' : '-'}${Math.abs(entry.amount).toLocaleString(
                            'en-US',
                            { minimumFractionDigits: 2, maximumFractionDigits: 2 }
                          )}
                        </Text>
                      </HStack>
                    {/snippet}
                    <ListItem
                      label={entry.title}
                      description="{entry.account} · {entry.location} · {entry.date}"
                      startContent={entryDot}
                      endContent={entryEnd}
                    />
                  {/each}
                </List>
              </VStack>
            </VStack>
          </Card>
        </VStack>

        <!-- Key Specs -->
        <Section variant="transparent" dividers={['top', 'bottom']} padding={6}>
          <Grid columns={{ minWidth: 160, max: 4 }} gap={6}>
            {#each specs as spec (spec.value)}
              <VStack gap={1} hAlign="center">
                <Heading level={2} justify="center">{spec.value}</Heading>
                <Text type="supporting" color="secondary" justify="center">{spec.label}</Text>
              </VStack>
            {/each}
          </Grid>
        </Section>

        <!-- Capabilities (editorial typographic grid, no repetitive card chrome) -->
        <VStack gap={8} id="features">
          <VStack gap={2} maxWidth={600}>
            <Text type="label" color="secondary" weight="semibold">Capabilities</Text>
            <Heading level={2} type="display-3" textWrap="balance">
              Built for effortless financial clarity
            </Heading>
            <Text color="secondary">
              Traditional finance apps make you wait on slow servers. Sunder puts local speed,
              multi-currency accuracy, and privacy first.
            </Text>
          </VStack>

          <Grid columns={{ minWidth: 300, max: 3 }} gap={8}>
            {#each capabilities as item (item.title)}
              <VStack gap={3}>
                <Divider />
                <HStack gap={2} vAlign="center" paddingBlockStart={2}>
                  <Icon icon={item.icon} size="md" color="accent" />
                  <Heading level={4}>{item.title}</Heading>
                </HStack>
                <Text type="supporting" color="secondary">{item.body}</Text>
              </VStack>
            {/each}
          </Grid>
        </VStack>

        <!-- Platforms -->
        <VStack gap={6} id="experience">
          <VStack gap={2} maxWidth={600}>
            <Text type="label" color="secondary" weight="semibold">Ecosystem</Text>
            <Heading level={2} type="display-3">Designed for all your devices</Heading>
            <Text color="secondary">
              Access and update your ledger seamlessly whether you are at your desk or on the move.
            </Text>
          </VStack>

          <Grid columns={{ minWidth: 260, max: 3 }} gap={4}>
            {#each platforms as platform (platform.title)}
              <Card padding={6}>
                <VStack gap={3}>
                  <HStack hAlign="between" vAlign="center" gap={2}>
                    <Icon icon={platform.icon} size="md" color="accent" />
                    <Token label={platform.tag} size="sm" />
                  </HStack>
                  <Heading level={4}>{platform.title}</Heading>
                  <Text type="supporting" color="secondary">{platform.body}</Text>
                </VStack>
              </Card>
            {/each}
          </Grid>
        </VStack>

        <!-- Closing CTA -->
        <Section variant="muted" padding={8}>
          <HStack hAlign="between" vAlign="center" wrap="wrap" gap={6}>
            <VStack gap={1} maxWidth={560}>
              <Heading level={3}>Ready to take control of your ledger?</Heading>
              <Text color="secondary">
                Sign in to your workspace or redeem an invite code to start tracking offline.
              </Text>
            </VStack>
            <HStack gap={3} wrap="wrap">
              {#if $authStore.isAuthenticated}
                <Button
                  label="Open dashboard"
                  variant="primary"
                  href="/dashboard"
                  endContent={arrowIcon}
                />
              {:else}
                <Button
                  label="Sign in"
                  variant="primary"
                  href="/auth/sign-in"
                  endContent={arrowIcon}
                />
                <Button label="Sign up" variant="secondary" href="/auth/sign-up" />
              {/if}
            </HStack>
          </HStack>
        </Section>
      </VStack>
    </Center>
  </LayoutContent>
{/snippet}

<Layout height="auto" {content} />
