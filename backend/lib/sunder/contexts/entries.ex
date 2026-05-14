defmodule Sunder.Contexts.Entries do
  import Ecto.Query

  alias Sunder.Repo
  alias Sunder.Eco.{Entry}

  def list_entries(user_id) do
    Repo.all(from(e in Entry, where: e.user_id == ^user_id))
  end
end
