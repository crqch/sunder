defmodule Sunder.Contexts.Currencies do
  @moduledoc """
  Server-side currency registry, user default currency preferences, historical
  exchange rate storage, and cross-currency conversion.
  """

  import Ecto.Query

  require Logger

  alias Sunder.Eco.{Currency, EcoUser, ExchangeRate}
  alias Sunder.Repo

  @canonical_base "USD"
  @rate_precision 12

  def canonical_base, do: @canonical_base

  def normalize_code(code) when is_binary(code) do
    case String.trim(code) do
      "" -> nil
      trimmed -> String.upcase(trimmed)
    end
  end

  def normalize_code(_), do: nil

  # ---------------------------------------------------------------------------
  # Currency Registry (Server-Managed Allowed Currencies)
  # ---------------------------------------------------------------------------

  def list_active_currencies do
    Repo.all(
      from(c in Currency,
        where: c.is_active == true,
        order_by: [asc: c.code]
      )
    )
  end

  def list_all_currencies do
    Repo.all(
      from(c in Currency,
        order_by: [asc: c.code]
      )
    )
  end

  def get_currency(code) when is_binary(code) do
    case normalize_code(code) do
      nil -> nil
      normalized -> Repo.get(Currency, normalized)
    end
  end

  def get_currency(_), do: nil

  def get_active_currency(code) when is_binary(code) do
    case normalize_code(code) do
      nil ->
        nil

      normalized ->
        Repo.one(from(c in Currency, where: c.code == ^normalized and c.is_active == true))
    end
  end

  def get_active_currency(_), do: nil

  def currency_allowed?(code) do
    not is_nil(get_active_currency(code))
  end

  def create_currency(attrs) do
    %Currency{}
    |> Currency.changeset(attrs)
    |> Repo.insert()
  end

  def update_currency(%Currency{} = currency, attrs) do
    currency
    |> Currency.update_changeset(attrs)
    |> Repo.update()
  end

  # ---------------------------------------------------------------------------
  # User Default Currency Preferences
  # ---------------------------------------------------------------------------

  def get_user_default_currency(%EcoUser{} = eco_user) do
    code = eco_user.default_currency_code || @canonical_base
    get_currency(code) || get_currency(@canonical_base)
  end

  def set_user_default_currency(%EcoUser{} = eco_user, currency_code) do
    case get_active_currency(currency_code) do
      nil ->
        {:error, :currency_not_allowed}

      %Currency{code: normalized_code} = currency ->
        eco_user
        |> EcoUser.preferences_changeset(%{default_currency_code: normalized_code})
        |> Repo.update()
        |> case do
          {:ok, updated_eco_user} -> {:ok, %{eco_user: updated_eco_user, currency: currency}}
          {:error, changeset} -> {:error, changeset}
        end
    end
  end

  # ---------------------------------------------------------------------------
  # Exchange Rate Storage & Synchronization
  # ---------------------------------------------------------------------------

  def known_currency_codes do
    Repo.all(from(c in Currency, select: c.code))
    |> MapSet.new()
  end

  def has_rates_for_date?(%Date{} = date) do
    Repo.exists?(
      from(r in ExchangeRate,
        where:
          r.base_currency_code == @canonical_base and
            r.rate_date == ^date and
            r.source != "manual" and
            r.target_currency_code != @canonical_base
      )
    )
  end

  def latest_rate_date do
    Repo.one(
      from(r in ExchangeRate,
        where: r.base_currency_code == @canonical_base,
        select: max(r.rate_date)
      )
    )
  end

  def sync_latest_rates do
    fetcher = fetcher_module()

    with {:ok, %{date: date, base: base, source: source, rates: rates}} <-
           fetcher.fetch_rates(:latest),
         {:ok, count} <- store_rates(date, rates, source, base) do
      {:ok, %{date: date, base: base, source: source, stored_count: count}}
    end
  end

  def sync_rates_for_date(%Date{} = date) do
    fetcher = fetcher_module()

    with {:ok, %{date: fetched_date, base: base, source: source, rates: rates}} <-
           fetcher.fetch_rates(date),
         # Store under the fetched date, and if the provider returned a nearby date
         # (e.g. Friday for a weekend), also index it under the requested date if distinct
         {:ok, count} <- store_rates(fetched_date, rates, source, base) do
      if fetched_date != date do
        store_rates(date, rates, source, base)
      end

      {:ok, %{date: date, fetched_date: fetched_date, base: base, source: source, stored_count: count}}
    end
  end

  def backfill_rates(%Date{} = from_date, %Date{} = to_date, opts \\ []) do
    force = Keyword.get(opts, :force, false)

    if Date.compare(from_date, to_date) == :gt do
      {:error, :invalid_date_range}
    else
      results =
        Date.range(from_date, to_date)
        |> Enum.reduce(%{synced: 0, skipped: 0, failed: 0}, fn date, acc ->
          cond do
            not force and has_rates_for_date?(date) ->
              %{acc | skipped: acc.skipped + 1}

            true ->
              case sync_rates_for_date(date) do
                {:ok, _} -> %{acc | synced: acc.synced + 1}
                {:error, _} -> %{acc | failed: acc.failed + 1}
              end
          end
        end)

      {:ok, results}
    end
  end

  def store_rates(%Date{} = rate_date, rates_map, source, base \\ @canonical_base)
      when is_map(rates_map) and is_binary(source) do
    base_code = normalize_code(base) || @canonical_base

    known_codes =
      Repo.all(from(c in Currency, select: c.code))
      |> MapSet.new()

    if not MapSet.member?(known_codes, base_code) do
      {:error, :unknown_base_currency}
    else
      now = NaiveDateTime.truncate(NaiveDateTime.utc_now(), :second)

      normalized_rates =
        rates_map
        |> Enum.reduce(%{base_code => Decimal.new("1")}, fn {raw_code, raw_rate}, acc ->
          code = normalize_code(raw_code)

          with true <- not is_nil(code) and MapSet.member?(known_codes, code),
               {:ok, dec_rate} <- cast_positive_decimal(raw_rate) do
            Map.put(acc, code, Decimal.round(dec_rate, @rate_precision))
          else
            _ -> acc
          end
        end)

      rows =
        Enum.map(normalized_rates, fn {target_code, rate_dec} ->
          %{
            id: ExCuid2.generate(),
            base_currency_code: base_code,
            target_currency_code: target_code,
            rate: rate_dec,
            rate_date: rate_date,
            source: source,
            inserted_at: now,
            updated_at: now
          }
        end)

      {count, _} =
        Repo.insert_all(ExchangeRate, rows,
          on_conflict: {:replace, [:rate, :source, :updated_at]},
          conflict_target: [:base_currency_code, :target_currency_code, :rate_date]
        )

      {:ok, count}
    end
  end

  def upsert_manual_rate(base_code, target_code, raw_rate, %Date{} = rate_date) do
    with base when not is_nil(base) <- normalize_code(base_code),
         target when not is_nil(target) <- normalize_code(target_code),
         %Currency{} <- get_currency(base),
         %Currency{} <- get_currency(target),
         {:ok, rate_dec} <- cast_positive_decimal(raw_rate) do
      cond do
        base == target ->
          {:error, :same_currency}

        base == @canonical_base ->
          upsert_canonical_rate(target, rate_dec, rate_date, "manual")

        target == @canonical_base ->
          inverted = Decimal.div(Decimal.new("1"), rate_dec) |> Decimal.round(@rate_precision)
          upsert_canonical_rate(base, inverted, rate_date, "manual")

        true ->
          case fetch_usd_rate(base, rate_date) do
            {:ok, {usd_to_base, _}} ->
              usd_to_target = Decimal.mult(usd_to_base, rate_dec) |> Decimal.round(@rate_precision)
              upsert_canonical_rate(target, usd_to_target, rate_date, "manual")

            :error ->
              {:error, :base_rate_not_found}
          end
      end
    else
      _ -> {:error, :invalid_rate_input}
    end
  end

  defp upsert_canonical_rate(target_code, rate_dec, rate_date, source) do
    case store_rates(rate_date, %{target_code => rate_dec}, source, @canonical_base) do
      {:ok, _} ->
        rate =
          Repo.get_by!(ExchangeRate,
            base_currency_code: @canonical_base,
            target_currency_code: target_code,
            rate_date: rate_date
          )

        {:ok, rate}

      error ->
        error
    end
  end

  # ---------------------------------------------------------------------------
  # Historical Lookup & Conversion
  # ---------------------------------------------------------------------------

  def get_rate(from_code, to_code, date \\ Date.utc_today(), opts \\ [])

  def get_rate(from_code, to_code, %Date{} = date, opts) do
    from_norm = normalize_code(from_code)
    to_norm = normalize_code(to_code)

    with %Currency{} <- get_currency(from_norm),
         %Currency{} <- get_currency(to_norm) do
      if from_norm == to_norm do
        {:ok,
         %{
           from: from_norm,
           to: to_norm,
           rate: Decimal.new("1"),
           rate_date: date,
           requested_date: date
         }}
      else
        maybe_fetch_on_demand(date, opts)

        with {:ok, {from_usd_rate, from_date}} <- fetch_usd_rate(from_norm, date),
             {:ok, {to_usd_rate, to_date}} <- fetch_usd_rate(to_norm, date) do
          cross_rate =
            Decimal.div(to_usd_rate, from_usd_rate)
            |> Decimal.round(@rate_precision)

          effective_date =
            cond do
              from_norm == @canonical_base -> to_date
              to_norm == @canonical_base -> from_date
              Date.compare(from_date, to_date) == :lt -> from_date
              true -> to_date
            end

          {:ok,
           %{
             from: from_norm,
             to: to_norm,
             rate: cross_rate,
             rate_date: effective_date,
             requested_date: date
           }}
        else
          :error -> {:error, :rate_not_found}
        end
      end
    else
      _ -> {:error, :unknown_currency}
    end
  end

  def convert(amount, from_code, to_code, date \\ Date.utc_today(), opts \\ [])

  def convert(amount, from_code, to_code, %Date{} = date, opts) do
    with {:ok, amount_dec} <- cast_decimal(amount),
         to_norm when not is_nil(to_norm) <- normalize_code(to_code),
         %Currency{decimal_digits: digits} <- get_currency(to_norm),
         {:ok, rate_info} <- get_rate(from_code, to_norm, date, opts) do
      exact = Decimal.mult(amount_dec, rate_info.rate)
      rounded = Decimal.round(exact, digits)

      {:ok,
       %{
         from: rate_info.from,
         to: rate_info.to,
         amount: amount_dec,
         converted_amount: rounded,
         exact_converted_amount: exact,
         rate: rate_info.rate,
         rate_date: rate_info.rate_date,
         requested_date: date
       }}
    else
      {:error, reason} -> {:error, reason}
      _ -> {:error, :invalid_conversion_input}
    end
  end

  def get_rates_snapshot(base_code \\ @canonical_base, opts \\ []) do
    requested_date = Keyword.get(opts, :date) || Date.utc_today()
    symbols = Keyword.get(opts, :symbols)

    with base_norm when not is_nil(base_norm) <- normalize_code(base_code),
         %Currency{} <- get_currency(base_norm) do
      maybe_fetch_on_demand(requested_date, opts)

      usd_rates = latest_usd_rates_on_or_before(requested_date)

      usd_rates =
        if map_size(usd_rates) == 0 do
          earliest_usd_rates_after(requested_date)
        else
          usd_rates
        end

      base_lookup =
        if base_norm == @canonical_base do
          {:ok, {Decimal.new("1"), requested_date}}
        else
          case Map.get(usd_rates, base_norm) do
            nil -> :error
            pair -> {:ok, pair}
          end
        end

      case base_lookup do
        :error ->
          {:error, :rate_not_found}

        {:ok, {base_usd_rate, _base_date}} ->
          active_codes =
            Repo.all(from(c in Currency, where: c.is_active == true, select: c.code))
            |> MapSet.new()

          filter_set =
            case symbols do
              nil -> nil
              [] -> nil
              list -> list |> Enum.map(&normalize_code/1) |> Enum.reject(&is_nil/1) |> MapSet.new()
            end

          {rates_map, effective_date} =
            usd_rates
            |> Enum.reduce({%{}, nil}, fn {code, {usd_rate, r_date}}, {acc_rates, acc_date} ->
              allowed =
                if filter_set do
                  MapSet.member?(filter_set, code)
                else
                  MapSet.member?(active_codes, code)
                end

              if allowed do
                rebased =
                  if code == base_norm do
                    Decimal.new("1")
                  else
                    Decimal.div(usd_rate, base_usd_rate) |> Decimal.round(@rate_precision)
                  end

                next_date =
                  cond do
                    is_nil(acc_date) -> r_date
                    Date.compare(r_date, acc_date) == :gt -> r_date
                    true -> acc_date
                  end

                {Map.put(acc_rates, code, Decimal.to_float(rebased)), next_date}
              else
                {acc_rates, acc_date}
              end
            end)

          rates_map =
            if is_nil(filter_set) or MapSet.member?(filter_set, base_norm) do
              Map.put_new(rates_map, base_norm, 1.0)
            else
              rates_map
            end

          if map_size(rates_map) == 0 do
            {:error, :rate_not_found}
          else
            {:ok,
             %{
               base: base_norm,
               date: effective_date || requested_date,
               requested_date: requested_date,
               rates: rates_map
             }}
          end
      end
    else
      _ -> {:error, :unknown_currency}
    end
  end

  def get_rates_history(base_code \\ @canonical_base, %Date{} = from_date, %Date{} = to_date, opts \\ []) do
    symbols = Keyword.get(opts, :symbols)

    with :ok <- validate_history_range(from_date, to_date),
         base_norm when not is_nil(base_norm) <- normalize_code(base_code),
         %Currency{} <- get_currency(base_norm) do
      filter_codes =
        case symbols do
          nil -> nil
          [] -> nil
          list -> list |> Enum.map(&normalize_code/1) |> Enum.reject(&is_nil/1) |> Enum.uniq()
        end

      query =
        from(r in ExchangeRate,
          where:
            r.base_currency_code == @canonical_base and
              r.rate_date >= ^from_date and
              r.rate_date <= ^to_date,
          order_by: [asc: r.rate_date, asc: r.target_currency_code]
        )

      query =
        if filter_codes do
          codes_with_base = Enum.uniq([base_norm | filter_codes])
          from(r in query, where: r.target_currency_code in ^codes_with_base)
        else
          query
        end

      rows = Repo.all(query)

      grouped_by_date =
        rows
        |> Enum.group_by(& &1.rate_date)
        |> Enum.reduce(%{}, fn {rate_date, date_rows}, acc ->
          row_map = Map.new(date_rows, &{&1.target_currency_code, &1.rate})

          base_usd =
            if base_norm == @canonical_base do
              Decimal.new("1")
            else
              Map.get(row_map, base_norm)
            end

          if is_nil(base_usd) do
            acc
          else
            rebased_map =
              row_map
              |> Enum.reduce(%{}, fn {code, usd_rate}, inner ->
                if is_nil(filter_codes) or code in filter_codes do
                  val =
                    if code == base_norm do
                      1.0
                    else
                      Decimal.div(usd_rate, base_usd)
                      |> Decimal.round(@rate_precision)
                      |> Decimal.to_float()
                    end

                  Map.put(inner, code, val)
                else
                  inner
                end
              end)

            rebased_map =
              if is_nil(filter_codes) or base_norm in filter_codes do
                Map.put_new(rebased_map, base_norm, 1.0)
              else
                rebased_map
              end

            Map.put(acc, Date.to_iso8601(rate_date), rebased_map)
          end
        end)

      {:ok,
       %{
         base: base_norm,
         from: from_date,
         to: to_date,
         rates: grouped_by_date
       }}
    else
      {:error, reason} -> {:error, reason}
      _ -> {:error, :unknown_currency}
    end
  end

  # ---------------------------------------------------------------------------
  # Private Helpers
  # ---------------------------------------------------------------------------

  defp validate_history_range(from_date, to_date) do
    cond do
      Date.compare(from_date, to_date) == :gt ->
        {:error, :invalid_date_range}

      Date.diff(to_date, from_date) > 366 ->
        {:error, :date_range_too_large}

      true ->
        :ok
    end
  end

  defp fetch_usd_rate(@canonical_base, date), do: {:ok, {Decimal.new("1"), date}}

  defp fetch_usd_rate(currency_code, date) do
    on_or_before =
      Repo.one(
        from(r in ExchangeRate,
          where:
            r.base_currency_code == @canonical_base and
              r.target_currency_code == ^currency_code and
              r.rate_date <= ^date,
          order_by: [desc: r.rate_date],
          limit: 1,
          select: {r.rate, r.rate_date}
        )
      )

    case on_or_before do
      {rate, r_date} ->
        {:ok, {rate, r_date}}

      nil ->
        earliest_after =
          Repo.one(
            from(r in ExchangeRate,
              where:
                r.base_currency_code == @canonical_base and
                  r.target_currency_code == ^currency_code and
                  r.rate_date > ^date,
              order_by: [asc: r.rate_date],
              limit: 1,
              select: {r.rate, r.rate_date}
            )
          )

        case earliest_after do
          {rate, r_date} -> {:ok, {rate, r_date}}
          nil -> :error
        end
    end
  end

  defp latest_usd_rates_on_or_before(%Date{} = date) do
    Repo.all(
      from(r in ExchangeRate,
        where: r.base_currency_code == @canonical_base and r.rate_date <= ^date,
        distinct: [asc: r.target_currency_code],
        order_by: [asc: r.target_currency_code, desc: r.rate_date],
        select: {r.target_currency_code, r.rate, r.rate_date}
      )
    )
    |> Map.new(fn {code, rate, r_date} -> {code, {rate, r_date}} end)
  end

  defp earliest_usd_rates_after(%Date{} = date) do
    Repo.all(
      from(r in ExchangeRate,
        where: r.base_currency_code == @canonical_base and r.rate_date > ^date,
        distinct: [asc: r.target_currency_code],
        order_by: [asc: r.target_currency_code, asc: r.rate_date],
        select: {r.target_currency_code, r.rate, r.rate_date}
      )
    )
    |> Map.new(fn {code, rate, r_date} -> {code, {rate, r_date}} end)
  end

  defp maybe_fetch_on_demand(%Date{} = date, opts) do
    default_on_demand = Application.get_env(:sunder, :currency_fetch_on_demand, true)
    fetch_if_missing = Keyword.get(opts, :fetch_if_missing, default_on_demand)
    today = Date.utc_today()

    if fetch_if_missing and Date.compare(date, today) != :gt and not has_rates_for_date?(date) and
         claim_on_demand_fetch_slot(date) do
      result =
        if date == today do
          sync_latest_rates()
        else
          sync_rates_for_date(date)
        end

      case result do
        {:ok, _} -> :ok
        {:error, reason} -> Logger.warning("On-demand rate fetch failed for #{date}: #{inspect(reason)}")
      end
    end

    :ok
  end

  defp claim_on_demand_fetch_slot(%Date{} = date) do
    cooldown_ms = Application.get_env(:sunder, :currency_on_demand_cooldown_ms, :timer.minutes(15))

    if cooldown_ms <= 0 or :ets.whereis(:sunder_currency_fetch_cooldown) == :undefined do
      true
    else
      now_ms = System.monotonic_time(:millisecond)
      key = Date.to_iso8601(date)

      case :ets.lookup(:sunder_currency_fetch_cooldown, key) do
        [{^key, last_attempt_ms}] when now_ms - last_attempt_ms < cooldown_ms ->
          false

        _ ->
          :ets.insert(:sunder_currency_fetch_cooldown, {key, now_ms})
          true
      end
    end
  end

  defp fetcher_module do
    Application.get_env(:sunder, :currency_fetcher, Sunder.Currencies.Fetcher)
  end

  defp cast_positive_decimal(value) do
    case cast_decimal(value) do
      {:ok, dec} ->
        if Decimal.compare(dec, 0) == :gt, do: {:ok, dec}, else: :error

      :error ->
        :error
    end
  end

  defp cast_decimal(%Decimal{} = dec), do: {:ok, dec}
  defp cast_decimal(value) when is_float(value), do: {:ok, Decimal.from_float(value)}
  defp cast_decimal(value) when is_integer(value), do: {:ok, Decimal.new(value)}

  defp cast_decimal(value) when is_binary(value) do
    case Decimal.parse(String.trim(value)) do
      {dec, ""} -> {:ok, dec}
      _ -> :error
    end
  end

  defp cast_decimal(_), do: :error
end
