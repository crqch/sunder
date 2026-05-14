defmodule SunderWeb.Sync.Pipeline do
  alias Sunder.Repo

  def process(user, txs, timestamp) do
    last_synced_at =
      if timestamp, do: NaiveDateTime.from_iso8601!(timestamp), else: nil

    Ecto.Multi.new()
    |> SunderWeb.Sync.Guard.validate_writes(user, txs)
    |> SunderWeb.Sync.Writer.apply_writes(user, last_synced_at)
    |> SunderWeb.Sync.Reader.fetch_updates(user, last_synced_at, txs)
    |> Repo.transact()
  end
end
