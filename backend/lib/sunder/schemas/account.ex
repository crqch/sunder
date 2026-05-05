defmodule Sunder.Eco.Account do
  use Sunder.Schema
  import Ecto.Changeset

  @derive {Jason.Encoder, only: [:name]}

  schema "accounts" do
    field(:name, :string)

    belongs_to(:eco_user, Sunder.Eco.EcoUser)

    has_many(:entries, Sunder.Eco.Entry)

    timestamps()
  end

  @doc false
  def changeset(account, attrs) do
    account
    |> cast(attrs, [:eco_user_id, :name])
    |> validate_required([:eco_user_id, :name])
  end
end
