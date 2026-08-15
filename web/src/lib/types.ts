export type User = {
  flags: string[];
  username: string;
  email: string;
};

export type Account = {
  id: string;
  name: string;
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
  deleted_at: string | null;
  created_at: string;
  updated_at: string;
};
