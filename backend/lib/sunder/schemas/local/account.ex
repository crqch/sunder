defmodule Sunder.Eco.Account do
  use Sunder.Schema
  import Ecto.Changeset

  @derive {Jason.Encoder, only: [:id, :name, :deleted_at, :inserted_at, :updated_at]}

  schema "accounts" do
    field(:name, :string)

    belongs_to(:eco_user, Sunder.Eco.EcoUser)

    has_many(:entries, Sunder.Eco.Entry)

    field(:deleted_at, :utc_datetime)
    timestamps()
  end

  @doc false
  def changeset(account, attrs) do
    account
    |> cast(attrs, [:id, :eco_user_id, :name, :deleted_at, :updated_at])
    |> validate_required([:eco_user_id, :name])
  end

  @doc false
  def update_changeset(account, attrs) do
    account
    |> cast(attrs, [:eco_user_id])
    |> validate_required([:eco_user_id])
  end
end
