import { db } from '$lib/db';
import { authenticatedFetch } from '$lib/auth';

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

  const payload = { accounts, categories, entries };

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
      await db.accounts.bulkPut(serverData.accounts);
    }
    if (serverData.categories?.length) {
      await db.entry_categories.bulkPut(serverData.categories);
    }
    if (serverData.entries?.length) {
      await db.account_entries.bulkPut(serverData.entries);
    }
  });

  localStorage.setItem(LAST_SYNC_KEY, now);
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
