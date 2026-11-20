export interface UnsyncedData {
  accounts: any[];
  categories: any[];
  entries: any[];
}

export const syncState = $state<{
  status: 'idle' | 'unsynced' | 'syncing' | 'success' | 'error';
  autoSyncEnabled: boolean;
  autoSyncInterval: number; // in minutes
  minimized: boolean;
  pendingCount: number;
  unsyncedData: UnsyncedData;
  lastSyncedAt: string | null;
}>({
  status: 'idle',
  autoSyncEnabled:
    typeof window !== 'undefined' ? localStorage.getItem('sunder_auto_sync') !== 'false' : true,
  autoSyncInterval:
    typeof window !== 'undefined'
      ? parseInt(localStorage.getItem('sunder_auto_sync_interval') || '5')
      : 5,
  minimized:
    typeof window !== 'undefined'
      ? localStorage.getItem('sunder_sync_minimized') === 'true'
      : false,
  pendingCount: 0,
  unsyncedData: { accounts: [], categories: [], entries: [] },
  lastSyncedAt:
    typeof window !== 'undefined' ? localStorage.getItem('sunder_last_sync_timestamp') : null
});
