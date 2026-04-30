defmodule Sunder.Account.Entry do
  use Sunder.Schema
  import Ecto.Changeset

  schema "account_entries" do
    field(:date, :naive_datetime)
    field(:description, :string)
    field(:title, :string)
    field(:location, :string)
    field(:amount, :float)
    field(:account_id, :id)
    field(:category_id, :id)

    timestamps()
  end

  @doc false
  def changeset(entry, attrs) do
    entry
    |> cast(attrs, [:date, :amount, :title, :description, :location])
    |> validate_required([:date, :amount, :title, :description, :location])
  end
end
