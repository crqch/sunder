export type User = {
  flags: string[];
  username: string;
  email: string;
};

export type Account = {
  id: number;
  name: string;
  deleted_at: string | null;
  created_at: string;
  updated_at: string;
};

export type EntryCategory = {
  id: number;
  name: string;
  description: string | null;
  color: string;
  deleted_at: string | null;
  created_at: string;
  updated_at: string;
};

export type AccountEntry = {
  id: number;
  account_id: number;
  category_id: number;
  title: string;
  description: string | null;
  location: string | null;
  amount: number;
  deleted_at: string | null;
  created_at: string;
  updated_at: string;
};
