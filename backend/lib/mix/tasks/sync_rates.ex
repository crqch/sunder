defmodule Mix.Tasks.Sunder.SyncRates do
  @moduledoc """
  Fetches and persists currency exchange rates.

  Examples:

      mix sunder.sync_rates
      mix sunder.sync_rates --date 2026-11-01
      mix sunder.sync_rates --days 30
      mix sunder.sync_rates --days 30 --force
  """

  use Mix.Task

  alias Sunder.Contexts.Currencies
  alias Sunder.Currencies.Catalog

  @shortdoc "Syncs currency exchange rates from upstream providers"

  @impl Mix.Task
  def run(args) do
    Mix.Task.run("app.start")

    {opts, _, _} =
      OptionParser.parse(args,
        strict: [date: :string, days: :integer, force: :boolean]
      )

    Catalog.sync_catalog!()

    cond do
      date_str = opts[:date] ->
        case Date.from_iso8601(date_str) do
          {:ok, date} ->
            case Currencies.sync_rates_for_date(date) do
              {:ok, info} ->
                Mix.shell().info(
                  "Synced #{info.stored_count} rates for #{info.date} (source: #{info.source})"
                )

              {:error, reason} ->
                Mix.raise("Failed to sync rates for #{date_str}: #{inspect(reason)}")
            end

          {:error, _} ->
            Mix.raise("Invalid --date format (expected YYYY-MM-DD): #{date_str}")
        end

      days = opts[:days] ->
        to_date = Date.utc_today()
        from_date = Date.add(to_date, -max(days - 1, 0))
        force = Keyword.get(opts, :force, false)

        {:ok, stats} = Currencies.backfill_rates(from_date, to_date, force: force)

        Mix.shell().info(
          "Backfill #{from_date}..#{to_date}: synced=#{stats.synced}, skipped=#{stats.skipped}, failed=#{stats.failed}"
        )

      true ->
        case Currencies.sync_latest_rates() do
          {:ok, info} ->
            Mix.shell().info(
              "Synced #{info.stored_count} latest rates for #{info.date} (source: #{info.source})"
            )

          {:error, reason} ->
            Mix.raise("Failed to sync latest rates: #{inspect(reason)}")
        end
    end
  end
end
