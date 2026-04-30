defmodule Sunder.Accounts.User do
  use Sunder.Schema
  import Ecto.Changeset

  schema "users" do
    field :flags, {:array, :string}
    field :pass, :string
    field :username, :string
    field :email, :string

    timestamps()
  end

  @doc false
  def changeset(user, attrs) do
    user
    |> cast(attrs, [:email, :username, :pass, :flags])
    |> validate_required([:email, :username, :pass, :flags])
  end
end
