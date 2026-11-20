import { apiFetch } from '$lib/api';
import { authenticatedFetch, authStore } from '$lib/auth';
import { db } from '$lib/db';
import type { AccountEntry, Currency, ExchangeRateSnapshot } from '$lib/types';

const DEFAULT_CURRENCY_STORAGE_KEY = 'sunder_default_currency';
const CANONICAL_BASE = 'USD';
const MAX_HISTORY_CHUNK_DAYS = 366;

export function normalizeCurrencyCode(code?: string | null): string {
  const cleaned = (code || CANONICAL_BASE).trim().toUpperCase();
  return cleaned || CANONICAL_BASE;
}

export function toDateKey(dateLike?: string | Date | null): string {
  if (!dateLike) return new Date().toISOString().slice(0, 10);
  if (dateLike instanceof Date) {
    if (isNaN(dateLike.getTime())) return new Date().toISOString().slice(0, 10);
    return dateLike.toISOString().slice(0, 10);
  }
  const str = String(dateLike).trim();
  if (/^\d{4}-\d{2}-\d{2}/.test(str)) {
    return str.slice(0, 10);
  }
  const parsed = new Date(str);
  if (isNaN(parsed.getTime())) return new Date().toISOString().slice(0, 10);
  return parsed.toISOString().slice(0, 10);
}

class CurrencyStore {
  currencies = $state<Currency[]>([]);
  allCurrencies = $state<Currency[]>([]);
  defaultCurrencyCode = $state<string>(
    typeof window !== 'undefined'
      ? normalizeCurrencyCode(localStorage.getItem(DEFAULT_CURRENCY_STORAGE_KEY))
      : CANONICAL_BASE
  );
  ratesByDate = $state<Record<string, Record<string, number>>>({});
  sortedDates = $state<string[]>([]);
  loaded = $state(false);
  syncingRates = $state(false);

  private currencyMap = $derived.by(() => {
    const map = new Map<string, Currency>();
    for (const c of this.allCurrencies) {
      map.set(c.code.toUpperCase(), c);
    }
    for (const c of this.currencies) {
      map.set(c.code.toUpperCase(), c);
    }
    return map;
  });

  setDefaultCurrencyLocal(code?: string | null) {
    if (!code) return;
    const normalized = normalizeCurrencyCode(code);
    this.defaultCurrencyCode = normalized;
    if (typeof window !== 'undefined') {
      localStorage.setItem(DEFAULT_CURRENCY_STORAGE_KEY, normalized);
    }
  }

  getCurrency(code?: string | null): Currency {
    const normalized = normalizeCurrencyCode(code);
    const found = this.currencyMap.get(normalized);
    if (found) return found;

    return {
      code: normalized,
      name: normalized,
      symbol: normalized === 'USD' ? '$' : normalized,
      symbol_after: normalized !== 'USD',
      decimal_digits: 2
    };
  }

  formatAmount(
    amount: number,
    code?: string | null,
    options: { showPlus?: boolean; absolute?: boolean } = {}
  ): string {
    const currency = this.getCurrency(code);
    const digits =
      typeof currency.decimal_digits === 'number' && currency.decimal_digits >= 0
        ? currency.decimal_digits
        : 2;

    const isNegative = amount < 0;
    const value = Math.abs(amount);
    const formattedNumber = value.toFixed(digits);

    let sign = '';
    if (!options.absolute) {
      if (isNegative) sign = '-';
      else if (options.showPlus && amount > 0) sign = '+';
    }

    if (currency.symbol_after) {
      return `${sign}${formattedNumber} ${currency.symbol}`;
    }

    // Add a space if the symbol is multi-character alphabetic (like "CHF" or "Rs")
    const needsSpace = currency.symbol.length > 1 && /^[A-Za-z.]+$/.test(currency.symbol);
    return needsSpace
      ? `${sign}${currency.symbol} ${formattedNumber}`
      : `${sign}${currency.symbol}${formattedNumber}`;
  }

