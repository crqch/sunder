defmodule SunderWeb.Authed.DashboardController do
  import SunderWeb.ApiMacros
  use SunderWeb, :controller

  alias Sunder.Contexts.Currencies
  alias Sunder.Repo

  authed_operation(:index,
    summary: "Get user",
    responses: [
      ok: {
        %{
          type: :object,
          properties: %{
            id: %{type: :string},
            username: %{type: :string},
            email: %{type: :string},
            flags: %{type: :array, items: %{type: :string}},
            default_currency_code: %{type: :string}
          }
        },
        [description: "Current user info and default display currency"]
      },
      bad_request: {
        %{
          type: :object,
          properties: %{
            error_code: %{type: :string},
            message: %{type: :string}
          }
        },
        [description: "Error"]
      }
    ]
  )

  def index(%{assigns: %{user: user}} = conn, _params) do
    user = Repo.preload(user, :eco_user)

    default_currency_code =
      (user.eco_user && user.eco_user.default_currency_code) || Currencies.canonical_base()

    conn
    |> json(%{
      id: user.id,
      username: user.username,
      email: user.email,
      flags: user.flags,
      default_currency_code: default_currency_code
    })
  end
end
