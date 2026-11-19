import { db } from '$lib/db';
import { authenticatedFetch, authStore } from '$lib/auth';
import { currencyStore, normalizeCurrencyCode } from '$lib/currency.svelte';

const LAST_SYNC_KEY = 'sunder_last_sync_timestamp';
const EPOCH = '1970-01-01T00:00:00.000Z';

export async function syncAll() {
  let lastSync = localStorage.getItem(LAST_SYNC_KEY);
  if (!lastSync || lastSync === '0') {
    lastSync = EPOCH;
  }

  // Gather local changes since lastSync
  const accounts = await db.accounts.filter((a) => a.updated_at > lastSync!).toArray();
  const categories = await db.entry_categories.filter((c) => c.updated_at > lastSync!).toArray();
  const entries = await db.account_entries.filter((e) => e.updated_at > lastSync!).toArray();

  if (
    accounts.length === 0 &&
    categories.length === 0 &&
    entries.length === 0 &&
    lastSync !== EPOCH
  ) {
    // We still sync to fetch potential server-side changes
  }

  const payload = {
    accounts: accounts.map((a) => ({
      ...a,
      currency_code: a.currency_code ? normalizeCurrencyCode(a.currency_code) : null
    })),
    categories,
    entries: entries.map((e) => ({
      ...e,
      currency_code: normalizeCurrencyCode(e.currency_code),
      date: e.date || e.created_at
    }))
  };

  const res = await authenticatedFetch(`/dashboard/eco/sync?last=${encodeURIComponent(lastSync)}`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload)
  });

  if (!res.ok) {
    throw new Error('Sync failed with status ' + res.status);
  }

  const serverData = await res.json();
  const now = new Date().toISOString();

  await db.transaction('rw', db.accounts, db.entry_categories, db.account_entries, async () => {
    if (serverData.accounts?.length) {
      await db.accounts.bulkPut(
        serverData.accounts.map((a: Record<string, unknown>) => ({
          ...a,
          currency_code:
            typeof a.currency_code === 'string' ? normalizeCurrencyCode(a.currency_code) : null
        }))
      );
    }
    if (serverData.categories?.length) {
      await db.entry_categories.bulkPut(serverData.categories);
    }
    if (serverData.entries?.length) {
      await db.account_entries.bulkPut(
        serverData.entries.map((e: Record<string, unknown>) => ({
          ...e,
          currency_code: normalizeCurrencyCode(
            typeof e.currency_code === 'string' ? e.currency_code : null
          )
        }))
      );
    }
  });

  if (serverData.default_currency_code) {
    const code = normalizeCurrencyCode(serverData.default_currency_code);
    currencyStore.setDefaultCurrencyLocal(code);
    authStore.update((s) =>
      s.user ? { ...s, user: { ...s.user, default_currency_code: code } } : s
    );
  }

  localStorage.setItem(LAST_SYNC_KEY, now);

  // Refresh server-allowed currencies and historical exchange rates (non-blocking for data sync)
  currencyStore.syncAllCurrencyData().catch((err) => {
    console.warn('Currency data refresh failed during sync', err);
  });

  return serverData;
}

export async function getUnsyncedChanges() {
  let lastSync = localStorage.getItem(LAST_SYNC_KEY);
  if (!lastSync || lastSync === '0') {
    lastSync = EPOCH;
  }

  const accounts = await db.accounts.filter((a) => a.updated_at > lastSync!).toArray();
  const categories = await db.entry_categories.filter((c) => c.updated_at > lastSync!).toArray();
  const entries = await db.account_entries.filter((e) => e.updated_at > lastSync!).toArray();

  return { accounts, categories, entries };
}

export async function clearLocalDatabase() {
  await db.transaction('rw', db.accounts, db.entry_categories, db.account_entries, async () => {
    await db.accounts.clear();
    await db.entry_categories.clear();
    await db.account_entries.clear();
  });
  localStorage.removeItem(LAST_SYNC_KEY);
}
