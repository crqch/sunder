defmodule SunderWeb.CurrencyController do
  use SunderWeb, :controller

  alias Sunder.Contexts.Currencies

  @currency_schema %{
    type: :object,
    properties: %{
      code: %{type: :string},
      name: %{type: :string},
      symbol: %{type: :string},
      symbol_after: %{type: :boolean},
      decimal_digits: %{type: :integer}
    }
  }

  operation(:index,
    summary: "List server-allowed active currencies",
    responses: [
      ok: {
        %{
          type: :object,
          properties: %{
            currencies: %{type: :array, items: @currency_schema}
          }
        },
        [description: "Active currencies supported by this server"]
      }
    ]
  )

  def index(conn, _params) do
    currencies =
      Currencies.list_active_currencies()
      |> Enum.map(&serialize_currency/1)

    json(conn, %{currencies: currencies})
  end

  operation(:show,
    summary: "Get currency details by ISO code",
    parameters: [
      code: [
        in: :path,
        schema: %{type: :string},
        required: true,
        description: "Currency code (e.g. USD, EUR, PLN)"
      ]
    ],
    responses: [
      ok: {@currency_schema, [description: "Currency metadata"]},
      not_found: {
        %{type: :object, properties: %{error: %{type: :string}}},
        [description: "Currency not found or inactive"]
      }
    ]
  )

  def show(conn, %{"code" => code}) do
    case Currencies.get_active_currency(code) do
      nil ->
        conn |> put_status(:not_found) |> json(%{error: "currency_not_found"})

      currency ->
        json(conn, serialize_currency(currency))
    end
  end

  operation(:rates,
    summary: "Get exchange rates for a date",
    parameters: [
      base: [
        in: :query,
        schema: %{type: :string},
        required: false,
        description: "Base currency code (default: USD)"
      ],
      date: [
        in: :query,
        schema: %{type: :string, format: :date},
        required: false,
        description: "Target date in YYYY-MM-DD (default: today)"
      ],
      symbols: [
        in: :query,
        schema: %{type: :string},
        required: false,
        description: "Optional comma-separated list of target currency codes"
      ]
    ],
    responses: [
      ok: {
        %{
          type: :object,
          properties: %{
            base: %{type: :string},
            date: %{type: :string, format: :date},
            requested_date: %{type: :string, format: :date},
            rates: %{type: :object}
          }
        },
        [description: "Exchange rates snapshot"]
      },
      bad_request: {
        %{type: :object, properties: %{error: %{type: :string}}},
        [description: "Invalid query parameters"]
      },
      not_found: {
        %{type: :object, properties: %{error: %{type: :string}}},
        [description: "Rates unavailable for requested currency/date"]
      }
    ]
  )

  def rates(conn, params) do
    base = Map.get(params, "base", Currencies.canonical_base())
    symbols = parse_symbols(Map.get(params, "symbols"))

    with {:ok, date} <- parse_optional_date(Map.get(params, "date")),
         {:ok, snapshot} <- Currencies.get_rates_snapshot(base, date: date, symbols: symbols) do
      json(conn, %{
        base: snapshot.base,
        date: Date.to_iso8601(snapshot.date),
        requested_date: Date.to_iso8601(snapshot.requested_date),
        rates: snapshot.rates
      })
    else
      {:error, :invalid_date} ->
        conn |> put_status(:bad_request) |> json(%{error: "invalid_date"})

      {:error, :unknown_currency} ->
        conn |> put_status(:bad_request) |> json(%{error: "unknown_currency"})

      {:error, :rate_not_found} ->
        conn |> put_status(:not_found) |> json(%{error: "rate_not_found"})
    end
  end

  operation(:history,
    summary: "Get historical exchange rates over a date range",
    parameters: [
      base: [
        in: :query,
        schema: %{type: :string},
        required: false,
        description: "Base currency code (default: USD)"
      ],
      from: [
        in: :query,
        schema: %{type: :string, format: :date},
        required: true,
        description: "Start date (YYYY-MM-DD)"
      ],
      to: [
        in: :query,
        schema: %{type: :string, format: :date},
        required: false,
        description: "End date (YYYY-MM-DD, default: today)"
      ],
      symbols: [
        in: :query,
        schema: %{type: :string},
        required: false,
        description: "Optional comma-separated list of target currency codes"
      ]
    ],
    responses: [
      ok: {
        %{
          type: :object,
          properties: %{
            base: %{type: :string},
            from: %{type: :string, format: :date},
            to: %{type: :string, format: :date},
            rates: %{type: :object},
            count: %{type: :integer},
            available_from: %{type: :string, format: :date, nullable: true},
            available_to: %{type: :string, format: :date, nullable: true}
          }
        },
        [description: "Historical exchange rates keyed by date"]
      },
      bad_request: {
        %{type: :object, properties: %{error: %{type: :string}}},
        [description: "Invalid query parameters"]
      }
    ]
  )

  def history(conn, params) do
    base = Map.get(params, "base", Currencies.canonical_base())
    symbols = parse_symbols(Map.get(params, "symbols"))

    with {:ok, from_date} <- parse_required_date(Map.get(params, "from")),
         {:ok, to_date} <- parse_optional_date(Map.get(params, "to")),
         {:ok, history} <- Currencies.get_rates_history(base, from_date, to_date, symbols: symbols) do
      json(conn, %{
        base: history.base,
        from: Date.to_iso8601(history.from),
        to: Date.to_iso8601(history.to),
        rates: history.rates,
        count: history.count,
        available_from: history.available_from,
        available_to: history.available_to
      })
    else
      {:error, :missing_date} ->
        conn |> put_status(:bad_request) |> json(%{error: "missing_from_date"})

      {:error, :invalid_date} ->
        conn |> put_status(:bad_request) |> json(%{error: "invalid_date"})

      {:error, :invalid_date_range} ->
        conn |> put_status(:bad_request) |> json(%{error: "invalid_date_range"})

      {:error, :date_range_too_large} ->
        conn |> put_status(:bad_request) |> json(%{error: "date_range_too_large"})

      {:error, :unknown_currency} ->
        conn |> put_status(:bad_request) |> json(%{error: "unknown_currency"})
    end
  end

  operation(:convert,
    summary: "Convert an amount between two currencies on a given date",
    parameters: [
      amount: [
        in: :query,
        schema: %{type: :number},
        required: true,
        description: "Amount to convert"
      ],
      from: [
        in: :query,
        schema: %{type: :string},
        required: true,
        description: "Source currency code"
      ],
      to: [
        in: :query,
        schema: %{type: :string},
        required: true,
        description: "Target currency code"
      ],
      date: [
        in: :query,
        schema: %{type: :string, format: :date},
        required: false,
        description: "Conversion date (YYYY-MM-DD, default: today)"
      ]
    ],
    responses: [
      ok: {
        %{
          type: :object,
          properties: %{
            from: %{type: :string},
            to: %{type: :string},
            amount: %{type: :number},
            converted_amount: %{type: :number},
            rate: %{type: :number},
            rate_date: %{type: :string, format: :date},
            requested_date: %{type: :string, format: :date}
          }
        },
        [description: "Converted amount and historical rate used"]
      },
      bad_request: {
        %{type: :object, properties: %{error: %{type: :string}}},
        [description: "Invalid conversion input"]
      },
      not_found: {
        %{type: :object, properties: %{error: %{type: :string}}},
        [description: "Exchange rate unavailable"]
      }
    ]
  )

  def convert(conn, %{"amount" => amount, "from" => from_code, "to" => to_code} = params) do
    with {:ok, date} <- parse_optional_date(Map.get(params, "date")),
         {:ok, res} <- Currencies.convert(amount, from_code, to_code, date) do
      json(conn, %{
        from: res.from,
        to: res.to,
        amount: Decimal.to_float(res.amount),
        converted_amount: Decimal.to_float(res.converted_amount),
        rate: Decimal.to_float(res.rate),
        rate_date: Date.to_iso8601(res.rate_date),
        requested_date: Date.to_iso8601(res.requested_date)
      })
    else
      {:error, :invalid_date} ->
        conn |> put_status(:bad_request) |> json(%{error: "invalid_date"})

      {:error, :unknown_currency} ->
        conn |> put_status(:bad_request) |> json(%{error: "unknown_currency"})

      {:error, :invalid_conversion_input} ->
        conn |> put_status(:bad_request) |> json(%{error: "invalid_conversion_input"})

      {:error, :rate_not_found} ->
        conn |> put_status(:not_found) |> json(%{error: "rate_not_found"})
    end
  end

  def convert(conn, _params) do
    conn |> put_status(:bad_request) |> json(%{error: "missing_required_params"})
  end

  defp serialize_currency(c) do
    %{
      code: c.code,
      name: c.name,
      symbol: c.symbol,
      symbol_after: c.symbol_after,
      decimal_digits: c.decimal_digits
    }
  end

  defp parse_optional_date(nil), do: {:ok, Date.utc_today()}
  defp parse_optional_date(""), do: {:ok, Date.utc_today()}

  defp parse_optional_date(date_str) when is_binary(date_str) do
    case Date.from_iso8601(String.trim(date_str)) do
      {:ok, date} -> {:ok, date}
      {:error, _} -> {:error, :invalid_date}
    end
  end

  defp parse_optional_date(_), do: {:error, :invalid_date}

  defp parse_required_date(nil), do: {:error, :missing_date}
  defp parse_required_date(""), do: {:error, :missing_date}
  defp parse_required_date(date_str), do: parse_optional_date(date_str)

  defp parse_symbols(nil), do: nil
  defp parse_symbols(""), do: nil

  defp parse_symbols(symbols_str) when is_binary(symbols_str) do
    symbols_str
    |> String.split(",", trim: true)
    |> Enum.map(&String.trim/1)
    |> Enum.reject(&(&1 == ""))
  end

  defp parse_symbols(_), do: nil
end
