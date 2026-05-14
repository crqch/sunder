defmodule Sunder.Accounts.AccessToken do
  use Sunder.Schema
  import Ecto.Changeset

  @derive {Jason.Encoder, only: [:token]}

  schema "access_tokens" do
    field(:token, :string)
    field(:expires_at, :naive_datetime)

    belongs_to(:user, Sunder.Accounts.User)

    timestamps()
  end

  @doc false
  def changeset(access_token, attrs) do
    access_token
    |> cast(attrs, [:token, :expires_at, :user_id])
    |> validate_required([:token, :expires_at, :user_id])
  end
end