  private findUsdRateForCurrency(code: string, dateKey: string): number | null {
    if (code === CANONICAL_BASE) return 1;
    const dates = this.sortedDates;
    if (dates.length === 0) return null;

    // Binary search for largest index with dates[idx] <= dateKey
    let lo = 0;
    let hi = dates.length - 1;
    let idx = -1;
    while (lo <= hi) {
      const mid = (lo + hi) >> 1;
      if (dates[mid] <= dateKey) {
        idx = mid;
        lo = mid + 1;
      } else {
        hi = mid - 1;
      }
    }

    // Search backwards from idx for nearest prior rate containing `code`
    for (let i = idx; i >= 0; i--) {
      const rate = this.ratesByDate[dates[i]]?.[code];
      if (typeof rate === 'number' && rate > 0) return rate;
    }

    // Fallback: search forwards from idx + 1 (if entry predates earliest available snapshot)
    for (let i = Math.max(0, idx + 1); i < dates.length; i++) {
      const rate = this.ratesByDate[dates[i]]?.[code];
      if (typeof rate === 'number' && rate > 0) return rate;
    }

    return null;
  }

  getRate(
    fromCode?: string | null,
    toCode?: string | null,
    dateLike?: string | Date | null
  ): number | null {
    const from = normalizeCurrencyCode(fromCode);
    const to = normalizeCurrencyCode(toCode ?? this.defaultCurrencyCode);
    if (from === to) return 1;

    const dateKey = toDateKey(dateLike);
    const usdToFrom = this.findUsdRateForCurrency(from, dateKey);
    const usdToTo = this.findUsdRateForCurrency(to, dateKey);
    if (!usdToFrom || !usdToTo) return null;

    return usdToTo / usdToFrom;
  }

  tryConvert(
    amount: number,
    fromCode?: string | null,
    toCode?: string | null,
    dateLike?: string | Date | null
  ): { amount: number; converted: boolean; rate: number | null } {
    const from = normalizeCurrencyCode(fromCode);
    const to = normalizeCurrencyCode(toCode ?? this.defaultCurrencyCode);
    if (from === to) {
      return { amount, converted: true, rate: 1 };
    }

    const rate = this.getRate(from, to, dateLike);
    if (rate === null) {
      return { amount, converted: false, rate: null };
    }

    return { amount: amount * rate, converted: true, rate };
  }

  convertToDefault(
    amount: number,
    fromCode?: string | null,
    dateLike?: string | Date | null
  ): number {
    return this.tryConvert(amount, fromCode, this.defaultCurrencyCode, dateLike).amount;
  }

  convertEntryToDefault(
    entry: Pick<AccountEntry, 'amount' | 'currency_code' | 'date' | 'created_at'>
  ): number {
    return this.convertToDefault(
      entry.amount,
      entry.currency_code || CANONICAL_BASE,
      entry.date || entry.created_at
    );
  }

  async loadFromCache() {
    if (typeof window === 'undefined') return;
    try {
      const [cachedCurrencies, cachedSnapshots] = await Promise.all([
        db.currencies.orderBy('code').toArray(),
        db.exchange_rates.orderBy('date').toArray()
      ]);

      if (cachedCurrencies.length > 0) {
        this.allCurrencies = cachedCurrencies;
        this.currencies = cachedCurrencies.filter((c) => c.is_active !== false);
      }

      const map: Record<string, Record<string, number>> = {};
      for (const snap of cachedSnapshots) {
        map[snap.date] = snap.rates;
      }
      this.ratesByDate = map;
      this.sortedDates = Object.keys(map).sort();
      this.loaded = true;
    } catch (err) {
      console.error('Failed to load currency cache', err);
    }
  }

