import { db } from '$lib/db';
import { authenticatedFetch } from '$lib/auth';

const LAST_SYNC_KEY = 'sunder_last_sync_timestamp';

export async function syncAll() {
  const lastSync = localStorage.getItem(LAST_SYNC_KEY) || '0';

  // Gather local changes since lastSync
  const accounts = await db.accounts.filter((a) => a.updated_at > lastSync).toArray();
  const categories = await db.entry_categories.filter((c) => c.updated_at > lastSync).toArray();
  const entries = await db.account_entries.filter((e) => e.updated_at > lastSync).toArray();

  if (
    accounts.length === 0 &&
    categories.length === 0 &&
    entries.length === 0 &&
    lastSync !== '0'
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
