defmodule Sunder.Repo.Migrations.CreateCurrenciesAndExchangeRates do
  use Ecto.Migration

  def change do
    create table(:currencies, primary_key: false) do
      add :code, :string, primary_key: true
      add :name, :string, null: false
      add :symbol, :string, null: false
      add :symbol_after, :boolean, default: false, null: false
      add :decimal_digits, :integer, default: 2, null: false
      add :is_active, :boolean, default: true, null: false

      timestamps()
    end

    create index(:currencies, [:is_active])

    create table(:exchange_rates) do
      add :base_currency_code,
          references(:currencies, column: :code, type: :string, on_delete: :restrict),
          null: false,
          default: "USD"

      add :target_currency_code,
          references(:currencies, column: :code, type: :string, on_delete: :restrict),
          null: false

      add :rate, :decimal, precision: 24, scale: 12, null: false
      add :rate_date, :date, null: false
      add :source, :string, null: false, default: "unknown"

      timestamps()
    end

    create unique_index(:exchange_rates, [:base_currency_code, :target_currency_code, :rate_date])
    create index(:exchange_rates, [:target_currency_code, :rate_date])
    create index(:exchange_rates, [:rate_date])
  end
end
