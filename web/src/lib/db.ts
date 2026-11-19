import Dexie, { type EntityTable } from 'dexie';
import type { Account, AccountEntry, Currency, EntryCategory, ExchangeRateSnapshot } from './types';

export const db = new Dexie('sunder-db') as Dexie & {
  accounts: EntityTable<Account, 'id'>;
  entry_categories: EntityTable<EntryCategory, 'id'>;
  account_entries: EntityTable<AccountEntry, 'id'>;
  currencies: EntityTable<Currency, 'code'>;
  exchange_rates: EntityTable<ExchangeRateSnapshot, 'date'>;
};

db.version(2).stores({
  accounts: 'id, name, deleted_at, created_at, updated_at',
  entry_categories: 'id, title, description, color, deleted_at, created_at, updated_at',
  account_entries:
    'id, account_id, category_id, title, description, location, amount, date, deleted_at, created_at, updated_at'
});

db.version(3)
  .stores({
    accounts: 'id, name, currency_code, deleted_at, created_at, updated_at',
    entry_categories: 'id, title, description, color, deleted_at, created_at, updated_at',
    account_entries:
      'id, account_id, category_id, title, description, location, amount, currency_code, date, deleted_at, created_at, updated_at',
    currencies: 'code, name, is_active',
    exchange_rates: 'date'
  })
  .upgrade(async (tx) => {
    await tx
      .table('account_entries')
      .toCollection()
      .modify((entry: Record<string, unknown>) => {
        if (!entry.currency_code) {
          entry.currency_code = 'USD';
        }
      });
  });
