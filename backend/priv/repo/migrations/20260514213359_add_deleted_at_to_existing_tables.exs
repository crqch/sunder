defmodule Sunder.Repo.Migrations.AddDeletedAtToExistingTables do
  use Ecto.Migration

  def change do
    alter table("accounts") do
      add :deleted_at, :utc_datetime
    end

    alter table("entry_categories") do
      add :deleted_at, :utc_datetime
    end

    alter table("account_entries") do
      add :deleted_at, :utc_datetime
    end
  end
end