  async refreshCurrencies(): Promise<Currency[]> {
    const res = await apiFetch('/currencies');
    if (!res.ok) {
      throw new Error(`Failed to fetch currencies (${res.status})`);
    }
    const data = (await res.json()) as { currencies?: Currency[] };
    const activeList: Currency[] = (data.currencies || []).map((c: Currency) => ({
      code: normalizeCurrencyCode(c.code),
      name: c.name,
      symbol: c.symbol,
      symbol_after: Boolean(c.symbol_after),
      decimal_digits: typeof c.decimal_digits === 'number' ? c.decimal_digits : 2,
      is_active: c.is_active ?? true
    }));

    if (activeList.length === 0) {
      this.loaded = true;
      return this.currencies;
    }

    const mergedByCode = new Map<string, Currency>();
    const existing = await db.currencies.toArray();
    for (const prev of existing) {
      const code = normalizeCurrencyCode(prev.code);
      mergedByCode.set(code, { ...prev, code, is_active: false });
    }
    for (const active of activeList) {
      mergedByCode.set(active.code, active);
    }
    const mergedList = Array.from(mergedByCode.values()).sort((a, b) =>
      a.code.localeCompare(b.code)
    );

    await db.transaction('rw', db.currencies, async () => {
      await db.currencies.bulkPut(mergedList);
    });

    this.allCurrencies = mergedList;
    this.currencies = mergedList.filter((c) => c.is_active !== false);
    this.loaded = true;
    return this.currencies;
  }

  private async mergeAndStoreSnapshots(snapshots: Record<string, Record<string, number>>) {
    const dates = Object.keys(snapshots);
    if (dates.length === 0) return;

    const existing = await db.exchange_rates.where('date').anyOf(dates).toArray();
    const existingByDate = new Map(existing.map((s) => [s.date, s]));

    const toPut: ExchangeRateSnapshot[] = dates.map((date) => {
      const prev = existingByDate.get(date);
      return {
        date,
        base: CANONICAL_BASE,
        rates: {
          ...(prev?.rates || {}),
          ...snapshots[date],
          [CANONICAL_BASE]: 1
        }
      };
    });

    await db.exchange_rates.bulkPut(toPut);

    const nextMap = { ...this.ratesByDate };
    for (const snap of toPut) {
      nextMap[snap.date] = snap.rates;
    }
    this.ratesByDate = nextMap;
    this.sortedDates = Object.keys(nextMap).sort();
  }

  async refreshLatestRates(symbols?: string[]) {
    const params = new URLSearchParams({ base: CANONICAL_BASE });
    if (symbols && symbols.length > 0) {
      params.set('symbols', symbols.join(','));
    }
    const res = await apiFetch(`/currencies/rates?${params.toString()}`);
    if (!res.ok) return;
    const data = (await res.json()) as { date?: string; rates?: Record<string, number> };
    if (data?.date && data?.rates) {
      await this.mergeAndStoreSnapshots({ [data.date]: data.rates });
    }
  }

  private recentRangeFetches = new Map<string, number>();

  private async fetchHistoryRange(fromStr: string, toStr: string, symbolsParam: string) {
    let cursor = new Date(`${fromStr}T00:00:00Z`);
    const end = new Date(`${toStr}T00:00:00Z`);

    while (cursor <= end) {
      const chunkEnd = new Date(cursor.getTime());
      chunkEnd.setUTCDate(chunkEnd.getUTCDate() + MAX_HISTORY_CHUNK_DAYS);
      const effectiveEnd = chunkEnd > end ? end : chunkEnd;

      const params = new URLSearchParams({
        base: CANONICAL_BASE,
        from: cursor.toISOString().slice(0, 10),
        to: effectiveEnd.toISOString().slice(0, 10)
      });
      if (symbolsParam) {
        params.set('symbols', symbolsParam);
      }

      const res = await apiFetch(`/currencies/rates/history?${params.toString()}`);
      if (!res.ok) {
        throw new Error(`Failed to fetch exchange rate history (HTTP ${res.status})`);
      }
      const data = (await res.json()) as {
        rates?: Record<string, Record<string, number>>;
      };
      if (data?.rates && typeof data.rates === 'object') {
        await this.mergeAndStoreSnapshots(data.rates);
      }

      cursor = new Date(effectiveEnd.getTime());
      cursor.setUTCDate(cursor.getUTCDate() + 1);
    }
  }

  private countWeekdays(fromStr: string, toStr: string): number {
    let count = 0;
    const cursor = new Date(`${fromStr}T00:00:00Z`);
    const end = new Date(`${toStr}T00:00:00Z`);
    while (cursor <= end) {
      const dow = cursor.getUTCDay();
      if (dow !== 0 && dow !== 6) count++;
      cursor.setUTCDate(cursor.getUTCDate() + 1);
    }
    return count;
  }

