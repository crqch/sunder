defmodule Sunder.Account do
  use Sunder.Schema
  import Ecto.Changeset

  @derive {Jason.Encoder, only: [:name]}

  schema "accounts" do
    field(:name, :string)
    field(:user_id, :string)

    timestamps()
  end

  @doc false
  def changeset(account, attrs) do
    account
    |> cast(attrs, [:user_id, :name])
    |> validate_required([:user_id, :name])
  end
end
