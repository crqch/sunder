import Dexie, { type EntityTable } from 'dexie';
import type { Account, AccountEntry, EntryCategory } from './types';

export const db = new Dexie('sunder-db') as Dexie & {
  accounts: EntityTable<Account, 'id'>;
  entry_categories: EntityTable<EntryCategory, 'id'>;
  account_entries: EntityTable<AccountEntry, 'id'>;
};

db.version(1).stores({
  accounts: 'id, name, deleted_at, created_at, updated_at',
  entry_categories: 'id, title, description, color, deleted_at, created_at, updated_at',
  account_entries:
    'id, account_id, category_id, title, description, location, amount, deleted_at, created_at, updated_at'
});
