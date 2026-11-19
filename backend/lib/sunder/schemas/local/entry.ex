defmodule Sunder.Eco.Entry do
  use Sunder.Schema
  import Ecto.Changeset

  alias Sunder.Eco.Currency

  @derive {Jason.Encoder,
           only: [
             :id,
             :date,
             :amount,
             :currency_code,
             :title,
             :description,
             :location,
             :account_id,
             :category_id,
             :eco_user_id,
             :deleted_at,
             :inserted_at,
             :updated_at
           ]}

  schema "account_entries" do
    field(:date, :naive_datetime)
    field(:description, :string)
    field(:title, :string)
    field(:location, :string)
    field(:amount, :float)
    field(:currency_code, :string, default: "USD")

    belongs_to(:account, Sunder.Eco.Account)
    belongs_to(:category, Sunder.Eco.Category)
    belongs_to(:eco_user, Sunder.Eco.EcoUser)

    belongs_to(:currency, Currency,
      foreign_key: :currency_code,
      references: :code,
      define_field: false
    )

    field(:deleted_at, :utc_datetime)
    timestamps()
  end

  @doc false
  def changeset(entry, attrs) do
    entry
    |> cast(attrs, [
      :id,
      :date,
      :amount,
      :currency_code,
      :title,
      :description,
      :location,
      :account_id,
      :category_id,
      :eco_user_id,
      :deleted_at,
      :updated_at
    ])
    |> normalize_currency_code()
    |> validate_required([
      :date,
      :amount,
      :currency_code,
      :title,
      :account_id,
      :category_id,
      :eco_user_id
    ])
    |> foreign_key_constraint(:currency_code)
  end

  defp normalize_currency_code(changeset) do
    update_change(changeset, :currency_code, fn code ->
      code |> String.trim() |> String.upcase()
    end)
  end
end
