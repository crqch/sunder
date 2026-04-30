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
    |> cast(attrs, [:token, :used_by])
    |> validate_required([:token, :used_by])
  end
end
