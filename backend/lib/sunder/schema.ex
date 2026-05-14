defmodule Sunder.Schema do
  defmacro __using__(_) do
    quote do
      use Ecto.Schema
      @primary_key {:id, :string, autogenerate: {ExCuid2, :generate, []}}
      @foreign_key_type :string
      import Sunder.Schema, only: [local_schema: 2]
    end
  end

  defmacro local_schema(source, do: block) do
    quote do
      schema unquote(source) do
        unquote(block)

        field(:deleted_at, :utc_datetime)
        timestamps()
      end
    end
  end
end
