defmodule SunderWeb.Authed.PreferencesController do
  import SunderWeb.ApiMacros
  use SunderWeb, :controller

  alias Sunder.Contexts.Currencies

  @currency_schema %{
    type: :object,
    properties: %{
      code: %{type: :string},
      name: %{type: :string},
      symbol: %{type: :string},
      symbol_after: %{type: :boolean},
      decimal_digits: %{type: :integer}
    }
  }

  @preferences_response %{
    type: :object,
    properties: %{
      default_currency_code: %{type: :string},
      default_currency: @currency_schema
    }
  }

  authed_operation(:show,
    summary: "Get user financial preferences",
    responses: [
      ok: {@preferences_response, [description: "User preferences including default display currency"]}
    ]
  )

  def show(%{assigns: %{eco_user: eco_user}} = conn, _params) do
    currency = Currencies.get_user_default_currency(eco_user)

    json(conn, %{
      default_currency_code: eco_user.default_currency_code,
      default_currency: serialize_currency(currency)
    })
  end

  authed_operation(:update,
    summary: "Update user financial preferences (e.g. default display currency)",
    request_body: {
      %{
        type: :object,
        properties: %{
          default_currency_code: %{
            type: :string,
            description: "ISO 4217 currency code from GET /currencies"
          }
        },
        required: [:default_currency_code]
      },
      [required: true, description: "Preferences to update"]
    },
    responses: [
      ok: {@preferences_response, [description: "Updated user preferences"]},
      bad_request: {
        %{type: :object, properties: %{error: %{type: :string}}},
        [description: "Invalid or disallowed currency"]
      }
    ]
  )

  def update(%{assigns: %{eco_user: eco_user}} = conn, %{
        "default_currency_code" => currency_code
      }) do
    case Currencies.set_user_default_currency(eco_user, currency_code) do
      {:ok, %{eco_user: updated, currency: currency}} ->
        json(conn, %{
          default_currency_code: updated.default_currency_code,
          default_currency: serialize_currency(currency)
        })

      {:error, :currency_not_allowed} ->
        conn |> put_status(:bad_request) |> json(%{error: "currency_not_allowed"})

      {:error, _changeset} ->
        conn |> put_status(:bad_request) |> json(%{error: "invalid_preferences"})
    end
  end

  def update(conn, _params) do
    conn |> put_status(:bad_request) |> json(%{error: "missing_default_currency_code"})
  end

  defp serialize_currency(nil), do: nil

  defp serialize_currency(c) do
    %{
      code: c.code,
      name: c.name,
      symbol: c.symbol,
      symbol_after: c.symbol_after,
      decimal_digits: c.decimal_digits
    }
  end
end
