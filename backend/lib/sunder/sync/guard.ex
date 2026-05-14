defmodule SunderWeb.Sync.Guard do
  import Ecto.Query

  alias Sunder.Eco.{Entry, Category, Account}

  def validate_writes(multi, %{eco_user: eco_user}, txs) do
    Ecto.Multi.run(multi, :guard, fn repo, _changes ->
      case txs do
        %{
          "accounts" => tx_accounts,
          "categories" => tx_categories,
          "entries" => tx_entries
        } ->
          {:ok,
           %{
             accounts: partition_txs(repo, tx_accounts, eco_user.id, Account),
             categories: partition_txs(repo, tx_categories, eco_user.id, Category),
             entries: partition_txs(repo, tx_entries, eco_user.id, Entry)
           }}

        _ ->
          {:error, :txs_invalid_format}
      end
    end)
  end

  defp partition_txs(_repo, [], _user_id, _schema), do: %{valid: [], invalid: [], db_records: %{}}

  defp partition_txs(_repo, nil, _user_id, _schema),
    do: %{valid: [], invalid: [], db_records: %{}}

  defp partition_txs(repo, client_txs, user_id, schema) do
    ids = Enum.map(client_txs, fn tx -> tx["id"] end)

    db_records_map =
      from(s in schema, where: s.id in ^ids)
      |> repo.all()
      |> Map.new(&{&1.id, &1})

    Enum.reduce(client_txs, %{valid: [], invalid: [], db_records: db_records_map}, fn tx, acc ->
      case Map.get(db_records_map, tx["id"]) do
        nil ->
          %{acc | valid: [tx | acc.valid]}

        %{eco_user_id: ^user_id} ->
          %{acc | valid: [tx | acc.valid]}

        _other ->
          %{acc | invalid: [tx | acc.invalid]}
      end
    end)
  end
end
