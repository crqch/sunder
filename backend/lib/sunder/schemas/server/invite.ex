defmodule Sunder.Accounts.Invite do
  use Sunder.Schema
  import Ecto.Changeset

  @derive {Jason.Encoder, only: [:token]}

  schema "invites" do
    field(:token, :string)
    field(:used_by, :string)

    timestamps()
  end

  @doc false
  def changeset(invite, attrs) do
    invite
    |> cast(attrs, [:token])
    |> validate_required([:token])
  end
end
