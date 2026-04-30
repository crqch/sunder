defmodule SunderWeb.AuthController do
  use SunderWeb, :controller
  alias Sunder.Contexts.Accounts

  def register(conn, params) do
    case Accounts.register_with_invite(params) do
      {:ok, %{user: user}} ->
        conn
        |> put_status(:created)
        |> json(%{message: "User registered succesfully!", id: user.id})

      {:error, :email_not_used, _changes} ->
        conn
        |> put_status(:bad_request)
        |> json(%{error_code: "INVALID_INVITE", message: "That invite code does not exist!"})

      {:error, :invite, :invalid_invite, _changes} ->
        conn
        |> put_status(:bad_request)
        |> json(%{error_code: "INVALID_INVITE", message: "That invite code does not exist!"})

      {:error, :invite, :invite_already_used, _changes} ->
        conn
        |> put_status(:bad_request)
        |> json(%{error_code: "USED_INVITE", message: "That invite code has already been used."})

      {:error, :user, %Ecto.Changeset{} = changeset, _changes} ->
        conn
        |> put_status(:bad_request)
        |> json(%{
          error_code: "VALIDATION_FAILED",
          errors: Ecto.Changeset.traverse_errors(changeset, fn {msg, _opts} -> msg end)
        })
    end
  end

  def login(conn, params) do
    case Accounts.login(params) do
      {:ok, %{user: user, refresh_token: refresh_token}} ->
        conn
        |> put_status(:ok)
        |> json(%{message: "Login successful!", id: user.id, refresh_token: refresh_token})

      {:error, :user, %Ecto.Changeset{} = changeset, _changes} ->
        conn
        |> put_status(:bad_request)
        |> json(%{
          error_code: "VALIDATION_FAILED",
          errors: Ecto.Changeset.traverse_errors(changeset, fn {msg, _opts} -> msg end)
        })

      {:error, :user, _reason, _changes} ->
        conn
        |> put_status(:bad_request)
        |> json(%{error_code: "INVALID_CREDENTIALS", message: "Invalid email or password."})

      {:error, :pass_matching, :missing_password, _changes} ->
        conn
        |> put_status(:bad_request)
        |> json(%{error_code: "MISSING_PASSWORD", message: "Password is required."})

      {:error, :pass_matching, _reason, _changes} ->
        conn
        |> put_status(:bad_request)
        |> json(%{error_code: "INVALID_CREDENTIALS", message: "Invalid email or password."})
    end
  end

  def refresh_token(conn, params) do
    case Accounts.refresh_token(params) do
      {:ok, %{user: user}} ->
        conn
        |> put_status(:ok)
        |> json(%{message: "Token refreshed successfully!", id: user.id})

      {:error, :user, %Ecto.Changeset{} = changeset, _changes} ->
        conn
        |> put_status(:bad_request)
        |> json(%{
          error_code: "VALIDATION_FAILED",
          errors: Ecto.Changeset.traverse_errors(changeset, fn {msg, _opts} -> msg end)
        })

      {:error, :user, _changes} ->
        conn
        |> put_status(:bad_request)
        |> json(%{error_code: "INVALID_CREDENTIALS", message: "Invalid email or password."})
    end
  end
end
