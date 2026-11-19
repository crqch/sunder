defmodule Sunder.Eco.ExchangeRate do
  use Sunder.Schema
  import Ecto.Changeset

  alias Sunder.Eco.Currency

  @derive {Jason.Encoder,
           only: [
             :id,
             :base_currency_code,
             :target_currency_code,
             :rate,
             :rate_date,
             :source,
             :inserted_at,
             :updated_at
           ]}

  schema "exchange_rates" do
    field(:base_currency_code, :string, default: "USD")
    field(:target_currency_code, :string)
    field(:rate, :decimal)
    field(:rate_date, :date)
    field(:source, :string, default: "unknown")

    belongs_to(:base_currency, Currency,
      foreign_key: :base_currency_code,
      references: :code,
      define_field: false
    )

    belongs_to(:target_currency, Currency,
      foreign_key: :target_currency_code,
      references: :code,
      define_field: false
    )

    timestamps()
  end

  @doc false
  def changeset(exchange_rate, attrs) do
    exchange_rate
    |> cast(attrs, [:base_currency_code, :target_currency_code, :rate, :rate_date, :source])
    |> normalize_code(:base_currency_code)
    |> normalize_code(:target_currency_code)
    |> validate_required([:base_currency_code, :target_currency_code, :rate, :rate_date, :source])
    |> validate_number(:rate, greater_than: 0)
    |> foreign_key_constraint(:base_currency_code)
    |> foreign_key_constraint(:target_currency_code)
    |> unique_constraint([:base_currency_code, :target_currency_code, :rate_date])
  end

  defp normalize_code(changeset, field) do
    update_change(changeset, field, fn code ->
      code |> String.trim() |> String.upcase()
    end)
  end
end
