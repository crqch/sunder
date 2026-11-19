defmodule Sunder.Repo.Migrations.AddCurrencyFieldsToEcoTables do
  use Ecto.Migration

  def up do
    now = NaiveDateTime.truncate(NaiveDateTime.utc_now(), :second)

    rows =
      Enum.map(Sunder.Currencies.Catalog.all(), fn entry ->
        Map.merge(entry, %{
          is_active: true,
          inserted_at: now,
          updated_at: now
        })
      end)

    repo().insert_all("currencies", rows, on_conflict: :nothing, conflict_target: [:code])

    alter table(:eco_users) do
      add :default_currency_code,
          references(:currencies, column: :code, type: :string, on_delete: :restrict),
          null: false,
          default: "USD"
    end

    alter table(:accounts) do
      add :currency_code,
          references(:currencies, column: :code, type: :string, on_delete: :restrict),
          null: true
    end

    alter table(:account_entries) do
      add :currency_code,
          references(:currencies, column: :code, type: :string, on_delete: :restrict),
          null: false,
          default: "USD"
    end

    create index(:account_entries, [:currency_code])
    create index(:accounts, [:currency_code])
  end

  def down do
    drop_if_exists index(:accounts, [:currency_code])
    drop_if_exists index(:account_entries, [:currency_code])

    alter table(:account_entries) do
      remove :currency_code
    end

    alter table(:accounts) do
      remove :currency_code
    end

    alter table(:eco_users) do
      remove :default_currency_code
    end
  end
end