  private hasRangeCoverage(fromStr: string, toStr: string, symbols: string[]): boolean {
    const dates = this.sortedDates.filter((d) => d >= fromStr && d <= toStr);
    if (dates.length === 0) return false;

    let expected = this.countWeekdays(fromStr, toStr);
    // Today's rates may not be published upstream yet; don't require them.
    if (toStr === toDateKey(new Date()) && expected > 0) expected -= 1;
    // Allow ~15% tolerance for ECB public holidays so holiday-spanning windows
    // do not trigger redundant refetches on every mount.
    const minRequired =
      expected <= 5 ? Math.max(expected - 1, 1) : Math.max(Math.floor(expected * 0.85), 1);
    if (dates.length < minRequired) return false;

    const firstDateMs = Date.parse(`${dates[0]}T00:00:00Z`);
    const fromDateMs = Date.parse(`${fromStr}T00:00:00Z`);
    if (Number.isFinite(firstDateMs) && Number.isFinite(fromDateMs)) {
      const leadingGapDays = (firstDateMs - fromDateMs) / 86_400_000;
      if (leadingGapDays > 7) return false;
    }

    if (symbols.length > 0) {
      const first = dates[0];
      const last = dates[dates.length - 1];
      if (!this.hasRateForAllSymbolsOnDate(first, symbols)) return false;
      if (!this.hasRateForAllSymbolsOnDate(last, symbols)) return false;
    }
    return true;
  }

  /**
   * Ensures daily USD-based snapshots covering [fromStr, toStr] are present in
   * IndexedDB and the reactive store, fetching (chunked) history from the API
   * when local coverage is sparse. An empty/omitted `symbols` list fetches all
   * stored currencies for the range.
   */
  async ensureHistoryForRange(fromStr: string, toStr: string, symbols?: string[], force = false) {
    if (typeof window === 'undefined') return;
    const from = toDateKey(fromStr);
    const today = toDateKey(new Date());
    const requestedTo = toDateKey(toStr);
    const to = requestedTo > today ? today : requestedTo;
    if (from > to) return;

    const normalizedSymbols = Array.from(
      new Set(
        (symbols ?? []).map((s) => normalizeCurrencyCode(s)).filter((s) => s !== CANONICAL_BASE)
      )
    ).sort();

    if (!force && this.hasRangeCoverage(from, to, normalizedSymbols)) return;

    const cacheKey = `${from}:${to}:${normalizedSymbols.join(',')}`;
    const lastFetched = this.recentRangeFetches.get(cacheKey);
    if (!force && lastFetched && Date.now() - lastFetched < 5 * 60 * 1000) {
      return;
    }

    await this.fetchHistoryRange(from, to, normalizedSymbols.join(','));
    this.recentRangeFetches.set(cacheKey, Date.now());
  }

  /**
   * Clean daily cross-rate series (usdToTarget / usdToBase) for every stored
   * snapshot date within [fromStr, toStr]. Dates missing either leg are skipped.
   */
  getRateSeries(
    baseCode: string,
    targetCode: string,
    fromStr: string,
    toStr: string
  ): Array<{ date: string; rate: number }> {
    const base = normalizeCurrencyCode(baseCode);
    const target = normalizeCurrencyCode(targetCode);
    const from = toDateKey(fromStr);
    const to = toDateKey(toStr);
    if (from > to) return [];

    const series: Array<{ date: string; rate: number }> = [];
    for (const date of this.sortedDates) {
      if (date < from) continue;
      if (date > to) break;
      if (base === target) {
        series.push({ date, rate: 1 });
        continue;
      }
      const table = this.ratesByDate[date];
      if (!table) continue;
      const usdToBase = base === CANONICAL_BASE ? 1 : table[base];
      const usdToTarget = target === CANONICAL_BASE ? 1 : table[target];
      if (
        typeof usdToBase !== 'number' ||
        usdToBase <= 0 ||
        typeof usdToTarget !== 'number' ||
        usdToTarget <= 0
      ) {
        continue;
      }
      series.push({ date, rate: usdToTarget / usdToBase });
    }
    return series;
  }

