defmodule Sunder.Repo.Migrations.CreateAccounts do
  use Ecto.Migration

  def change do
    create table(:accounts) do
      add :user_id, :string
      add :name, :string

      timestamps()
    end

  end
end
