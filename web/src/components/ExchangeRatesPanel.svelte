<script lang="ts">
  import {
    Badge,
    Button,
    Card,
    Divider,
    Grid,
    Heading,
    HStack,
    Icon,
    IconButton,
    List,
    ListItem,
    SegmentedControl,
    SegmentedControlItem,
    Skeleton,
    Stack,
    Text,
    TextInput,
    Token,
    VStack
  } from '@astryx-sv/core';
  import { ArrowLeftRight, ArrowRight, RefreshCw } from '@lucide/svelte';
  import CurrencySelect from '$components/CurrencySelect.svelte';
  import ExchangeRateChart, {
    COMPARE_PALETTE,
    formatFxRate,
    type CompareSeries
  } from '$components/ExchangeRateChart.svelte';
  import { currencyStore, normalizeCurrencyCode, toDateKey } from '$lib/currency.svelte';

  const POPULAR_CODES = ['USD', 'EUR', 'GBP', 'JPY', 'PLN', 'CHF', 'CAD', 'AUD'];
  const RANGES = [
    { value: '7D', days: 7 },
    { value: '1M', days: 30 },
    { value: '3M', days: 91 },
    { value: '6M', days: 182 },
    { value: '1Y', days: 365 }
  ];

  let baseCode = $state('USD');
  let targetCode = $state(
    currencyStore.defaultCurrencyCode !== 'USD' ? currencyStore.defaultCurrencyCode : 'EUR'
  );
  let targetManuallyChosen = $state(false);
  let range = $state('1M');
  let compareMode = $state(false);
  let compareCodes = $state<string[]>([]);
  let amountStr = $state('1');
  let loading = $state(false);
  let loadError = $state<string | null>(null);

  // Sync target currency with user preferences once async store loads, unless manually changed.
  $effect(() => {
    const def = currencyStore.defaultCurrencyCode;
    if (!targetManuallyChosen && def && def !== baseCode && targetCode !== def) {
      targetCode = def;
    }
  });

  let rangeDays = $derived(RANGES.find((r) => r.value === range)?.days ?? 30);
  let rangeTo = $derived(toDateKey(new Date()));
  let rangeFrom = $derived.by(() => {
    const d = new Date(`${rangeTo}T00:00:00Z`);
    d.setUTCDate(d.getUTCDate() - (rangeDays - 1));
    return d.toISOString().slice(0, 10);
  });

  // Include POPULAR_CODES so the Popular Rates rows immediately display real % change deltas.
  let neededSymbols = $derived.by(() => {
    const codes = [baseCode, targetCode, ...POPULAR_CODES, ...(compareMode ? compareCodes : [])];
    return Array.from(new Set(codes.map((c) => normalizeCurrencyCode(c)))).filter(
      (c) => c !== 'USD'
    );
  });

  let loadToken = 0;
  $effect(() => {
    const from = rangeFrom;
    const to = rangeTo;
    const symbols = neededSymbols;
    const token = ++loadToken;
    loading = true;
    loadError = null;
    currencyStore
      .ensureHistoryForRange(from, to, symbols)
      .catch((err) => {
        console.error('Failed to load exchange rate history', err);
        if (token === loadToken) loadError = 'Could not load exchange rates. Try refreshing.';
      })
      .finally(() => {
        if (token === loadToken) loading = false;
      });
  });

  async function refreshRates() {
    const token = ++loadToken;
    loading = true;
    loadError = null;
    try {
      await currencyStore.refreshLatestRates();
      await currencyStore.ensureHistoryForRange(rangeFrom, rangeTo, neededSymbols, true);
    } catch (err) {
      console.error('Failed to refresh exchange rates', err);
      if (token === loadToken) loadError = 'Could not refresh exchange rates.';
    } finally {
      if (token === loadToken) loading = false;
    }
  }

  let series = $derived(currencyStore.getRateSeries(baseCode, targetCode, rangeFrom, rangeTo));

  let compareSeries = $derived.by<CompareSeries[]>(() => {
    if (!compareMode) return [];
    const codes = Array.from(new Set([targetCode, ...compareCodes])).filter((c) => c !== baseCode);
    return codes.map((code) => ({
      code,
      points: currencyStore.getRateSeries(baseCode, code, rangeFrom, rangeTo)
    }));
  });

  let isCompareActive = $derived(compareMode && compareSeries.length >= 2);

  let latestRate = $derived(series.length > 0 ? series[series.length - 1].rate : null);
  let firstRate = $derived(series.length > 0 ? series[0].rate : null);
  let periodPct = $derived(
    latestRate !== null && firstRate !== null && firstRate > 0
      ? ((latestRate - firstRate) / firstRate) * 100
      : null
  );
  let periodAbs = $derived(
    latestRate !== null && firstRate !== null ? latestRate - firstRate : null
  );
  let periodHigh = $derived.by(() =>
    series.length > 0 ? Math.max(...series.map((p) => p.rate)) : null
  );
  let periodLow = $derived.by(() =>
    series.length > 0 ? Math.min(...series.map((p) => p.rate)) : null
  );
  let periodAvg = $derived.by(() =>
    series.length > 0 ? series.reduce((sum, p) => sum + p.rate, 0) / series.length : null
  );

  let amountNum = $derived.by(() => {
    const n = parseFloat(amountStr.replace(',', '.'));
    return Number.isFinite(n) ? n : null;
  });
  let convertedAmount = $derived(
    amountNum !== null && latestRate !== null ? amountNum * latestRate : null
  );
  let inverseRate = $derived(latestRate !== null && latestRate > 0 ? 1 / latestRate : null);

  let visiblePopularCodes = $derived(POPULAR_CODES.filter((code) => code !== baseCode));

  let popularRows = $derived.by(() =>
    visiblePopularCodes.map((code) => {
      const pts = currencyStore.getRateSeries(baseCode, code, rangeFrom, rangeTo);
      const rate =
        pts.length > 0 ? pts[pts.length - 1].rate : currencyStore.getRate(baseCode, code);
      const pct =
        pts.length >= 2 && pts[0].rate > 0
          ? ((pts[pts.length - 1].rate - pts[0].rate) / pts[0].rate) * 100
          : null;
      return { code, rate, pct };
    })
  );

  function fmtRate(value: number | null): string {
    if (value === null) return '—';
    return formatFxRate(value);
  }

  function fmtPct(pct: number | null): string {
    if (pct === null) return '—';
    return `${pct > 0 ? '+' : ''}${pct.toFixed(2)}%`;
  }

  function pctBadgeVariant(pct: number | null): 'success' | 'error' | 'neutral' {
    if (pct === null) return 'neutral';
    if (pct > 0.005) return 'success';
    if (pct < -0.005) return 'error';
    return 'neutral';
  }

  function pctTextClass(pct: number | null): string {
    if (pct === null) return 'sunder-tabular';
    if (pct > 0.005) return 'sunder-tabular sunder-positive';
    if (pct < -0.005) return 'sunder-tabular sunder-negative';
    return 'sunder-tabular';
  }

  function handleBaseChange(code: string) {
    const next = normalizeCurrencyCode(code);
    if (!code || next === baseCode) return;
    if (next === targetCode) {
      targetCode = baseCode;
      targetManuallyChosen = true;
    }
    baseCode = next;
    compareCodes = compareCodes.filter((c) => c !== next);
  }

  function handleTargetChange(code: string) {
    const next = normalizeCurrencyCode(code);
    if (!code || next === targetCode) return;
    if (next === baseCode) baseCode = targetCode;
    targetCode = next;
    targetManuallyChosen = true;
    compareCodes = compareCodes.filter((c) => c !== next);
  }

  function swapPair() {
    const prevBase = baseCode;
    baseCode = targetCode;
    targetCode = prevBase;
    targetManuallyChosen = true;
    compareCodes = compareCodes.filter((c) => c !== baseCode && c !== targetCode);
  }

  function toggleCompareMode() {
    compareMode = !compareMode;
    if (
      compareMode &&
      compareCodes.filter((c) => c !== baseCode && c !== targetCode).length === 0
    ) {
      compareCodes = visiblePopularCodes.filter((c) => c !== targetCode).slice(0, 2);
    }
  }

  function handleCurrencySelectOrCompare(code: string) {
    if (!compareMode) {
      handleTargetChange(code);
      return;
    }
    if (code === targetCode) return;
    const next = compareCodes.includes(code)
      ? compareCodes.filter((c) => c !== code)
      : [...compareCodes, code];
    compareCodes = next;
    if (next.length === 0) {
      compareMode = false;
    }
  }

  function isChipActive(code: string): boolean {
    if (code === targetCode) return true;
    return compareMode && compareCodes.includes(code);
  }

  let showChartSkeleton = $derived(loading && series.length === 0);
