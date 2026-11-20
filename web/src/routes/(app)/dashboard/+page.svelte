<script lang="ts">
  import {
    Button,
    Card,
    Divider,
    EmptyState,
    Grid,
    Heading,
    HStack,
    Icon,
    Layout,
    LayoutContent,
    LayoutHeader,
    Link,
    List,
    ListItem,
    ProgressBar,
    Text,
    Token,
    VStack
  } from '@astryx-sv/core';
  import { ArrowRight, Plus, Wallet } from '@lucide/svelte';
  import { liveQuery } from 'dexie';
  import { db } from '$lib/db';
  import type { Account, AccountEntry, EntryCategory } from '$lib/types';
  import { modals } from '$lib/modals.svelte';
  import { currencyStore, normalizeCurrencyCode } from '$lib/currency.svelte';
  import CategoryDot from '$components/CategoryDot.svelte';
  import ExchangeRatesPanel from '$components/ExchangeRatesPanel.svelte';

  let accounts = $state<Account[]>([]);
  let entries = $state<AccountEntry[]>([]);
  let categories = $state<EntryCategory[]>([]);

  $effect(() => {
    const subAccounts = liveQuery(() =>
      db.accounts.filter((a) => !a.deleted_at).toArray()
    ).subscribe({ next: (v) => (accounts = v) });
    const subEntries = liveQuery(() =>
      db.account_entries
        .filter((e) => !e.deleted_at)
        .reverse()
        .sortBy('date')
    ).subscribe({ next: (v) => (entries = v) });
    const subCategories = liveQuery(() =>
      db.entry_categories.filter((c) => !c.deleted_at).toArray()
    ).subscribe({ next: (v) => (categories = v) });

    return () => {
      subAccounts.unsubscribe();
      subEntries.unsubscribe();
      subCategories.unsubscribe();
    };
  });

  let defaultCurrency = $derived(currencyStore.defaultCurrencyCode);

  let totalBalance = $derived(
    entries.reduce((sum, entry) => sum + currencyStore.convertEntryToDefault(entry), 0)
  );
  let incomeEntries = $derived(entries.filter((e) => e.amount > 0));
  let expenseEntries = $derived(entries.filter((e) => e.amount < 0));

  let income = $derived(
    incomeEntries.reduce((sum, e) => sum + currencyStore.convertEntryToDefault(e), 0)
  );
  let expenses = $derived(
    expenseEntries.reduce((sum, e) => sum + Math.abs(currencyStore.convertEntryToDefault(e)), 0)
  );
  let savingsRate = $derived(income > 0 ? ((income - expenses) / income) * 100 : null);

  let recentEntries = $derived(entries.slice(0, 10));

  let categorySpending = $derived.by(() => {
    let spending: Record<string, number> = {};
    entries.forEach((e) => {
      if (e.amount < 0) {
        const converted = Math.abs(currencyStore.convertEntryToDefault(e));
        spending[e.category_id] = (spending[e.category_id] || 0) + converted;
      }
    });
    return Object.entries(spending)
      .map(([id, amount]) => ({
        id,
        category: categories.find((c) => c.id === id),
        amount,
        share: expenses > 0 ? (amount / expenses) * 100 : 0
      }))
      .sort((a, b) => b.amount - a.amount)
      .slice(0, 5);
  });

  let accountSummaries = $derived.by(() => {
    return accounts
      .map((account) => {
        const displayCurrency = account.currency_code
          ? normalizeCurrencyCode(account.currency_code)
          : defaultCurrency;
        const accountEntries = entries.filter((e) => e.account_id === account.id);
        const balance = accountEntries.reduce(
          (sum, e) =>
            sum +
            currencyStore.tryConvert(
              e.amount,
              e.currency_code,
              displayCurrency,
              e.date || e.created_at
            ).amount,
          0
        );
        return {
          account,
          displayCurrency,
          balance,
          count: accountEntries.length
        };
      })
      .slice(0, 6);
  });

  function getCategoryColor(id: string) {
    return categories.find((c) => c.id === id)?.color || '#71717a';
  }
  function getCategoryName(id: string) {
    return categories.find((c) => c.id === id)?.title || 'Unknown';
  }
  function getAccountName(id: string) {
    return accounts.find((a) => a.id === id)?.name || 'Unknown';
  }
</script>

