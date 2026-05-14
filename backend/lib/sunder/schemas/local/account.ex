defmodule Sunder.Eco.Account do
  use Sunder.Schema
  import Ecto.Changeset

  @derive {Jason.Encoder, only: [:name]}

  local_schema "accounts" do
    field(:name, :string)

    belongs_to(:eco_user, Sunder.Eco.EcoUser)

    has_many(:entries, Sunder.Eco.Entry)
  end

  @doc false
  def create_changeset(account, attrs) do
    account
    |> cast(attrs, [:eco_user_id, :name])
    |> validate_required([:eco_user_id, :name])
  end

  @doc false
  def update_changeset(account, attrs) do
    account
    |> cast(attrs, [:eco_user_id])
    |> validate_required([:eco_user_id])
  end
end
