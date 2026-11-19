defmodule SunderWeb.Sync.Writer do
  alias Sunder.Contexts.Currencies
  alias Sunder.Eco.{Account, Category, Entry}

  def apply_writes(multi, %{eco_user: eco_user}, last_synced_at) do
    Ecto.Multi.run(multi, :upsert_sync, fn repo, %{guard: guard_data} ->
      known_codes = Currencies.known_currency_codes()

      with {:ok, acc_res} <-
             process_domain(repo, guard_data.accounts, last_synced_at, Account, eco_user, known_codes),
           {:ok, cat_res} <-
             process_domain(repo, guard_data.categories, last_synced_at, Category, eco_user, known_codes),
           {:ok, ent_res} <-
             process_domain(repo, guard_data.entries, last_synced_at, Entry, eco_user, known_codes) do
        {:ok, %{accounts: acc_res, categories: cat_res, entries: ent_res}}
      end
    end)
  end

  defp process_domain(repo, domain_data, last_synced_at, schema_module, eco_user, known_codes) do
    %{valid: valid_txs, db_records: db_records} = domain_data

    Enum.reduce_while(valid_txs, {:ok, %{conflicts: [], ok: []}}, fn client_tx,
                                                                     {:ok,
                                                                      %{
                                                                        conflicts: conflicts,
                                                                        ok: ok
                                                                      }} ->
      db_record = Map.get(db_records, client_tx["id"])

      case prepare_tx(schema_module, client_tx, db_record, eco_user, known_codes) do
        {:error, reason} ->
          {:halt, {:error, reason}}

        {:ok, prepared_tx} ->
          result =
            cond do
              db_record && updated_after_sync?(db_record.updated_at, last_synced_at) ->
                {:conflict,
                 SunderWeb.Sync.Conflict.resolve(schema_module, db_record, prepared_tx)
                 |> repo.update()}

              true ->
                (db_record || struct(schema_module))
                |> schema_module.changeset(Map.put(prepared_tx, "eco_user_id", eco_user.id))
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
      end
    end)
  end

  defp prepare_tx(Entry, client_tx, db_record, eco_user, known_codes) do
    raw_code = Map.get(client_tx, "currency_code")

    case Currencies.normalize_code(raw_code) do
      nil ->
        if db_record do
          {:ok, Map.delete(client_tx, "currency_code")}
        else
          default_code = eco_user.default_currency_code || Currencies.canonical_base()
          {:ok, Map.put(client_tx, "currency_code", default_code)}
        end

      normalized_code ->
        same_as_existing = db_record != nil and db_record.currency_code == normalized_code

        if same_as_existing or MapSet.member?(known_codes, normalized_code) do
          {:ok, Map.put(client_tx, "currency_code", normalized_code)}
        else
          {:error, :currency_not_allowed}
        end
    end
  end

  defp prepare_tx(Account, client_tx, db_record, _eco_user, known_codes) do
    if Map.has_key?(client_tx, "currency_code") do
      case Currencies.normalize_code(Map.get(client_tx, "currency_code")) do
        nil ->
          {:ok, Map.put(client_tx, "currency_code", nil)}

        normalized_code ->
          same_as_existing = db_record != nil and db_record.currency_code == normalized_code

          if same_as_existing or MapSet.member?(known_codes, normalized_code) do
            {:ok, Map.put(client_tx, "currency_code", normalized_code)}
          else
            {:error, :currency_not_allowed}
          end
      end
    else
      {:ok, client_tx}
    end
  end

  defp prepare_tx(_schema_module, client_tx, _db_record, _eco_user, _known_codes), do: {:ok, client_tx}

  defp updated_after_sync?(_db_updated_at, nil), do: true

  defp updated_after_sync?(db_updated_at, last_synced_at) do
    NaiveDateTime.compare(db_updated_at, last_synced_at) == :gt
  end
end
