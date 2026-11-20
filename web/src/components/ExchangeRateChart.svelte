<script module lang="ts">
  export type RatePoint = { date: string; rate: number };
  export type CompareSeries = { code: string; points: RatePoint[] };

  export const COMPARE_PALETTE = [
    'var(--color-accent)',
    'var(--color-text-blue)',
    'var(--color-text-orange)',
    'var(--color-text-purple)',
    'var(--color-text-teal)',
    'var(--color-text-pink)',
    'var(--color-text-green)',
    'var(--color-text-yellow)'
  ];

  export function formatFxRate(v: number): string {
    const abs = Math.abs(v);
    if (abs === 0) return '0.00';
    if (abs < 0.001) return v.toPrecision(4);
    if (abs < 0.1) return v.toFixed(5);
    if (abs < 10) return v.toFixed(4);
    if (abs < 1000) return v.toFixed(2);
    return new Intl.NumberFormat('en-US', {
      maximumFractionDigits: abs >= 10000 ? 0 : 2
    }).format(v);
  }
</script>

<script lang="ts">
  let {
    series = [],
    compareSeries = [],
    baseCode = 'USD',
    targetCode = 'EUR',
    height = 280
  }: {
    series?: RatePoint[];
    compareSeries?: CompareSeries[];
    baseCode?: string;
    targetCode?: string;
    height?: number;
  } = $props();

  const gradientId = $props.id();

  const MARGIN_TOP = 16;
  const MARGIN_RIGHT = 16;
  const MARGIN_BOTTOM = 28;

  let figureEl = $state<HTMLElement | null>(null);
  let width = $state(640);
  let activeIndex = $state<number | null>(null);

  $effect(() => {
    if (!figureEl) return;
    const initialW = figureEl.getBoundingClientRect().width;
    if (initialW > 0) width = initialW;
    const observer = new ResizeObserver((entries) => {
      const w = entries[0]?.contentRect.width;
      if (w && w > 0) width = w;
    });
    observer.observe(figureEl);
    return () => observer.disconnect();
  });

  let isCompare = $derived(compareSeries.length >= 2);

  type Line = {
    code: string;
    color: string;
    startDate: string;
    points: Array<{ date: string; value: number; rate: number }>;
  };

  // Build the sorted union of dates in compare mode so no series loses points.
  let axisDates = $derived.by(() => {
    if (isCompare) {
      const set = new Set<string>();
      for (const s of compareSeries) {
        for (const p of s.points) set.add(p.date);
      }
      return Array.from(set).sort();
    }
    return series.map((p) => p.date);
  });

  let trendColor = $derived.by(() => {
    if (series.length < 2) return 'var(--color-accent)';
    const first = series[0].rate;
    const last = series[series.length - 1].rate;
    const pct = first > 0 ? ((last - first) / first) * 100 : 0;
    if (pct > 0.005) return 'var(--color-text-success)';
    if (pct < -0.005) return 'var(--color-text-error)';
    return 'var(--color-accent)';
  });

  let lines = $derived.by<Line[]>(() => {
    if (isCompare) {
      return compareSeries
        .filter((s) => s.points.length > 0)
        .map((s, i) => {
          const first = s.points[0].rate;
          return {
            code: s.code,
            color: COMPARE_PALETTE[i % COMPARE_PALETTE.length],
            startDate: s.points[0].date,
            points: s.points.map((p) => ({
              date: p.date,
              value: first > 0 ? (p.rate / first - 1) * 100 : 0,
              rate: p.rate
            }))
          };
        });
    }
    if (series.length === 0) return [];
    return [
      {
        code: targetCode,
        color: trendColor,
        startDate: series[0].date,
        points: series.map((p) => ({ date: p.date, value: p.rate, rate: p.rate }))
      }
    ];
  });

  let hasMismatchedStartDates = $derived.by(() => {
    if (!isCompare || lines.length < 2) return false;
    const firstStart = lines[0].startDate;
    return lines.some((l) => l.startDate !== firstStart);
  });

  let domain = $derived.by(() => {
    const values: number[] = [];
    for (const line of lines) {
      for (const p of line.points) values.push(p.value);
    }
    if (values.length === 0) return { lo: 0, hi: 1 };
    let lo = Math.min(...values);
    let hi = Math.max(...values);
    if (isCompare) {
      lo = Math.min(lo, 0);
      hi = Math.max(hi, 0);
    }
    if (hi - lo < 1e-12) {
      const pad = Math.max(Math.abs(hi) * 0.01, 0.5);
      lo -= pad;
      hi += pad;
    } else {
      const pad = (hi - lo) * 0.08;
      lo -= pad;
      hi += pad;
    }
    return { lo, hi };
  });

  function formatTick(v: number): string {
    if (isCompare) {
      const span = domain.hi - domain.lo;
      const digits = span < 1 ? 2 : span < 10 ? 1 : 0;
      const sign = v > 0 ? '+' : '';
      return `${sign}${v.toFixed(digits)}%`;
    }
    const abs = Math.abs(v);
    if (abs >= 10000) {
      return new Intl.NumberFormat('en-US', {
        notation: 'compact',
        maximumFractionDigits: 1
      }).format(v);
    }
    return formatFxRate(v);
  }

  let yTickValues = $derived.by(() => {
    const { lo, hi } = domain;
    const vals: number[] = [];
    const steps = 4;
    for (let i = 0; i <= steps; i++) {
      vals.push(lo + ((hi - lo) * i) / steps);
    }
    return vals;
  });

  // Dynamic left margin prevents long tick labels (e.g. 0.000039 or 25,400) from clipping.
  let marginLeft = $derived.by(() => {
    let maxLen = 5;
    for (const v of yTickValues) {
      const len = formatTick(v).length;
      if (len > maxLen) maxLen = len;
    }
    return Math.max(54, Math.min(88, Math.ceil(maxLen * 7.1 + 14)));
  });

  let innerW = $derived(Math.max(width - marginLeft - MARGIN_RIGHT, 10));
  let innerH = $derived(Math.max(height - MARGIN_TOP - MARGIN_BOTTOM, 10));
  let baselineY = $derived(MARGIN_TOP + innerH);

  function xForIndex(i: number): number {
    const n = axisDates.length;
    if (n <= 1) return marginLeft + innerW / 2;
    return marginLeft + (i / (n - 1)) * innerW;
  }

  function yForValue(v: number): number {
    const { lo, hi } = domain;
    return MARGIN_TOP + innerH - ((v - lo) / (hi - lo)) * innerH;
  }

  let yTicks = $derived(
    yTickValues.map((v) => ({
      v,
      y: yForValue(v),
      label: formatTick(v)
    }))
  );

  let zeroLineY = $derived(isCompare && domain.lo < 0 && domain.hi > 0 ? yForValue(0) : null);

  let dateIndex = $derived.by(() => {
    const map = new Map<string, number>();
    axisDates.forEach((d, i) => map.set(d, i));
    return map;
  });

  type XY = { x: number; y: number };

  function lineXY(line: Line): XY[] {
    const out: XY[] = [];
    for (const p of line.points) {
      const idx = dateIndex.get(p.date);
      if (idx === undefined) continue;
      out.push({ x: xForIndex(idx), y: yForValue(p.value) });
    }
    return out;
  }

  // Fritsch-Carlson monotone cubic Hermite interpolation: smooth curves that
  // strictly preserve monotonicity and never overshoot peaks/troughs or bounds.
  function monotonePath(pts: XY[], minY: number, maxY: number): string {
    const n = pts.length;
    if (n === 0) return '';
    const f = (v: number) => v.toFixed(2);
    if (n === 1) return `M ${f(pts[0].x)} ${f(pts[0].y)}`;
    if (n === 2) return `M ${f(pts[0].x)} ${f(pts[0].y)} L ${f(pts[1].x)} ${f(pts[1].y)}`;

    const dx: number[] = new Array(n - 1);
    const slope: number[] = new Array(n - 1);
    for (let i = 0; i < n - 1; i++) {
      dx[i] = pts[i + 1].x - pts[i].x;
      slope[i] = dx[i] > 1e-6 ? (pts[i + 1].y - pts[i].y) / dx[i] : 0;
    }

    const m: number[] = new Array(n);
    m[0] = slope[0];
    m[n - 1] = slope[n - 2];
    for (let i = 1; i < n - 1; i++) {
      if (slope[i - 1] * slope[i] <= 0) {
        m[i] = 0;
      } else {
        const w1 = 2 * dx[i] + dx[i - 1];
        const w2 = dx[i] + 2 * dx[i - 1];
        m[i] = (w1 + w2) / (w1 / slope[i - 1] + w2 / slope[i]);
      }
    }

    for (let i = 0; i < n - 1; i++) {
      if (Math.abs(slope[i]) < 1e-9) {
        m[i] = 0;
        m[i + 1] = 0;
      } else {
        const alpha = m[i] / slope[i];
        const beta = m[i + 1] / slope[i];
        const h = Math.hypot(alpha, beta);
        if (h > 3) {
          const tau = 3 / h;
          m[i] = tau * alpha * slope[i];
          m[i + 1] = tau * beta * slope[i];
        }
      }
    }

    const clampY = (y: number) => Math.max(minY, Math.min(maxY, y));
    let d = `M ${f(pts[0].x)} ${f(pts[0].y)}`;
    for (let i = 0; i < n - 1; i++) {
      const p1 = pts[i];
      const p2 = pts[i + 1];
      const segDx = dx[i];
      const c1x = p1.x + segDx / 3;
      const c1y = clampY(p1.y + (m[i] * segDx) / 3);
      const c2x = p2.x - segDx / 3;
      const c2y = clampY(p2.y - (m[i + 1] * segDx) / 3);
      d += ` C ${f(c1x)} ${f(c1y)}, ${f(c2x)} ${f(c2y)}, ${f(p2.x)} ${f(p2.y)}`;
    }
    return d;
  }

  let renderedLines = $derived.by(() =>
    lines.map((line) => {
      const xy = lineXY(line);
      const path = monotonePath(xy, MARGIN_TOP, baselineY);
      const area =
        !isCompare && xy.length >= 2
          ? `${path} L ${xy[xy.length - 1].x.toFixed(2)} ${baselineY.toFixed(2)} L ${xy[0].x.toFixed(2)} ${baselineY.toFixed(2)} Z`
          : '';
      return { ...line, xy, path, area };
    })
  );

  let xLabels = $derived.by(() => {
    const n = axisDates.length;
    if (n === 0) return [];
    const target = Math.max(2, Math.min(6, Math.floor(innerW / 90)));
    const count = Math.min(target, n);
    const seen = new Set<number>();
    const labels: Array<{ x: number; label: string; anchor: 'start' | 'middle' | 'end' }> = [];
    for (let k = 0; k < count; k++) {
      const idx = count <= 1 ? 0 : Math.round((k * (n - 1)) / (count - 1));
      if (seen.has(idx)) continue;
      seen.add(idx);
      const anchor = count <= 1 ? 'middle' : k === 0 ? 'start' : k === count - 1 ? 'end' : 'middle';
      labels.push({ x: xForIndex(idx), label: formatShortDate(axisDates[idx]), anchor });
    }
    return labels;
  });

  function formatShortDate(dateStr: string): string {
    return new Date(`${dateStr}T00:00:00Z`).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      timeZone: 'UTC'
    });
  }

  function formatLongDate(dateStr: string): string {
    return new Date(`${dateStr}T00:00:00Z`).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      timeZone: 'UTC'
    });
  }

  function formatPct(pct: number): string {
    const sign = pct > 0 ? '+' : '';
    return `${sign}${pct.toFixed(2)}%`;
  }

  function pctClass(pct: number): string {
    if (pct > 0.005) return 'xr-positive';
    if (pct < -0.005) return 'xr-negative';
    return 'xr-flat';
  }

  function updateFromPointer(e: PointerEvent) {
    const n = axisDates.length;
    if (n === 0) return;
    const target = e.currentTarget as SVGSVGElement;
    const rect = target.getBoundingClientRect();
    const x = e.clientX - rect.left;
    if (n === 1) {
      activeIndex = 0;
      return;
    }
    const ratio = (x - marginLeft) / innerW;
    const idx = Math.round(ratio * (n - 1));
    activeIndex = Math.max(0, Math.min(n - 1, idx));
  }

  function handlePointerLeave() {
    activeIndex = null;
  }

  function handleKeyDown(e: KeyboardEvent) {
    const n = axisDates.length;
    if (n === 0) return;
    if (e.key === 'ArrowRight') {
      e.preventDefault();
      activeIndex = activeIndex === null ? n - 1 : Math.min(n - 1, activeIndex + 1);
    } else if (e.key === 'ArrowLeft') {
      e.preventDefault();
      activeIndex = activeIndex === null ? n - 1 : Math.max(0, activeIndex - 1);
    } else if (e.key === 'Home') {
      e.preventDefault();
      activeIndex = 0;
    } else if (e.key === 'End') {
      e.preventDefault();
      activeIndex = n - 1;
    } else if (e.key === 'Escape') {
      activeIndex = null;
    }
  }

  let activeDate = $derived(activeIndex !== null ? axisDates[activeIndex] : null);
  let activeX = $derived(activeIndex !== null ? xForIndex(activeIndex) : 0);

  let activeEntries = $derived.by(() => {
    if (activeDate === null) return [];
    return renderedLines
      .map((line) => {
        const point = line.points.find((p) => p.date === activeDate);
        if (!point) return null;
        const first = line.points[0];
        const pct = first.rate > 0 ? (point.rate / first.rate - 1) * 100 : 0;
        return {
          code: line.code,
          color: line.color,
          y: yForValue(point.value),
          rate: point.rate,
          pct
        };
      })
      .filter((e): e is NonNullable<typeof e> => e !== null);
  });

  // Compute clamped pixel left position so the tooltip never clips off left or right edges.
  const TOOLTIP_WIDTH = 204;
  let tooltipLeft = $derived.by(() => {
    if (activeIndex === null) return 8;
    const rightSpace = activeX + 14 + TOOLTIP_WIDTH <= width - 8;
    const raw = rightSpace ? activeX + 14 : activeX - 14 - TOOLTIP_WIDTH;
    return Math.max(8, Math.min(Math.max(8, width - TOOLTIP_WIDTH - 8), raw));
  });

  let ariaLabel = $derived(
    isCompare
      ? `Normalized percentage change of ${compareSeries.map((s) => s.code).join(', ')} versus ${baseCode} over the selected period. Use Left and Right arrow keys to inspect dates.`
      : `${baseCode} to ${targetCode} exchange rate over the selected period. Use Left and Right arrow keys to inspect dates.`
  );
