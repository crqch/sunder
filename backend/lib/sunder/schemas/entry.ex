defmodule Sunder.Eco.Entry do
  use Sunder.Schema
  import Ecto.Changeset

  @derive {Jason.Encoder, only: [:date, :amount, :title, :description, :location]}

  schema "account_entries" do
    field(:date, :naive_datetime)
    field(:description, :string)
    field(:title, :string)
    field(:location, :string)
    field(:amount, :float)

    belongs_to(:account, Sunder.Eco.Account)
    belongs_to(:category, Sunder.Eco.Category)
    belongs_to(:eco_user, Sunder.Eco.EcoUser)

    timestamps()
  end

  @doc false
  def changeset(entry, attrs) do
    entry
    |> cast(attrs, [
      :date,
      :amount,
      :title,
      :description,
      :location,
      :account_id,
      :category_id,
      :eco_user_id
    ])
    |> validate_required([
      :date,
      :amount,
      :title,
      :description,
      :location,
      :account_id,
      :category_id,
      :eco_user_id
    ])
  end
end
