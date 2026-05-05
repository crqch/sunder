defmodule Sunder.Repo.Migrations.CreateEntryCategories do
  use Ecto.Migration

  def change do
    create table(:entry_categories) do
      add :title, :string
      add :description, :string
      add :color, :string
      add :eco_user_id, :string

      timestamps()
    end

  end
end
