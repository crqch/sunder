defmodule Sunder.Eco.Category do
  alias Sunder.Eco.EcoUser
  use Sunder.Schema
  import Ecto.Changeset

  @derive {Jason.Encoder, only: [:title, :description, :color]}

  schema "entry_categories" do
    field(:description, :string)
    field(:title, :string)
    field(:color, :string)

    belongs_to(:eco_user, EcoUser)

    timestamps()
  end

  @doc false
  def changeset(category, attrs) do
    category
    |> cast(attrs, [:title, :description, :color])
    |> validate_required([:title, :description, :color])
  end
end
