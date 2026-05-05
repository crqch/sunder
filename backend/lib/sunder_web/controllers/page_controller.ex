defmodule SunderWeb.StatusController do
  use SunderWeb, :controller

  operation(:index,
    summary: "API Status",
    responses: [
      ok: {
        %{
          type: :object,
          properties: %{version: %{type: :string}}
        },
        [description: "Status"]
      }
    ]
  )

  def index(conn, _params) do
    json(conn, %{version: "1.0.0"})
  end
end
