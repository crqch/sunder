defmodule Sunder.Currencies.FetcherBehaviour do
  @moduledoc """
  Behaviour for fetching exchange rates relative to a canonical base currency (USD).
  """

  @type fetch_result :: %{
          date: Date.t(),
          base: String.t(),
          source: String.t(),
          rates: %{String.t() => Decimal.t()}
        }

  @callback fetch_rates(Date.t() | :latest) :: {:ok, fetch_result()} | {:error, term()}
end

defmodule Sunder.Currencies.Fetcher do
  @moduledoc """
  Fetches daily and historical exchange rates (160+ world currencies) using Erlang's
  built-in `:httpc` client with automatic multi-provider fallback:

    1. fawazahmed0/exchange-api (jsDelivr CDN) — 160+ fiat currencies, latest & historical
    2. fawazahmed0/exchange-api (Cloudflare Pages mirror) — 160+ fiat currencies, latest & historical
    3. open.er-api.com — 160+ fiat currencies (latest only)
    4. api.frankfurter.dev (ECB) — major currencies, latest & historical
  """

  @behaviour Sunder.Currencies.FetcherBehaviour

  require Logger

  @connect_timeout_ms 10_000
  @request_timeout_ms 15_000

  @impl true
  def fetch_rates(:latest) do
    providers = [
      {"fawazahmed0_jsdelivr",
       "https://cdn.jsdelivr.net/npm/@fawazahmed0/currency-api@latest/v1/currencies/usd.json",
       &parse_fawazahmed0/2},
      {"fawazahmed0_pages", "https://latest.currency-api.pages.dev/v1/currencies/usd.json",
       &parse_fawazahmed0/2},
      {"open_er_api", "https://open.er-api.com/v6/latest/USD", &parse_open_er_api/2},
      {"frankfurter", "https://api.frankfurter.dev/v1/latest?base=USD", &parse_frankfurter/2}
    ]

    try_providers(providers, Date.utc_today())
  end

  @impl true
  def fetch_rates(%Date{} = date) do
    date_str = Date.to_iso8601(date)

    providers = [
      {"fawazahmed0_jsdelivr",
       "https://cdn.jsdelivr.net/npm/@fawazahmed0/currency-api@#{date_str}/v1/currencies/usd.json",
       &parse_fawazahmed0/2},
      {"fawazahmed0_pages", "https://#{date_str}.currency-api.pages.dev/v1/currencies/usd.json",
       &parse_fawazahmed0/2},
      {"frankfurter", "https://api.frankfurter.dev/v1/#{date_str}?base=USD", &parse_frankfurter/2}
    ]

    try_providers(providers, date)
  end

  defp try_providers([], _fallback_date) do
    {:error, :all_providers_failed}
  end

  defp try_providers([{source, url, parser} | rest], fallback_date) do
    case http_get_json(url) do
      {:ok, payload} ->
        case parser.(payload, fallback_date) do
          {:ok, result} ->
            {:ok, Map.put(result, :source, source)}

          {:error, reason} ->
            Logger.warning("Currency provider #{source} parse failed: #{inspect(reason)}")
            try_providers(rest, fallback_date)
        end

      {:error, reason} ->
        Logger.warning("Currency provider #{source} request failed: #{inspect(reason)}")
        try_providers(rest, fallback_date)
    end
  end

  defp http_get_json(url) do
    uri = URI.parse(url)
    host_charlist = String.to_charlist(uri.host || "")

    ssl_opts = [
      verify: :verify_peer,
      cacerts: :public_key.cacerts_get(),
      depth: 3,
      server_name_indication: host_charlist,
      customize_hostname_check: [
        match_fun: :public_key.pkix_verify_hostname_match_fun(:https)
      ]
    ]

    http_opts = [
      timeout: @request_timeout_ms,
      connect_timeout: @connect_timeout_ms,
      ssl: ssl_opts
    ]

    headers = [
      {~c"accept", ~c"application/json"},
      {~c"user-agent", ~c"Sunder/1.0"}
    ]

    case :httpc.request(:get, {String.to_charlist(url), headers}, http_opts,
           body_format: :binary
         ) do
      {:ok, {{_, 200, _}, _resp_headers, body}} ->
        Jason.decode(body)

      {:ok, {{_, status, _}, _resp_headers, _body}} ->
        {:error, {:http_status, status}}

      {:error, reason} ->
        {:error, reason}
    end
  end

  defp parse_fawazahmed0(%{"date" => date_str, "usd" => raw_rates}, fallback_date)
       when is_map(raw_rates) do
    date = parse_date_or(date_str, fallback_date)
    rates = normalize_rates_map(raw_rates)

    if map_size(rates) > 1 do
      {:ok, %{date: date, base: "USD", rates: rates}}
    else
      {:error, :empty_rates}
    end
  end

  defp parse_fawazahmed0(_, _), do: {:error, :invalid_payload}

  defp parse_open_er_api(%{"result" => "success", "rates" => raw_rates} = payload, fallback_date)
       when is_map(raw_rates) do
    date =
      case payload["time_last_update_unix"] do
        unix when is_integer(unix) ->
          unix |> DateTime.from_unix!() |> DateTime.to_date()

        _ ->
          fallback_date
      end

    rates = normalize_rates_map(raw_rates)

    if map_size(rates) > 1 do
      {:ok, %{date: date, base: "USD", rates: rates}}
    else
      {:error, :empty_rates}
    end
  end

  defp parse_open_er_api(_, _), do: {:error, :invalid_payload}

  defp parse_frankfurter(%{"date" => date_str, "rates" => raw_rates}, fallback_date)
       when is_map(raw_rates) do
    date = parse_date_or(date_str, fallback_date)
    rates = normalize_rates_map(raw_rates)

    if map_size(rates) > 1 do
      {:ok, %{date: date, base: "USD", rates: rates}}
    else
      {:error, :empty_rates}
    end
  end

  defp parse_frankfurter(_, _), do: {:error, :invalid_payload}

  defp parse_date_or(date_str, fallback_date) when is_binary(date_str) do
    case Date.from_iso8601(date_str) do
      {:ok, date} -> date
      _ -> fallback_date
    end
  end

  defp parse_date_or(_, fallback_date), do: fallback_date

  defp normalize_rates_map(raw_rates) do
    raw_rates
    |> Enum.reduce(%{"USD" => Decimal.new("1")}, fn {code, value}, acc ->
      with true <- is_binary(code),
           {:ok, decimal} <- to_positive_decimal(value) do
        Map.put(acc, code |> String.trim() |> String.upcase(), decimal)
      else
        _ -> acc
      end
    end)
  end

  defp to_positive_decimal(value) when is_float(value) and value > 0 do
    {:ok, Decimal.from_float(value)}
  end

  defp to_positive_decimal(value) when is_integer(value) and value > 0 do
    {:ok, Decimal.new(value)}
  end

  defp to_positive_decimal(value) when is_binary(value) do
    case Decimal.parse(value) do
      {dec, ""} ->
        if Decimal.compare(dec, 0) == :gt, do: {:ok, dec}, else: :error

      _ ->
        :error
    end
  end

  defp to_positive_decimal(_), do: :error
end