</script>

{#snippet refreshIcon()}
  <Icon icon={RefreshCw} size="sm" />
{/snippet}
{#snippet swapIcon()}
  <Icon icon={ArrowLeftRight} size="sm" />
{/snippet}
{#snippet convertArrowIcon()}
  <Icon icon={ArrowRight} size="sm" />
{/snippet}

<Card padding={5}>
  <VStack gap={4}>
    <!-- Header: title, live headline, period badge, range + refresh -->
    <HStack hAlign="between" vAlign="center" wrap="wrap" gap={3}>
      <VStack gap={1}>
        <Heading level={3}>Exchange rates</Heading>
        <HStack gap={2} vAlign="center" wrap="wrap">
          <Text weight="semibold" hasTabularNumbers>
            1 {baseCode} = {fmtRate(latestRate)}
            {targetCode}
          </Text>
          {#if periodPct !== null}
            <Badge variant={pctBadgeVariant(periodPct)} label={fmtPct(periodPct)} />
          {/if}
        </HStack>
      </VStack>

      <HStack gap={2} vAlign="center" wrap="wrap">
        <SegmentedControl label="Time range" size="sm" value={range} onChange={(v) => (range = v)}>
          {#each RANGES as r (r.value)}
            <SegmentedControlItem value={r.value} label={r.value} />
          {/each}
        </SegmentedControl>
        <IconButton
          label="Refresh rates"
          tooltip="Refresh latest rates and history"
          icon={refreshIcon}
          isLoading={loading}
          onclick={refreshRates}
        />
      </HStack>
    </HStack>

    <!-- Controls: base / swap / target / compare toggle -->
    <HStack gap={2} vAlign="end" wrap="wrap">
      <Stack width={220}>
        <CurrencySelect
          label="Base currency"
          showLabel
          value={baseCode}
          onchange={handleBaseChange}
        />
      </Stack>
      <IconButton
        label="Swap base and target"
        tooltip="Swap pair"
        icon={swapIcon}
        onclick={swapPair}
      />
      <Stack width={220}>
        <CurrencySelect
          label="Target currency"
          showLabel
          value={targetCode}
          onchange={handleTargetChange}
        />
      </Stack>
      <Button
        label={compareMode ? 'Comparing' : 'Compare'}
        size="md"
        variant={compareMode ? 'primary' : 'secondary'}
        tooltip="Overlay multiple currencies as % change"
        onclick={toggleCompareMode}
      />
    </HStack>

    <!-- Quick currency chips: switch target, or toggle overlays in compare mode -->
    <HStack gap={1.5} wrap="wrap">
      {#each visiblePopularCodes as code (code)}
        <Button
          label={code}
          size="sm"
          variant={isChipActive(code) ? 'primary' : 'secondary'}
          onclick={() => handleCurrencySelectOrCompare(code)}
        />
      {/each}
    </HStack>

    <!-- Compare legend -->
    {#if isCompareActive}
      <HStack gap={3} wrap="wrap" vAlign="center">
        {#each compareSeries as s, i (s.code)}
          <HStack gap={1} vAlign="center">
            <span class="erp-swatch" style:background={COMPARE_PALETTE[i % COMPARE_PALETTE.length]}
            ></span>
            <Text
              type="supporting"
              color="secondary"
              weight={s.code === targetCode ? 'semibold' : 'normal'}
            >
              {s.code}
            </Text>
          </HStack>
        {/each}
        <Text type="supporting" color="secondary">· % change vs period start</Text>
      </HStack>
    {/if}

    {#if showChartSkeleton}
      <Skeleton height={280} radius={3} />
    {:else if loadError && series.length === 0}
      <Text class="sunder-negative">{loadError}</Text>
    {:else}
      <ExchangeRateChart
        {series}
        compareSeries={isCompareActive ? compareSeries : []}
        {baseCode}
        {targetCode}
        height={280}
      />
    {/if}

    <!-- Period KPI strip -->
    <VStack gap={2}>
      {#if isCompareActive}
        <Text type="supporting" color="secondary">
          {baseCode} → {targetCode} summary ({range})
        </Text>
      {/if}
      <Grid columns={{ minWidth: 150, max: 4 }} gap={4}>
        <VStack gap={0.5}>
          <Text type="supporting" color="secondary">Period high</Text>
          <Text weight="semibold" hasTabularNumbers>{fmtRate(periodHigh)}</Text>
        </VStack>
        <VStack gap={0.5}>
          <Text type="supporting" color="secondary">Period low</Text>
          <Text weight="semibold" hasTabularNumbers>{fmtRate(periodLow)}</Text>
        </VStack>
        <VStack gap={0.5}>
          <Text type="supporting" color="secondary">Average</Text>
          <Text weight="semibold" hasTabularNumbers>{fmtRate(periodAvg)}</Text>
        </VStack>
        <VStack gap={0.5}>
          <Text type="supporting" color="secondary">Period change</Text>
          <Text weight="semibold" class={pctTextClass(periodPct)}>
            {periodAbs === null ? '—' : `${periodAbs > 0 ? '+' : ''}${fmtRate(periodAbs)}`}
            {periodPct !== null ? `(${fmtPct(periodPct)})` : ''}
          </Text>
        </VStack>
      </Grid>
    </VStack>

    <Divider />

    <!-- Converter + popular rates -->
    <Grid columns={{ minWidth: 300, max: 2 }} gap={6}>
      <VStack gap={2}>
        <Text type="supporting" color="secondary">Quick converter</Text>
        <div class="erp-converter-row">
          <div class="erp-converter-input">
            <TextInput
              label="Amount in {baseCode}"
              placeholder="1.00"
              value={amountStr}
              onChange={(v) => (amountStr = v)}
              controlClassName="sunder-tabular"
            />
          </div>
          <span class="erp-converter-arrow" aria-hidden="true">
            {@render convertArrowIcon()}
          </span>
          <VStack gap={0.5}>
            <Text type="supporting" color="secondary">{targetCode}</Text>
            <div class="erp-converter-output">
              <Heading level={3} class="sunder-tabular">
                {convertedAmount === null
                  ? '—'
                  : currencyStore.formatAmount(convertedAmount, targetCode)}
              </Heading>
            </div>
          </VStack>
        </div>
        <Text type="supporting" color="secondary" hasTabularNumbers>
          1 {targetCode} = {fmtRate(inverseRate)}
          {baseCode}
        </Text>
      </VStack>

      <VStack gap={2}>
        <Text type="supporting" color="secondary">
          Popular rates vs {baseCode}{compareMode ? ' · click to toggle overlay' : ''}
        </Text>
        <List hasDividers>
          {#each popularRows as row (row.code)}
            {#snippet popularLabel()}
              <HStack gap={2} vAlign="center">
                <Token label={row.code} size="sm" />
                <Text type="supporting" color="secondary" maxLines={1}>
                  {currencyStore.getCurrency(row.code).name}
                </Text>
              </HStack>
            {/snippet}
            {#snippet popularEnd()}
              <HStack gap={2} vAlign="center">
                <Text weight="semibold" hasTabularNumbers>{fmtRate(row.rate)}</Text>
                <Badge variant={pctBadgeVariant(row.pct)} label={fmtPct(row.pct)} />
              </HStack>
            {/snippet}
            <ListItem
              label={popularLabel}
              endContent={popularEnd}
              isSelected={isChipActive(row.code)}
              onclick={() => handleCurrencySelectOrCompare(row.code)}
            />
          {/each}
        </List>
      </VStack>
    </Grid>
  </VStack>
</Card>

<style>
  .erp-swatch {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: var(--radius-full);
    flex: none;
  }

  .erp-converter-row {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    gap: var(--spacing-3);
  }

  .erp-converter-input {
    width: min(100%, 160px);
    flex: 0 1 160px;
    min-width: 120px;
  }

  .erp-converter-arrow {
    display: inline-flex;
    align-items: center;
    height: var(--size-element-md, 36px);
    color: var(--color-text-secondary);
  }

  .erp-converter-output {
    display: flex;
    align-items: center;
    min-height: var(--size-element-md, 36px);
  }
</style>
