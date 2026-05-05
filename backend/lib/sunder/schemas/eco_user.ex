defmodule Sunder.Eco.EcoUser do
  use Ecto.Schema
  import Ecto.Changeset

  schema "eco_users" do
    has_many(:entries, Sunder.Eco.Entry)
    has_many(:accounts, Sunder.Eco.Account)
    has_many(:categories, Sunder.Eco.Category)

    belongs_to(:user, Sunder.Accounts.User)

    timestamps()
  end

  @doc false
  def changeset(eco_user, attrs) do
    eco_user
    |> cast(attrs, [])
    |> validate_required([:user_id])
  end
end