  private hasRateForAllSymbolsOnDate(date: string, symbols: string[]): boolean {
    const table = this.ratesByDate[date];
    if (!table) return false;
    return symbols.every((sym) => typeof table[sym] === 'number' && table[sym] > 0);
  }

  async refreshRatesForEntries(force = false) {
    if (typeof window === 'undefined' || this.syncingRates) return;
    this.syncingRates = true;
    try {
      const entries = await db.account_entries.filter((e) => !e.deleted_at).toArray();
      const neededCodes = new Set<string>([this.defaultCurrencyCode]);
      let earliestDate: string | null = null;

      for (const e of entries) {
        const code = normalizeCurrencyCode(e.currency_code);
        neededCodes.add(code);
        const dateKey = toDateKey(e.date || e.created_at);
        if (!earliestDate || dateKey < earliestDate) {
          earliestDate = dateKey;
        }
      }

      // Always refresh latest rates for all active currencies so conversions work immediately
      await this.refreshLatestRates();

      // If multiple currencies are in use, fetch historical rates over the entries' date span
      const symbols = Array.from(neededCodes).filter((c) => c !== CANONICAL_BASE);
      if (symbols.length > 0 && earliestDate) {
        const today = toDateKey(new Date());
        const clampedStart = earliestDate > today ? today : earliestDate;
        const symbolsParam = symbols.join(',');

        const cachedEarliest = this.sortedDates[0];
        const cachedLatest = this.sortedDates[this.sortedDates.length - 1];
        const symbolsCachedAtBounds =
          cachedEarliest &&
          cachedLatest &&
          this.hasRateForAllSymbolsOnDate(cachedEarliest, symbols) &&
          this.hasRateForAllSymbolsOnDate(cachedLatest, symbols);

        if (force || !cachedEarliest || !cachedLatest || !symbolsCachedAtBounds) {
          await this.fetchHistoryRange(clampedStart, today, symbolsParam);
        } else {
          if (clampedStart < cachedEarliest) {
            const prevDay = new Date(`${cachedEarliest}T00:00:00Z`);
            prevDay.setUTCDate(prevDay.getUTCDate() - 1);
            const prefixEnd = prevDay.toISOString().slice(0, 10);
            if (clampedStart <= prefixEnd) {
              await this.fetchHistoryRange(clampedStart, prefixEnd, symbolsParam);
            }
          }
          if (cachedLatest < today) {
            await this.fetchHistoryRange(cachedLatest, today, symbolsParam);
          }
        }
      }
    } catch (err) {
      console.error('Failed to refresh exchange rates', err);
    } finally {
      this.syncingRates = false;
    }
  }

  async refreshPreferences() {
    try {
      const res = await authenticatedFetch('/dashboard/eco/preferences');
      if (!res.ok) return;
      const data = (await res.json()) as { default_currency_code?: string };
      if (data?.default_currency_code) {
        this.setDefaultCurrencyLocal(data.default_currency_code);
        authStore.update((s) =>
          s.user
            ? { ...s, user: { ...s.user, default_currency_code: data.default_currency_code } }
            : s
        );
      }
    } catch (err) {
      console.error('Failed to refresh currency preferences', err);
    }
  }

  async updateDefaultCurrency(code: string): Promise<string> {
    const normalized = normalizeCurrencyCode(code);
    const res = await authenticatedFetch('/dashboard/eco/preferences', {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ default_currency_code: normalized })
    });
    const data = (await res.json()) as {
      error?: string;
      default_currency_code?: string;
    };
    if (!res.ok) {
      throw new Error(data.error || 'Failed to update default currency');
    }
    const updatedCode = normalizeCurrencyCode(data.default_currency_code || normalized);
    this.setDefaultCurrencyLocal(updatedCode);
    authStore.update((s) =>
      s.user ? { ...s, user: { ...s.user, default_currency_code: updatedCode } } : s
    );
    await this.refreshRatesForEntries();
    return updatedCode;
  }

  async syncAllCurrencyData(force = false) {
    await this.loadFromCache();
    await Promise.allSettled([this.refreshCurrencies(), this.refreshPreferences()]);
    await this.refreshRatesForEntries(force);
  }
}

export const currencyStore = new CurrencyStore();
