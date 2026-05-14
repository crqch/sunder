defmodule Sunder.Accounts.User do
  use Sunder.Schema
  import Ecto.Changeset

  @derive {Jason.Encoder, only: [:username, :email]}

  schema "users" do
    field(:flags, {:array, :string})
    field(:pass, :string)
    field(:username, :string)
    field(:email, :string)

    has_many(:access_tokens, Sunder.Accounts.AccessToken)
    has_many(:refresh_tokens, Sunder.Accounts.RefreshToken)

    has_one(:eco_user, Sunder.Eco.EcoUser)

    timestamps()
  end

  @doc false
  def changeset(user, attrs) do
    user
    |> cast(attrs, [:email, :username, :pass, :flags])
    |> validate_required([:email, :username, :pass, :flags])
    |> put_password_hash()
  end

  @doc false
  def registration_changeset(user, attrs) do
    user
    |> cast(attrs, [:email, :username, :pass])
    |> validate_required([:email, :username, :pass])
    |> put_password_hash()
  end

  defp put_password_hash(%Ecto.Changeset{valid?: true, changes: %{pass: pass}} = changeset) do
    put_change(changeset, :pass, Argon2.hash_pwd_salt(pass))
  end

  defp put_password_hash(changeset), do: changeset
end
