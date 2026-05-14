defmodule SunderWeb.Sync.Writer do
  alias Sunder.Eco.{Account, Category, Entry}

  def apply_writes(multi, %{eco_user: eco_user}, last_synced_at) do
    Ecto.Multi.run(multi, :upsert_sync, fn repo, %{guard: guard_data} ->
      with {:ok, acc_res} <-
             process_domain(repo, guard_data.accounts, last_synced_at, Account, eco_user.id),
           {:ok, cat_res} <-
             process_domain(repo, guard_data.categories, last_synced_at, Category, eco_user.id),
           {:ok, ent_res} <-
             process_domain(repo, guard_data.entries, last_synced_at, Entry, eco_user.id) do
        {:ok, %{accounts: acc_res, categories: cat_res, entries: ent_res}}
      end
    end)
  end

  defp process_domain(repo, domain_data, last_synced_at, schema_module, eco_user_id) do
    %{valid: valid_txs, db_records: db_records} = domain_data

    Enum.reduce_while(valid_txs, {:ok, %{conflicts: [], ok: []}}, fn client_tx,
                                                                     {:ok,
                                                                      %{
                                                                        conflicts: conflicts,
                                                                        ok: ok
                                                                      }} ->
      db_record = Map.get(db_records, client_tx["id"])

      result =
        cond do
          db_record && updated_after_sync?(db_record.updated_at, last_synced_at) ->
            {:conflict,
             SunderWeb.Sync.Conflict.resolve(schema_module, db_record, client_tx)
             |> repo.update()}

          true ->
            (db_record || struct(schema_module))
            |> schema_module.changeset(Map.put(client_tx, "eco_user_id", eco_user_id))
            |> repo.insert_or_update()
        end

      case result do
        {:conflict, {:ok, saved_record}} ->
          {:cont, {:ok, %{conflicts: [saved_record | conflicts], ok: ok}}}

        {:ok, saved_record} ->
          {:cont, {:ok, %{conflicts: conflicts, ok: [saved_record | ok]}}}

        {:error, changeset} ->
          {:halt, {:error, changeset}}
      end
    end)
  end

  defp updated_after_sync?(_db_updated_at, nil), do: true

  defp updated_after_sync?(db_updated_at, last_synced_at) do
    NaiveDateTime.compare(db_updated_at, last_synced_at) == :gt
  end
end
