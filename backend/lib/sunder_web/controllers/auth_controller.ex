defmodule SunderWeb.AuthController do
  use SunderWeb, :controller
  alias Sunder.Contexts.Users

  operation(:register,
    summary: "Register an account",
    parameters: [
      invite: [
        in: :path,
        schema: %{type: :string},
        description: "Invite code"
      ]
    ],
    request_body: {
      %{
        type: :object,
        properties: %{
          email: %{type: :string, description: "Email address"},
          username: %{type: :string, description: "Username"},
          pass: %{type: :string, description: "Password"}
        }
      },
      []
    },
    responses: [
      created: {
        %{
          type: :object,
          properties: %{
            id: %{type: :string},
            message: %{type: :string}
          }
        },
        [description: "User id"]
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

  def register(conn, params) do
    case Users.register_with_invite(params) do
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

  operation(:login,
    summary: "Login",
    request_body: {
      %{
        type: :object,
        properties: %{
          login: %{type: :string, description: "Email address or username"},
          pass: %{type: :string, description: "Password"}
        }
      },
      []
    },
    responses: [
      ok: {
        %{
          type: :object,
          properties: %{
            id: %{type: :string},
            message: %{type: :string},
            refresh_token: %{type: :string}
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

  def login(conn, params) do
    case Users.login(params) do
      {:ok, %{user: user, refresh_token: refresh_token}} ->
        conn
        |> put_status(:ok)
        |> json(%{message: "Login successful!", id: user.id, refresh_token: refresh_token.token})

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

  operation(:refresh_token,
    summary: "Refresh token",
    request_body: {
      %{
        type: :object,
        properties: %{
          token: %{type: :string, description: "Refresh token"}
        }
      },
      []
    },
    responses: [
      ok: {
        %{
          type: :object,
          properties: %{
            message: %{type: :string},
            id: %{type: :string},
            access_token: %{type: :string}
          }
        },
        [description: "Token refreshed"]
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

  def refresh_token(conn, params) do
    case Users.refresh_token(params) do
      {:ok, %{user: user, access_token: access_token}} ->
        conn
        |> put_status(:ok)
        |> json(%{
          message: "Token refreshed successfully!",
          id: user.id,
          access_token: access_token.token
        })

      {:error, :user, %Ecto.Changeset{} = changeset, _changes} ->
        conn
        |> put_status(:bad_request)
        |> json(%{
          error_code: "VALIDATION_FAILED",
          errors: Ecto.Changeset.traverse_errors(changeset, fn {msg, _opts} -> msg end)
        })

      {:error, :find_refresh_token, :token_expired, _token} ->
        conn
        |> put_status(:bad_request)
        |> json(%{
          error_code: "REFRESH_TOKEN_EXPIRED",
          message: "Refresh token has expired. Log in again"
        })

      {:error, :user, _changes} ->
        conn
        |> put_status(:bad_request)
        |> json(%{error_code: "INVALID_CREDENTIALS", message: "Invalid email or password."})
    end
  end
end
