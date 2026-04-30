defmodule Sunder.Repo.Migrations.CreateAccessTokens do
  use Ecto.Migration

  def change do
    create table(:access_tokens) do
      add :token, :string
      add :expires_at, :naive_datetime
      add :user_id, :string

      timestamps()
    end

  end
end
