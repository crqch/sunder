defmodule SunderWeb.StatusController do
  use SunderWeb, :controller

  def index(conn, _params) do
    json(conn, %{version: "1.0.0"})
  end
end
