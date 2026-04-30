defmodule Sunder.Repo.Migrations.CreateUsers do
  use Ecto.Migration

  def change do
    create table(:users) do
      add :email, :string
      add :username, :string
      add :pass, :string
      add :flags, {:array, :string}

      timestamps()
    end

  end
end
