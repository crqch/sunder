defmodule SunderWeb.Sync.Reader do
  import Ecto.Query

  require Logger
  alias Sunder.Eco.{Account, Category, Entry}

  def fetch_updates(multi, %{eco_user: eco_user}, timestamp, txs) do
    Ecto.Multi.run(multi, :reader, fn repo, %{upsert_sync: upsert_data} ->
      {:ok,
       %{
         accounts:
           process_domain(
             repo,
             eco_user,
             Map.get(txs, "accounts", []),
             timestamp,
             Account,
             upsert_data.accounts.conflicts
           ),
         categories:
           process_domain(
             repo,
             eco_user,
             Map.get(txs, "categories", []),
             timestamp,
             Category,
             upsert_data.categories.conflicts
           ),
         entries:
           process_domain(
             repo,
             eco_user,
             Map.get(txs, "entries", []),
             timestamp,
             Entry,
             upsert_data.entries.conflicts
           )
       }}
    end)
  end

  defp process_domain(repo, eco_user, txs, timestamp, schema_module, conflicts) do
    ids = Enum.map(txs, fn tx -> tx["id"] end)

    query =
      from(s in schema_module,
        where: s.id not in ^ids and s.eco_user_id == ^eco_user.id
      )

    query =
      if timestamp do
        from(s in query, where: s.updated_at > ^timestamp)
      else
        query
      end

    entries = repo.all(query)

    entries ++ conflicts
  end
end
