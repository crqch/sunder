defmodule Sunder.Eco.Category do
  alias Sunder.Eco.EcoUser
  use Sunder.Schema
  import Ecto.Changeset

  @derive {Jason.Encoder,
           only: [:id, :title, :description, :color, :deleted_at, :inserted_at, :updated_at]}

  schema "entry_categories" do
    field(:description, :string)
    field(:title, :string)
    field(:color, :string)

    belongs_to(:eco_user, EcoUser)

    field(:deleted_at, :utc_datetime)
    timestamps()
  end

  @doc false
  def changeset(category, attrs) do
    category
    |> cast(attrs, [:id, :title, :description, :color, :deleted_at, :updated_at])
    |> validate_required([:title, :description, :color])
  end
end
