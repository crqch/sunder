defmodule Sunder.Repo.Migrations.CreateEcoUsers do
  use Ecto.Migration

  def change do
    create table(:eco_users) do
      add :user_id, :string

      timestamps()
    end

  end
end