{#snippet addIcon()}
  <Icon icon={Plus} size="sm" />
{/snippet}
{#snippet walletIcon()}
  <Icon icon={Wallet} size="sm" />
{/snippet}
{#snippet viewAllIcon()}
  <Icon icon={ArrowRight} size="sm" />
{/snippet}
{#snippet emptyEntryAction()}
  <Button
    label="Add entry"
    variant="primary"
    icon={addIcon}
    onclick={() => (modals.createEntry = true)}
  />
{/snippet}
{#snippet emptyAccountAction()}
  <Button
    label="Add account"
    variant="secondary"
    icon={walletIcon}
    onclick={() => (modals.createAccount = true)}
  />
{/snippet}

{#snippet header()}
  <LayoutHeader hasDivider padding={6}>
    <HStack hAlign="between" vAlign="center" wrap="wrap" gap={4}>
      <VStack gap={1}>
        <HStack gap={2} vAlign="center" wrap="wrap">
          <Heading level={1}>Dashboard</Heading>
          <Token
            label="Totals in {defaultCurrency}"
            size="sm"
            href="/settings"
            description="Change default display currency in Settings"
          />
        </HStack>
        <Text type="supporting" color="secondary">
          {accounts.length}
          {accounts.length === 1 ? 'account' : 'accounts'} · {entries.length}
          {entries.length === 1 ? 'entry' : 'entries'}
        </Text>
      </VStack>

      <HStack gap={2} wrap="wrap">
        <Button
          label="Add account"
          variant="secondary"
          icon={walletIcon}
          tooltip="New account (Alt+N, A)"
          onclick={() => (modals.createAccount = true)}
        />
        <Button
          label="Add entry"
          variant="primary"
          icon={addIcon}
          tooltip="New entry (Alt+Space)"
          onclick={() => (modals.createEntry = true)}
        />
      </HStack>
    </HStack>
  </LayoutHeader>
{/snippet}

{#snippet content()}
  <LayoutContent padding={6} label="Dashboard">
    <VStack gap={6}>
      <Grid columns={{ minWidth: 210, max: 4 }} gap={4}>
        <Card padding={5}>
          <VStack gap={1}>
            <Text type="supporting" color="secondary">Net balance</Text>
            <Heading
              level={2}
              class={totalBalance >= 0
                ? 'sunder-tabular sunder-positive'
                : 'sunder-tabular sunder-negative'}
            >
              {currencyStore.formatAmount(totalBalance, defaultCurrency)}
            </Heading>
            <Text type="supporting" color="secondary">
              Across {accounts.length}
              {accounts.length === 1 ? 'account' : 'accounts'}
            </Text>
          </VStack>
        </Card>

        <Card padding={5}>
          <VStack gap={1}>
            <Text type="supporting" color="secondary">Total income</Text>
            <Heading level={2} class="sunder-tabular sunder-positive">
              {currencyStore.formatAmount(income, defaultCurrency, { showPlus: true })}
            </Heading>
            <Text type="supporting" color="secondary">
              {incomeEntries.length}
              {incomeEntries.length === 1 ? 'entry' : 'entries'}
            </Text>
          </VStack>
        </Card>

        <Card padding={5}>
          <VStack gap={1}>
            <Text type="supporting" color="secondary">Total expenses</Text>
            <Heading level={2} class="sunder-tabular sunder-negative">
              {currencyStore.formatAmount(-expenses, defaultCurrency)}
            </Heading>
            <Text type="supporting" color="secondary">
              {expenseEntries.length}
              {expenseEntries.length === 1 ? 'entry' : 'entries'}
            </Text>
          </VStack>
        </Card>

        <Card padding={5}>
          <VStack gap={1}>
            <Text type="supporting" color="secondary">Savings rate</Text>
            <Heading
              level={2}
              class={savingsRate === null
                ? 'sunder-tabular'
                : savingsRate >= 0
                  ? 'sunder-tabular sunder-positive'
                  : 'sunder-tabular sunder-negative'}
            >
              {savingsRate === null ? '—' : `${savingsRate.toFixed(1)}%`}
            </Heading>
            <Text type="supporting" color="secondary">
              {savingsRate === null
                ? 'Log income to calculate'
                : savingsRate >= 0
                  ? 'Net positive cash flow'
                  : 'Spending exceeds income'}
            </Text>
          </VStack>
        </Card>
      </Grid>

      <ExchangeRatesPanel />

      <Grid columns={{ minWidth: 340, max: 2 }} gap={6}>
        <Card padding={0}>
          <VStack gap={0}>
            <HStack paddingInline={5} paddingBlock={4} hAlign="between" vAlign="center" gap={3}>
              <Heading level={3}>Recent transactions</Heading>
              <Link href="/entries" type="supporting">
                <HStack gap={1} vAlign="center">View all {@render viewAllIcon()}</HStack>
              </Link>
            </HStack>
            <Divider />
            {#if recentEntries.length === 0}
              <EmptyState
                isCompact
                title="No transactions yet"
                description="Start by adding your first entry."
                actions={emptyEntryAction}
              />
            {:else}
              <List hasDividers edge="bleed">
                {#each recentEntries as entry (entry.id)}
                  {@const entryCode = normalizeCurrencyCode(entry.currency_code)}
                  {@const isForeign = entryCode !== defaultCurrency}
                  {@const converted = isForeign
                    ? currencyStore.tryConvert(
                        entry.amount,
                        entryCode,
                        defaultCurrency,
                        entry.date || entry.created_at
                      )
                    : null}
                  {#snippet entryStart()}
                    <CategoryDot color={getCategoryColor(entry.category_id)} />
                  {/snippet}
                  {#snippet entryEnd()}
                    <VStack gap={0} hAlign="end">
                      <Text
                        weight="semibold"
                        hasTabularNumbers
                        class={entry.amount >= 0 ? 'sunder-positive' : 'sunder-negative'}
                      >
                        {currencyStore.formatAmount(entry.amount, entryCode, { showPlus: true })}
                      </Text>
                      {#if converted?.converted}
                        <Text type="supporting" color="secondary" hasTabularNumbers>
                          ≈ {currencyStore.formatAmount(converted.amount, defaultCurrency, {
                            showPlus: true
                          })}
                        </Text>
                      {/if}
                    </VStack>
                  {/snippet}
                  <ListItem
                    label={entry.title}
                    description="{getCategoryName(entry.category_id)} · {getAccountName(
                      entry.account_id
                    )} · {new Date(entry.date || entry.created_at).toLocaleDateString()}"
                    href="/entries/{entry.id}"
                    startContent={entryStart}
                    endContent={entryEnd}
                  />
                {/each}
              </List>
            {/if}
          </VStack>
        </Card>

        <VStack gap={6}>
          <Card padding={5}>
            <VStack gap={4}>
              <HStack hAlign="between" vAlign="center" gap={3}>
                <Heading level={3}>Top spending categories</Heading>
                <Link href="/categories" type="supporting">
                  <HStack gap={1} vAlign="center">All categories {@render viewAllIcon()}</HStack>
                </Link>
              </HStack>

              {#if categorySpending.length === 0}
                <Text type="supporting" color="secondary">
                  Expense breakdowns appear here once you log spending.
                </Text>
              {:else}
                <VStack gap={4}>
                  {#each categorySpending as item (item.id)}
                    <VStack gap={1.5}>
                      <HStack hAlign="between" vAlign="center" gap={2}>
                        <HStack gap={2} vAlign="center">
                          <CategoryDot color={item.category?.color || '#71717a'} size="sm" />
                          {#if item.category}
                            <Link href="/categories/{item.category.id}" type="inherit">
                              <Text weight="medium" maxLines={1}>{item.category.title}</Text>
                            </Link>
                          {:else}
                            <Text weight="medium" maxLines={1}>Unknown</Text>
                          {/if}
                        </HStack>
                        <HStack gap={2} vAlign="center">
                          <Text type="supporting" color="secondary" hasTabularNumbers>
                            {item.share.toFixed(0)}%
                          </Text>
                          <Text weight="medium" hasTabularNumbers>
                            {currencyStore.formatAmount(item.amount, defaultCurrency, {
                              absolute: true
                            })}
                          </Text>
                        </HStack>
                      </HStack>
                      <ProgressBar
                        label="{item.category?.title || 'Unknown'} share of spending"
                        isLabelHidden
                        value={item.share}
                      />
                    </VStack>
                  {/each}
                </VStack>
              {/if}
            </VStack>
          </Card>

          <Card padding={0}>
            <VStack gap={0}>
              <HStack paddingInline={5} paddingBlock={4} hAlign="between" vAlign="center" gap={3}>
                <Heading level={3}>Accounts</Heading>
                <Link href="/accounts" type="supporting">
                  <HStack gap={1} vAlign="center">Manage {@render viewAllIcon()}</HStack>
                </Link>
              </HStack>
              <Divider />
              {#if accountSummaries.length === 0}
                <EmptyState
                  isCompact
                  title="No accounts yet"
                  description="Create an account to organize your balances."
                  actions={emptyAccountAction}
                />
              {:else}
                <List hasDividers edge="bleed">
                  {#each accountSummaries as item (item.account.id)}
                    {#snippet accountEnd()}
                      <HStack gap={2} vAlign="center">
                        <Token label={item.displayCurrency} size="sm" />
                        <Text
                          weight="semibold"
                          hasTabularNumbers
                          class={item.balance >= 0 ? 'sunder-positive' : 'sunder-negative'}
                        >
                          {currencyStore.formatAmount(item.balance, item.displayCurrency)}
                        </Text>
                      </HStack>
                    {/snippet}
                    <ListItem
                      label={item.account.name}
                      description="{item.count} {item.count === 1 ? 'entry' : 'entries'}"
                      href="/accounts/{item.account.id}"
                      endContent={accountEnd}
                    />
                  {/each}
                </List>
              {/if}
            </VStack>
          </Card>
        </VStack>
      </Grid>
    </VStack>
  </LayoutContent>
{/snippet}

<Layout height="fill" contentWidth={1280} {header} {content} />
