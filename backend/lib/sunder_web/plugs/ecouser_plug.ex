defmodule SunderWeb.Plugs.EcoUserPlug do
  import Ecto.Query
  import Plug.Conn
  alias Sunder.Eco.EcoUser
  use SunderWeb, :controller
  alias Sunder.Repo

  def init(opts), do: opts

  def call(%{assigns: %{user: user}} = conn, _opts) do
    case Repo.one(
           from(
             e in EcoUser,
             where: e.user_id == ^user.id
           )
         ) do
      nil ->
        conn
        |> put_status(500)
        |> json(%{
          code: "eco_user_not_found",
          message: "EcoUser associated with user not found! This should not happen. Report it."
        })
        |> halt()

      eco_user ->
        conn |> assign(:eco_user, eco_user)
    end
  end
end
