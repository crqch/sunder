defmodule Sunder.Currencies.Poller do
  @moduledoc """
  Background GenServer that polls daily exchange rates and persists them in Postgres.

  On boot it ensures the built-in currency catalog is synced and fetches today's
  rates if they are not already stored. It then polls once every 24 hours (with
  shorter retry intervals if an upstream provider fails).
  """

  use GenServer
  require Logger

  alias Sunder.Contexts.Currencies
  alias Sunder.Currencies.Catalog

  @default_initial_delay_ms :timer.seconds(5)
  @default_poll_interval_ms :timer.hours(24)
  @default_retry_interval_ms :timer.minutes(30)

  def start_link(opts \\ []) do
    GenServer.start_link(__MODULE__, opts, name: __MODULE__)
  end

  @doc """
  Triggers an immediate asynchronous exchange rate poll.
  """
  def poll_now do
    GenServer.cast(__MODULE__, :poll_now)
  end

  @impl true
  def init(opts) do
    if :ets.whereis(:sunder_currency_fetch_cooldown) == :undefined do
      :ets.new(:sunder_currency_fetch_cooldown, [:named_table, :public, :set])
    end

    config = Application.get_env(:sunder, __MODULE__, [])

    state = %{
      initial_delay_ms:
        Keyword.get(opts, :initial_delay_ms) ||
          Keyword.get(config, :initial_delay_ms, @default_initial_delay_ms),
      poll_interval_ms:
        Keyword.get(opts, :poll_interval_ms) ||
          Keyword.get(config, :poll_interval_ms, @default_poll_interval_ms),
      retry_interval_ms:
        Keyword.get(opts, :retry_interval_ms) ||
          Keyword.get(config, :retry_interval_ms, @default_retry_interval_ms),
      timer_ref: nil
    }

    timer_ref = Process.send_after(self(), :bootstrap, state.initial_delay_ms)
    {:ok, %{state | timer_ref: timer_ref}}
  end

  @impl true
  def handle_cast(:poll_now, state) do
    state = cancel_timer(state)
    next_delay = run_poll(force: true, state: state)
    {:noreply, schedule_next(state, next_delay)}
  end

  @impl true
  def handle_info(:bootstrap, state) do
    try do
      Catalog.sync_catalog!()
    rescue
      e ->
        Logger.warning("Failed to sync currency catalog on boot: #{Exception.message(e)}")
    end

    next_delay = run_poll(force: false, state: state)
    {:noreply, schedule_next(state, next_delay)}
  end

  @impl true
  def handle_info(:poll, state) do
    next_delay = run_poll(force: true, state: state)
    {:noreply, schedule_next(state, next_delay)}
  end

  defp run_poll(force: force, state: state) do
    today = Date.utc_today()

    try do
      ensure_catalog!()

      if not force and Currencies.has_rates_for_date?(today) do
        Logger.debug("Exchange rates for #{today} already present; skipping boot poll.")
        state.poll_interval_ms
      else
        case Currencies.sync_latest_rates() do
          {:ok, %{date: date, source: source, stored_count: count}} ->
            Logger.info(
              "Synced #{count} exchange rates for #{date} from #{source}."
            )

            state.poll_interval_ms

          {:error, reason} ->
            Logger.warning("Daily exchange rate poll failed: #{inspect(reason)}")
            state.retry_interval_ms
        end
      end
    rescue
      e ->
        Logger.warning("Exchange rate poller error: #{Exception.message(e)}")
        state.retry_interval_ms
    end
  end

  defp ensure_catalog! do
    if MapSet.size(Currencies.known_currency_codes()) == 0 do
      Catalog.sync_catalog!()
    end
  end

  defp schedule_next(state, delay_ms) do
    timer_ref = Process.send_after(self(), :poll, delay_ms)
    %{state | timer_ref: timer_ref}
  end

  defp cancel_timer(%{timer_ref: nil} = state), do: state

  defp cancel_timer(%{timer_ref: ref} = state) do
    Process.cancel_timer(ref)
    %{state | timer_ref: nil}
  end
end
