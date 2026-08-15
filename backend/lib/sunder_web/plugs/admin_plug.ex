defmodule SunderWeb.Plugs.AdminPlug do
  import Plug.Conn
  use SunderWeb, :controller

  def init(opts), do: opts

  def call(conn, _opts) do
    user = conn.assigns[:user]

    if user && user.flags && "is_admin" in user.flags do
      conn
    else
      conn
      |> put_status(403)
      |> json(%{error: "Forbidden"})
      |> halt()
    end
  end
end
