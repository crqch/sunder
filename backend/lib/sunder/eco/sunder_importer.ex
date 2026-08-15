defmodule Sunder.Eco.SunderImporter do
  alias Sunder.Eco.{Account, Category, Entry}
  alias Sunder.Repo
  import Ecto.Query

  def import_data(eco_user_id, data) do
    # 1. Accounts: match by name or map old IDs to new IDs
    account_map = Enum.reduce(data["accounts"] || [], %{}, fn acc, acc_map ->
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

    # 2. Categories
    cat_map = Enum.reduce(data["categories"] || [], %{}, fn cat, acc_map ->
      title = cat["title"]
      active = Repo.one(from c in Category, where: c.title == ^title and c.eco_user_id == ^eco_user_id and is_nil(c.deleted_at))
      sunder_cat = 
        if active do
          active
        else
          {:ok, new_cat} = %Category{} |> Category.changeset(%{title: title, color: cat["color"], description: cat["description"], eco_user_id: eco_user_id}) |> Repo.insert()
          new_cat
        end
      Map.put(acc_map, cat["id"], sunder_cat.id)
    end)

    # 3. Entries
    entries = data["entries"] || []
    entries_count = Enum.reduce(entries, 0, fn e, count ->
      date = case NaiveDateTime.from_iso8601(e["date"]) do
        {:ok, dt} -> dt
        _ -> NaiveDateTime.truncate(NaiveDateTime.utc_now(), :second)
      end
      
      acc_id = account_map[e["account_id"]]
      cat_id = cat_map[e["category_id"]]
      
      if acc_id do
        %Entry{} 
        |> Entry.changeset(%{
          date: date,
          amount: e["amount"],
          title: e["title"],
          description: e["description"],
          location: e["location"],
          account_id: acc_id,
          category_id: cat_id,
          eco_user_id: eco_user_id
        }) 
        |> Repo.insert!()
        count + 1
      else
        count
      end
    end)

    %{accounts: map_size(account_map), categories: map_size(cat_map), entries: entries_count}
  end
end
