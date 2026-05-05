defmodule Sunder.Repo.Migrations.CreateEntries do
  use Ecto.Migration

  def change do
    create table(:account_entries) do
      add :date, :naive_datetime
      add :description, :string
      add :title, :string
      add :location, :string
      add :amount, :float

      add :eco_user_id, :string
      add :account_id, :string
      add :category_id, :string

      timestamps()
    end

  end
end
