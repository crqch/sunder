defmodule Sunder.Entry.Category do
  use Sunder.Schema
  import Ecto.Changeset

  schema "entry_categories" do
    field(:description, :string)
    field(:title, :string)
    field(:color, :string)

    timestamps()
  end

  @doc false
  def changeset(category, attrs) do
    category
    |> cast(attrs, [:title, :description, :color])
    |> validate_required([:title, :description, :color])
  end
end
