defmodule SunderWeb.AdminController do
  use SunderWeb, :controller
  import Ecto.Query
  alias Sunder.Repo
  alias Sunder.Accounts.{User, Invite}

  def list_users(conn, _params) do
    users = Repo.all(from u in User, select: %{id: u.id, username: u.username, email: u.email, flags: u.flags})
    json(conn, %{users: users})
  end

  def delete_user(conn, %{"id" => id}) do
    case Repo.get(User, id) do
      nil ->
        conn |> put_status(404) |> json(%{error: "Not found"})
      user ->
        Repo.delete!(user)
        json(conn, %{success: true})
    end
  end

  def update_user(conn, %{"id" => id} = params) do
    user_params = if params["pass"] in [nil, ""], do: Map.delete(params, "pass"), else: params

    case Repo.get(User, id) do
      nil ->
        conn |> put_status(404) |> json(%{error: "Not found"})
      user ->
        user
        |> User.changeset(user_params)
        |> Repo.update()
        |> case do
          {:ok, updated_user} -> 
             json(conn, %{success: true, user: %{id: updated_user.id, username: updated_user.username, email: updated_user.email, flags: updated_user.flags}})
          {:error, _changeset} -> 
             conn |> put_status(400) |> json(%{error: "Invalid input"})
        end
    end
  end

  def list_invites(conn, _params) do
    invites = Repo.all(from i in Invite, select: %{id: i.id, token: i.token, used_by: i.used_by})
    json(conn, %{invites: invites})
  end

  def create_invite(conn, %{"token" => token}) do
    %Invite{}
    |> Invite.changeset(%{token: token})
    |> Repo.insert()
    |> case do
      {:ok, invite} -> json(conn, %{id: invite.id, token: invite.token, used_by: invite.used_by})
      {:error, _} -> conn |> put_status(400) |> json(%{error: "Invalid token"})
    end
  end

  def delete_invite(conn, %{"id" => id}) do
    case Repo.get(Invite, id) do
      nil ->
        conn |> put_status(404) |> json(%{error: "Not found"})
      invite ->
        Repo.delete!(invite)
        json(conn, %{success: true})
    end
  end
end
