defmodule Sunder.Repo.Migrations.CreateInvites do
  use Ecto.Migration

  def change do
    create table(:invites) do
      add :token, :string
      add :used_by, :string

      timestamps()
    end

  end
end
