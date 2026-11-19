export type Currency = {
  code: string;
  name: string;
  symbol: string;
  symbol_after: boolean;
  decimal_digits: number;
  is_active?: boolean;
};

export type ExchangeRateSnapshot = {
  date: string;
  base: string;
  rates: Record<string, number>;
};

export type User = {
  id?: string;
  flags: string[];
  username: string;
  email: string;
  default_currency_code?: string;
};

export type Account = {
  id: string;
  name: string;
  currency_code?: string | null;
  deleted_at: string | null;
  created_at: string;
  updated_at: string;
};

export type EntryCategory = {
  id: string;
  title: string;
  description: string | null;
  color: string;
  deleted_at: string | null;
  created_at: string;
  updated_at: string;
};

export type AccountEntry = {
  id: string;
  account_id: string;
  category_id: string;
  title: string;
  description: string | null;
  location: string | null;
  amount: number;
  currency_code: string;
  date: string;
  deleted_at: string | null;
  created_at: string;
  updated_at: string;
};
