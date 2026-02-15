defmodule Sunder.Repo.Migrations.CreateAccountEntries do
  use Ecto.Migration

  def change do
    create table(:account_entries) do
      add :date, :naive_datetime
      add :amount, :float
      add :title, :string
      add :description, :string
      add :location, :string
      add :account_id, references(:accounts, on_delete: :nothing)
      add :category, references(:categories, on_delete: :nothing)

      timestamps()
    end

    create index(:account_entries, [:account_id])
    create index(:account_entries, [:category])
  end
end
