defmodule SunderWeb.Authed.DashboardController do
  import SunderWeb.ApiMacros
  use SunderWeb, :controller

  authed_operation(:index,
    summary: "Get user",
    responses: [
      ok: {
        %{
          type: :object,
          properties: %{
            id: %{type: :string},
            username: %{type: :string}
          }
        },
        [description: "User id and refresh token"]
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
    conn |> json(%{id: user.id, username: user.username})
  end
end
