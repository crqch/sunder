defmodule Mix.Tasks.Sunder.ImportBagels do
  use Mix.Task
  import Ecto.Query
  alias Sunder.Repo
  alias Sunder.Accounts.User
  alias Sunder.Eco.BagelsImporter

  @shortdoc "Imports bagels db.db to a user"
  
  def run([email, db_path]) do
    Mix.Task.run("app.start")

    user = Repo.one(from u in User, where: u.email == ^email, preload: [:eco_user])
    if !user do
      IO.puts("User not found")
      System.halt(1)
    end
    
    IO.puts("Importing for user: #{user.email}...")
    {:ok, stats} = BagelsImporter.import_db(user.eco_user.id, db_path)
    
    IO.puts("Imported #{stats.accounts} accounts, #{stats.categories} categories, and #{stats.entries} entries.")
  end
  
  def run(_) do
    IO.puts("Usage: mix sunder.import_bagels <user_email> <db_path>")
  end
end
