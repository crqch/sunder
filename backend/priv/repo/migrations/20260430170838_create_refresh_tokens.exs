defmodule Sunder.Repo.Migrations.CreateRefreshTokens do
  use Ecto.Migration

  def change do
    create table(:refresh_tokens) do
      add :token, :string
      add :expires_at, :naive_datetime
      add :user_id, :string

      timestamps()
    end

  end
end
