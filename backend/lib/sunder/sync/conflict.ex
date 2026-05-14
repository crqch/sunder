defmodule SunderWeb.Sync.Conflict do
  alias Sunder.Eco.{Entry, Account, Category}

  # TODO: Conflict strategy to be developed (currently client wins)
  def resolve(Account, db_record, client_tx) do
    Account.changeset(db_record, client_tx)
  end

  def resolve(Category, db_record, client_tx) do
    Category.changeset(db_record, client_tx)
  end

  def resolve(Entry, db_record, client_tx) do
    Entry.changeset(db_record, client_tx)
  end

  def resolve(_schema, db_record, _client_tx) do
    db_record
  end
end