</script>

<figure class="xr-chart" bind:this={figureEl} style:height="{height}px">
  {#if axisDates.length === 0}
    <p class="xr-empty">No exchange rate history for this pair and period yet.</p>
  {:else}
    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <svg
      {width}
      {height}
      viewBox="0 0 {width} {height}"
      role="img"
      aria-label={ariaLabel}
      tabindex="0"
      onpointerdown={updateFromPointer}
      onpointermove={updateFromPointer}
      onpointerleave={handlePointerLeave}
      onpointercancel={handlePointerLeave}
      onkeydown={handleKeyDown}
      onblur={handlePointerLeave}
    >
      <defs>
        <linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" stop-color={trendColor} stop-opacity="0.22" />
          <stop offset="100%" stop-color={trendColor} stop-opacity="0" />
        </linearGradient>
      </defs>

      <!-- horizontal gridlines + y-axis ticks -->
      {#each yTicks as tick}
        <line x1={marginLeft} x2={width - MARGIN_RIGHT} y1={tick.y} y2={tick.y} class="xr-grid" />
        <text x={marginLeft - 8} y={tick.y + 3.5} text-anchor="end" class="xr-tick">
          {tick.label}
        </text>
      {/each}

      <!-- 0% reference baseline in compare mode -->
      {#if zeroLineY !== null}
        <line
          x1={marginLeft}
          x2={width - MARGIN_RIGHT}
          y1={zeroLineY}
          y2={zeroLineY}
          class="xr-zero-line"
        />
      {/if}

      <!-- x-axis date labels -->
      {#each xLabels as label}
        <text x={label.x} y={height - 8} text-anchor={label.anchor} class="xr-tick">
          {label.label}
        </text>
      {/each}

      <!-- area fill (single-pair mode only) -->
      {#each renderedLines as line (line.code)}
        {#if line.area}
          <path d={line.area} fill="url(#{gradientId})" stroke="none" />
        {/if}
      {/each}

      <!-- series lines -->
      {#each renderedLines as line (line.code)}
        {#if line.xy.length >= 2}
          <path
            d={line.path}
            fill="none"
            stroke={line.color}
            stroke-width={isCompare ? 1.85 : 2}
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        {:else if line.xy.length === 1}
          <line
            x1={marginLeft}
            x2={width - MARGIN_RIGHT}
            y1={line.xy[0].y}
            y2={line.xy[0].y}
            stroke={line.color}
            stroke-opacity="0.35"
            stroke-dasharray="4 4"
          />
          <circle cx={line.xy[0].x} cy={line.xy[0].y} r="4" fill={line.color} />
        {/if}
      {/each}

      <!-- crosshair + active points -->
      {#if activeIndex !== null && activeEntries.length > 0}
        <line x1={activeX} x2={activeX} y1={MARGIN_TOP} y2={baselineY} class="xr-crosshair" />
        {#each activeEntries as entry (entry.code)}
          <circle
            cx={activeX}
            cy={entry.y}
            r="4"
            fill={entry.color}
            stroke="var(--color-background-card)"
            stroke-width="1.5"
          />
        {/each}
      {/if}
    </svg>

    {#if activeDate !== null && activeEntries.length > 0}
      <div class="xr-tooltip" style:left="{tooltipLeft}px">
        <p class="xr-tooltip-date">{formatLongDate(activeDate)}</p>
        {#if isCompare}
          {#each activeEntries as entry (entry.code)}
            <div class="xr-tooltip-row">
              <span class="xr-swatch" style:background={entry.color}></span>
              <span class="xr-tooltip-code">{entry.code}</span>
              <span class="xr-tooltip-rate">{formatFxRate(entry.rate)}</span>
              <span class="xr-tooltip-pct {pctClass(entry.pct)}">{formatPct(entry.pct)}</span>
            </div>
          {/each}
        {:else}
          {@const entry = activeEntries[0]}
          <p class="xr-tooltip-main">
            1 {baseCode} = <strong>{formatFxRate(entry.rate)}</strong>
            {targetCode}
          </p>
          <p class="xr-tooltip-pct {pctClass(entry.pct)}">
            {formatPct(entry.pct)} vs period start
          </p>
        {/if}
      </div>
    {/if}

    {#if axisDates.length === 1}
      <p class="xr-single-note">
        Only one snapshot available for this pair — daily history covers major ECB currencies and
        newly synced days.
      </p>
    {:else if hasMismatchedStartDates}
      <p class="xr-footnote">
        Each compared line is indexed to 0% at its own earliest available snapshot.
      </p>
    {/if}
  {/if}
</figure>

<style>
  .xr-chart {
    position: relative;
    width: 100%;
    max-width: 100%;
    overflow: hidden;
    margin: 0;
    font-variant-numeric: tabular-nums;
  }

  .xr-chart svg {
    display: block;
    width: 100%;
    max-width: 100%;
    touch-action: pan-y;
    cursor: crosshair;
    border-radius: var(--radius-element);
  }

  .xr-chart svg:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: -2px;
  }

  .xr-grid {
    stroke: var(--color-border);
    stroke-width: 1;
    stroke-dasharray: 3 4;
    opacity: 0.7;
  }

  .xr-zero-line {
    stroke: var(--color-border-emphasized);
    stroke-width: 1.25;
    opacity: 0.9;
  }

  .xr-tick {
    fill: var(--color-text-secondary);
    font-size: 11px;
    font-family: inherit;
  }

  .xr-crosshair {
    stroke: var(--color-border-emphasized);
    stroke-width: 1;
    stroke-dasharray: 3 3;
  }

  .xr-empty,
  .xr-single-note {
    margin: 0;
    color: var(--color-text-secondary);
    font-size: 0.85rem;
    text-align: center;
    padding-inline: var(--spacing-4);
  }

  .xr-empty {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .xr-single-note {
    position: absolute;
    left: 0;
    right: 0;
    top: calc(50% + 20px);
  }

  .xr-footnote {
    position: absolute;
    right: 16px;
    top: 4px;
    margin: 0;
    font-size: 0.7rem;
    color: var(--color-text-secondary);
    pointer-events: none;
  }

  .xr-tooltip {
    position: absolute;
    top: 8px;
    width: 204px;
    max-width: calc(100% - 16px);
    padding: 8px 10px;
    background: var(--color-background-popover);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-element);
    box-shadow: var(--shadow-med);
    pointer-events: none;
    z-index: 2;
  }

  .xr-tooltip-date {
    margin: 0 0 4px;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--color-text-secondary);
  }

  .xr-tooltip-main {
    margin: 0;
    font-size: 0.85rem;
    color: var(--color-text-primary);
  }

  .xr-tooltip-row {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.78rem;
    line-height: 1.7;
  }

  .xr-swatch {
    width: 8px;
    height: 8px;
    border-radius: var(--radius-full, 999px);
    flex: none;
  }

  .xr-tooltip-code {
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .xr-tooltip-rate {
    margin-left: auto;
    color: var(--color-text-secondary);
  }

  .xr-tooltip-pct {
    margin: 0;
    font-weight: 600;
    font-size: 0.78rem;
  }

  .xr-tooltip-row .xr-tooltip-pct {
    min-width: 52px;
    text-align: right;
  }

  .xr-positive {
    color: var(--color-text-success);
  }

  .xr-negative {
    color: var(--color-text-error);
  }

  .xr-flat {
    color: var(--color-accent);
  }
</style>
