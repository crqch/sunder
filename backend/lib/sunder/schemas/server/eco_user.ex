defmodule Sunder.Eco.EcoUser do
  use Sunder.Schema
  import Ecto.Changeset

  alias Sunder.Eco.Currency

  schema "eco_users" do
    field(:default_currency_code, :string, default: "USD")

    has_many(:entries, Sunder.Eco.Entry)
    has_many(:accounts, Sunder.Eco.Account)
    has_many(:categories, Sunder.Eco.Category)

    belongs_to(:user, Sunder.Accounts.User)

    belongs_to(:default_currency, Currency,
      foreign_key: :default_currency_code,
      references: :code,
      define_field: false
    )

    timestamps()
  end

  @doc false
  def changeset(eco_user, attrs) do
    eco_user
    |> cast(attrs, [:default_currency_code])
    |> normalize_currency_code()
    |> validate_required([:user_id, :default_currency_code])
    |> foreign_key_constraint(:default_currency_code)
  end

  @doc false
  def preferences_changeset(eco_user, attrs) do
    eco_user
    |> cast(attrs, [:default_currency_code])
    |> normalize_currency_code()
    |> validate_required([:default_currency_code])
    |> foreign_key_constraint(:default_currency_code)
  end

  defp normalize_currency_code(changeset) do
    update_change(changeset, :default_currency_code, fn
      code when is_binary(code) -> code |> String.trim() |> String.upcase()
      other -> other
    end)
  end
end
