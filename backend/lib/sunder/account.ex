defmodule Sunder.Account do
  use Sunder.Schema
  import Ecto.Changeset

  schema "accounts" do
    field(:name, :string)

    timestamps()
  end

  @doc false
  def changeset(account, attrs) do
    account
    |> cast(attrs, [:id, :name])
    |> validate_required([:id, :name])
  end
end
