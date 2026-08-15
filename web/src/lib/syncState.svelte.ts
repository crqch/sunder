export const syncState = $state<{
  status: 'idle' | 'unsynced' | 'syncing' | 'success' | 'error';
  autoSyncEnabled: boolean;
  autoSyncInterval: number; // in minutes
  minimized: boolean;
}>({
  status: 'idle',
  autoSyncEnabled:
    typeof window !== 'undefined' ? localStorage.getItem('sunder_auto_sync') !== 'false' : true,
  autoSyncInterval:
    typeof window !== 'undefined'
      ? parseInt(localStorage.getItem('sunder_auto_sync_interval') || '5')
      : 5,
  minimized:
    typeof window !== 'undefined' ? localStorage.getItem('sunder_sync_minimized') === 'true' : false
});
