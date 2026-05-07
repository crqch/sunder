defmodule Sunder.Contexts.Users do
  import Ecto.Query

  alias Sunder.Repo
  alias Sunder.Accounts.{User, Invite, AccessToken, RefreshToken}

  def register_with_invite(params) do
    Ecto.Multi.new()
    |> Ecto.Multi.run(:email_not_used, fn repo, _changes ->
      case repo.get_by(User, email: params["email"]) do
        nil -> {:ok, nil}
        _ -> {:error, :email_already_used}
      end
    end)
    |> Ecto.Multi.run(:invite, fn repo, _changes ->
      case repo.get_by(Invite, token: params["invite"]) do
        nil -> {:error, :invalid_invite}
        %{used: true} -> {:error, :invite_already_used}
        invite -> {:ok, invite}
      end
    end)
    |> Ecto.Multi.insert(:user, fn _changes ->
      User.registration_changeset(%User{}, params)
    end)
    |> Ecto.Multi.update(:mark_invite_used, fn %{invite: invite} ->
      Ecto.Changeset.change(invite, used: true)
    end)
    |> Repo.transaction()
  end

  def login(params) do
    Ecto.Multi.new()
    |> Ecto.Multi.run(:user, fn repo, _changes ->
      case repo.get_by(User, email: params["email"]) do
        nil -> {:error, :invalid_email}
        user -> {:ok, user}
      end
    end)
    |> Ecto.Multi.run(:pass_matching, fn _repo, %{user: user} ->
      plain_text_password = params["pass"]

      cond do
        is_nil(plain_text_password) ->
          {:error, :missing_password}

        is_nil(user) || is_nil(user.pass) ->
          Argon2.no_user_verify()
          {:error, :pass_not_matching}

        Argon2.verify_pass(plain_text_password, user.pass) ->
          {:ok, nil}

        true ->
          {:error, :pass_not_matching}
      end
    end)
    |> Ecto.Multi.run(:refresh_token, fn repo, %{user: user} ->
      query = from(r in RefreshToken, where: r.user_id == ^user.id, limit: 1)

      case repo.one(query) do
        %RefreshToken{} = existing_token ->
          {:ok, existing_token}

        nil ->
          %RefreshToken{}
          |> RefreshToken.changeset(%{
            user_id: user.id,
            token: Sunder.Tokens.generate_token(),
            expires_at: Timex.shift(DateTime.utc_now(), days: 30)
          })
          |> repo.insert()
      end
    end)
    |> Repo.transaction()
  end

  def refresh_token(params) do
    Ecto.Multi.new()
    |> Ecto.Multi.run(:find_refresh_token, fn repo, _changes ->
      case repo.get_by(RefreshToken, token: params["token"]) do
        nil ->
          {:error, :invalid_token}

        token ->
          if token.expires_at |> Timex.before?(DateTime.utc_now()),
            do: {:error, :token_expired},
            else: {:ok, token}
      end
    end)
    |> Ecto.Multi.run(:user, fn repo, %{find_refresh_token: token} ->
      case repo.get(User, token.user_id) do
        nil ->
          {:error, :user_not_found}

        user ->
          {:ok, user}
      end
    end)
    |> Ecto.Multi.insert(:access_token, fn %{user: user} ->
      %AccessToken{}
      |> AccessToken.changeset(%{
        user_id: user.id,
        token: Sunder.Tokens.generate_token(),
        expires_at: Timex.shift(DateTime.utc_now(), hours: 1)
      })
    end)
    |> Ecto.Multi.update(:extend_token, fn %{find_refresh_token: token} ->
      RefreshToken.changeset(token, %{expires_at: Timex.shift(DateTime.utc_now(), days: 30)})
    end)
    |> Repo.transaction()
  end
end
