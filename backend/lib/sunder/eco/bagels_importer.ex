defmodule Sunder.Eco.BagelsImporter do
  import Ecto.Query
  alias Sunder.Repo
  alias Sunder.Eco.{Account, Category, Entry}

  def import_db(eco_user_id, db_path) do
    # 1. Import Accounts
    bagel_accounts = read_json(db_path, "SELECT * FROM account")
    account_map = Enum.reduce(bagel_accounts, %{}, fn acc, acc_map ->
      name = acc["name"]
      
      active = Repo.one(from a in Account, where: a.name == ^name and a.eco_user_id == ^eco_user_id and is_nil(a.deleted_at))
      sunder_acc = 
        if active do
          active
        else
          {:ok, new_acc} = %Account{} |> Account.changeset(%{name: name, eco_user_id: eco_user_id}) |> Repo.insert()
          new_acc
        end
        
      Map.put(acc_map, acc["id"], sunder_acc.id)
    end)

    # 2. Import Categories
    bagel_cats = read_json(db_path, "SELECT * FROM category")
    cat_map = Enum.reduce(bagel_cats, %{}, fn cat, acc_map ->
      name = cat["name"]
      color = cat["color"] || "gray"
      
      active = Repo.one(from c in Category, where: c.title == ^name and c.eco_user_id == ^eco_user_id and is_nil(c.deleted_at))
      sunder_cat = 
        if active do
          active
        else
          {:ok, new_cat} = %Category{} |> Category.changeset(%{title: name, color: color, eco_user_id: eco_user_id}) |> Repo.insert()
          new_cat
        end
        
      Map.put(acc_map, cat["id"], sunder_cat.id)
    end)

    # 3. Import Records
    bagel_records = read_json(db_path, "SELECT * FROM record")
    entries_count = Enum.reduce(bagel_records, 0, fn rec, count ->
      amount = rec["amount"] * 1.0
      
      date_str = String.replace(rec["date"] || "", " ", "T")
      date = case NaiveDateTime.from_iso8601(date_str) do
        {:ok, dt} -> dt
        _ -> NaiveDateTime.truncate(NaiveDateTime.utc_now(), :second)
      end
      
      base_entry = %{
        date: date,
        title: rec["label"] || "Untitled",
        description: rec["tags"],
        eco_user_id: eco_user_id,
        category_id: cat_map[rec["categoryId"]]
      }
      
      if rec["isTransfer"] == 1 or rec["isTransfer"] == true do
        from_acc = account_map[rec["accountId"]]
        to_acc = account_map[rec["transferToAccountId"]]
        
        c1 = if from_acc do
          %Entry{} |> Entry.changeset(Map.merge(base_entry, %{amount: -abs(amount), account_id: from_acc})) |> Repo.insert!()
          1
        else 0 end
        
        c2 = if to_acc do
          %Entry{} |> Entry.changeset(Map.merge(base_entry, %{amount: abs(amount), account_id: to_acc})) |> Repo.insert!()
          1
        else 0 end
        
        count + c1 + c2
      else
        final_amount = if rec["isIncome"] == 1 or rec["isIncome"] == true, do: abs(amount), else: -abs(amount)
        acc_id = account_map[rec["accountId"]]
        
        if acc_id do
          %Entry{} |> Entry.changeset(Map.merge(base_entry, %{amount: final_amount, account_id: acc_id})) |> Repo.insert!()
          count + 1
        else
          count
        end
      end
    end)

    {:ok, %{accounts: map_size(account_map), categories: map_size(cat_map), entries: entries_count}}
  end

  defp read_json(db_path, query) do
    case System.cmd("sqlite3", [db_path, "-json", query]) do
      {json, 0} -> 
        case Jason.decode(json) do
          {:ok, result} when is_list(result) -> result
          _ -> []
        end
      _ -> []
    end
  end
end
