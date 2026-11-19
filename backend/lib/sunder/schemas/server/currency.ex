defmodule Sunder.Eco.Currency do
  use Ecto.Schema
  import Ecto.Changeset

  @primary_key {:code, :string, autogenerate: false}
  @foreign_key_type :string

  @derive {Jason.Encoder,
           only: [
             :code,
             :name,
             :symbol,
             :symbol_after,
             :decimal_digits,
             :is_active,
             :inserted_at,
             :updated_at
           ]}

  schema "currencies" do
    field(:name, :string)
    field(:symbol, :string)
    field(:symbol_after, :boolean, default: false)
    field(:decimal_digits, :integer, default: 2)
    field(:is_active, :boolean, default: true)

    timestamps()
  end

  @doc false
  def changeset(currency, attrs) do
    currency
    |> cast(attrs, [:code, :name, :symbol, :symbol_after, :decimal_digits, :is_active])
    |> normalize_code()
    |> validate_required([:code, :name, :symbol, :symbol_after, :decimal_digits, :is_active])
    |> validate_format(:code, ~r/^[A-Z0-9]{2,10}$/,
      message: "must be 2-10 uppercase alphanumeric characters"
    )
    |> validate_number(:decimal_digits, greater_than_or_equal_to: 0, less_than_or_equal_to: 8)
    |> unique_constraint(:code, name: :currencies_pkey)
  end

  @doc false
  def update_changeset(currency, attrs) do
    currency
    |> cast(attrs, [:name, :symbol, :symbol_after, :decimal_digits, :is_active])
    |> validate_required([:name, :symbol, :symbol_after, :decimal_digits, :is_active])
    |> validate_number(:decimal_digits, greater_than_or_equal_to: 0, less_than_or_equal_to: 8)
  end

  defp normalize_code(changeset) do
    update_change(changeset, :code, fn code ->
      code |> String.trim() |> String.upcase()
    end)
  end
end
