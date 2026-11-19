defmodule Sunder.Eco.Account do
  use Sunder.Schema
  import Ecto.Changeset

  alias Sunder.Eco.Currency

  @derive {Jason.Encoder,
           only: [:id, :eco_user_id, :name, :currency_code, :deleted_at, :inserted_at, :updated_at]}

  schema "accounts" do
    field(:name, :string)
    field(:currency_code, :string)

    belongs_to(:eco_user, Sunder.Eco.EcoUser)

    belongs_to(:currency, Currency,
      foreign_key: :currency_code,
      references: :code,
      define_field: false
    )

    has_many(:entries, Sunder.Eco.Entry)

    field(:deleted_at, :utc_datetime)
    timestamps()
  end

  @doc false
  def changeset(account, attrs) do
    account
    |> cast(attrs, [:id, :eco_user_id, :name, :currency_code, :deleted_at, :updated_at])
    |> normalize_currency_code()
    |> validate_required([:eco_user_id, :name])
    |> foreign_key_constraint(:currency_code)
  end

  @doc false
  def update_changeset(account, attrs) do
    account
    |> cast(attrs, [:eco_user_id])
    |> validate_required([:eco_user_id])
  end

  defp normalize_currency_code(changeset) do
    update_change(changeset, :currency_code, fn
      nil ->
        nil

      code ->
        case String.trim(code) do
          "" -> nil
          trimmed -> String.upcase(trimmed)
        end
    end)
  end
end
