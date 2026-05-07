defmodule SunderWeb.Plugs.AuthPlug do
  import Ecto.Query
  import Plug.Conn
  use SunderWeb, :controller
  alias Sunder.Accounts.{User, AccessToken}
  alias Sunder.Repo

  def init(opts), do: opts

  def call(conn, _opts) do
    conn = fetch_cookies(conn)

    with {:ok, token} <- Map.fetch(conn.cookies, "authorization"),
          _ <- IO.inspect(token, label: "COOKIE TOKEN"),
          %User{} = user <- Repo.one(
            from(u in User,
              join: ac in AccessToken,
              on: ac.user_id == u.id,
              where: ac.token == ^token,
              select: u
            )
          ),
          _ <- IO.inspect(user.id, label: "FOUND USER") do
       assign(conn, :user, user)
     else
       err ->
         IO.inspect(err, label: "AUTH FAILURE REASON")
         conn
         |> put_status(401)
         |> json(%{message: "Missing/invalid cookie!"})
         |> halt()
     end
  end
end
