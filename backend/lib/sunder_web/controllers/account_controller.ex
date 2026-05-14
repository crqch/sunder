defmodule SunderWeb.Authed.AccountController do
  use SunderWeb, :controller
  alias SunderWeb.Contexts.Accounts

  # ✅ GET     /dashboard/accounts                    SunderWeb.Authed.AccountController :index
  def index(%{assigns: %{eco_user: eco_user}} = conn, _params) do
    conn
    |> json(%{data: Accounts.accounts_of_user(eco_user)})
  end

  # ⏭️(a page for form) GET     /dashboard/accounts/:id/edit           SunderWeb.Authed.AccountController :edit
  # ⏭️(a page for form) GET     /dashboard/accounts/new                SunderWeb.Authed.AccountController :new

  # ✅ GET     /dashboard/accounts/:id                SunderWeb.Authed.AccountController :show

  def show(%{assigns: %{eco_user: eco_user}} = conn, %{id: id}) do
    case Accounts.account(eco_user, id) do
      nil -> conn |> put_status(404) |> json(%{message: "Account not found", code: "not_found"})
      account -> conn |> json(%{account: account, code: "ok"})
    end
  end

  # ✅ POST    /dashboard/accounts                    SunderWeb.Authed.AccountController :create

  def create(%{assigns: %{eco_user: eco_user}} = conn, params) do
    case Accounts.create(eco_user, params) do
      {:ok, account} ->
        conn
        |> json(%{
          message: "New account created",
          account: account,
          code: "ok"
        })

      {:error, :changeset, _} ->
        conn
        |> put_status(400)
        |> json(%{
          message: "Missing or invalid data",
          code: "changeset"
        })

      {:error, _} ->
        conn
        |> put_status(400)
        |> json(%{message: "There was an error creating the account", code: "other"})
    end
  end

  # ✅ PATCH   /dashboard/accounts/:id                SunderWeb.Authed.AccountController :update
  # ✅ PUT     /dashboard/accounts/:id                SunderWeb.Authed.AccountController :update

  def update(%{assigns: %{eco_user: eco_user}} = conn, params) do
    case Accounts.update(eco_user, params) do
      {:ok, account} ->
        conn
        |> json(%{
          message: "Account updated",
          account: account,
          code: "ok"
        })

      {:error, :changeset, _} ->
        conn
        |> put_status(400)
        |> json(%{
          message: "Missing or invalid data",
          code: "changeset"
        })

      {:error, _} ->
        conn
        |> put_status(400)
        |> json(%{message: "There was an error updating the account", code: "other"})
    end
  end

  # ✅ DELETE  /dashboard/accounts/:id                SunderWeb.Authed.AccountController :delete

  def delete(%{assigns: %{eco_user: eco_user}} = conn, %{id: id}) do
    case Accounts.delete(eco_user, id) do
      {:ok} -> conn |> json(%{code: "ok", message: "Account deleted"})
      {:error} -> conn |> put_status(400) |> json(%{code: "error"})
    end
  end
end
