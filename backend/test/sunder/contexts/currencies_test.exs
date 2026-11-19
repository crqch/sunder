defmodule Sunder.Contexts.CurrenciesTest do
  use Sunder.DataCase, async: false

  alias Sunder.Accounts.User
  alias Sunder.Contexts.Currencies
  alias Sunder.Currencies.Catalog
  alias Sunder.Eco.EcoUser

  defmodule MockFetcher do
    @behaviour Sunder.Currencies.FetcherBehaviour

    @impl true
    def fetch_rates(:latest) do
      {:ok,
       %{
         date: ~D[2026-11-19],
         base: "USD",
         source: "mock_latest",
         rates: %{
           "USD" => Decimal.new("1"),
           "EUR" => Decimal.new("0.92"),
           "PLN" => Decimal.new("4.05"),
           "JPY" => Decimal.new("154.25"),
           "NONEXISTENT_TICKER" => Decimal.new("999.0")
         }
       }}
    end

    @impl true
    def fetch_rates(%Date{} = date) do
      case date do
        ~D[2026-11-10] ->
          {:ok,
           %{
             date: ~D[2026-11-10],
             base: "USD",
             source: "mock_historical",
             rates: %{
               "USD" => Decimal.new("1"),
               "EUR" => Decimal.new("0.90"),
               "PLN" => Decimal.new("3.96")
             }
           }}

        ~D[2026-11-11] ->
          {:ok,
           %{
             date: ~D[2026-11-11],
             base: "USD",
             source: "mock_historical",
             rates: %{
               "USD" => Decimal.new("1"),
               "EUR" => Decimal.new("0.91"),
               "PLN" => Decimal.new("4.00")
             }
           }}

        _ ->
          {:error, :not_available}
      end
    end
  end

  setup do
    Catalog.sync_catalog!()
    prev_fetcher = Application.get_env(:sunder, :currency_fetcher)
    Application.put_env(:sunder, :currency_fetcher, MockFetcher)

    on_exit(fn ->
      if prev_fetcher do
        Application.put_env(:sunder, :currency_fetcher, prev_fetcher)
      else
        Application.delete_env(:sunder, :currency_fetcher)
      end
    end)

    :ok
  end

  defp create_eco_user do
    user =
      Repo.insert!(%User{
        email: "user_#{System.unique_integer([:positive])}@example.com",
        username: "user_#{System.unique_integer([:positive])}",
        pass: "hashed",
        flags: []
      })

    Repo.insert!(%EcoUser{user_id: user.id})
  end

  describe "currency catalog & registry" do
    test "seeds 150+ active world ISO 4217 currencies with metadata" do
      active = Currencies.list_active_currencies()
      assert length(active) >= 150

      usd = Currencies.get_currency("usd")
      assert usd.code == "USD"
      assert usd.symbol == "$"
      assert usd.symbol_after == false
      assert usd.decimal_digits == 2

      pln = Currencies.get_currency("PLN")
      assert pln.symbol == "zł"
      assert pln.symbol_after == true
      assert pln.decimal_digits == 2

      jpy = Currencies.get_currency("JPY")
      assert jpy.decimal_digits == 0

      kwd = Currencies.get_currency("KWD")
      assert kwd.decimal_digits == 3
    end

    test "create and deactivate currency dynamically on server" do
      {:ok, created} =
        Currencies.create_currency(%{
          code: "xts",
          name: "Test Custom Currency",
          symbol: "TST",
          symbol_after: true,
          decimal_digits: 2
        })

      assert created.code == "XTS"
      assert Currencies.currency_allowed?("XTS")

      {:ok, disabled} = Currencies.update_currency(created, %{is_active: false})
      assert disabled.is_active == false
      refute Currencies.currency_allowed?("XTS")
      assert Currencies.get_currency("XTS") != nil
      assert Currencies.get_active_currency("XTS") == nil
    end
  end

  describe "user default currency preferences" do
    test "defaults to USD and allows switching to any active currency" do
      eco_user = create_eco_user()
      assert eco_user.default_currency_code == "USD"

      {:ok, %{eco_user: updated, currency: pln}} =
        Currencies.set_user_default_currency(eco_user, "pln")

      assert updated.default_currency_code == "PLN"
      assert pln.code == "PLN"
      assert Currencies.get_user_default_currency(updated).code == "PLN"
    end

    test "rejects inactive or unknown currency" do
      eco_user = create_eco_user()
      assert {:error, :currency_not_allowed} = Currencies.set_user_default_currency(eco_user, "ZZZ")

      eur = Currencies.get_currency("EUR")
      {:ok, _} = Currencies.update_currency(eur, %{is_active: false})
      assert {:error, :currency_not_allowed} = Currencies.set_user_default_currency(eco_user, "EUR")
    end
  end

  describe "exchange rate sync, historical lookup, and conversion" do
    test "sync_latest_rates ignores unknown upstream tickers and stores known currencies" do
      assert {:ok, %{date: ~D[2026-11-19], source: "mock_latest", stored_count: 4}} =
               Currencies.sync_latest_rates()

      assert Currencies.has_rates_for_date?(~D[2026-11-19])
      refute Currencies.has_rates_for_date?(~D[2026-11-01])
    end

    test "uses historical rate on exact date and falls back to most recent earlier date" do
      {:ok, _} = Currencies.sync_rates_for_date(~D[2026-11-10])
      {:ok, _} = Currencies.sync_rates_for_date(~D[2026-11-11])

      # Exact date: 2026-11-10 (1 USD = 0.90 EUR = 3.96 PLN -> 1 EUR = 4.40 PLN)
      {:ok, conv_nov10} = Currencies.convert(100, "EUR", "PLN", ~D[2026-11-10])
      assert Decimal.equal?(conv_nov10.rate, Decimal.new("4.4"))
      assert Decimal.equal?(conv_nov10.converted_amount, Decimal.new("440.00"))
      assert conv_nov10.rate_date == ~D[2026-11-10]

      # Later date with no direct record (2026-11-15) uses 2026-11-11 rate
      {:ok, rate_nov15} = Currencies.get_rate("USD", "PLN", ~D[2026-11-15])
      assert Decimal.equal?(rate_nov15.rate, Decimal.new("4.0"))
      assert rate_nov15.rate_date == ~D[2026-11-11]
      assert rate_nov15.requested_date == ~D[2026-11-15]

      # Date earlier than all history falls back to earliest available date (2026-11-10)
      {:ok, rate_early} = Currencies.get_rate("USD", "EUR", ~D[2026-01-01])
      assert Decimal.equal?(rate_early.rate, Decimal.new("0.9"))
      assert rate_early.rate_date == ~D[2026-11-10]
    end

    test "rounds converted amount to target currency decimal_digits (e.g. JPY = 0)" do
      {:ok, _} = Currencies.sync_latest_rates()
      {:ok, conv} = Currencies.convert("10.00", "USD", "JPY", ~D[2026-11-19])
      assert Decimal.equal?(conv.converted_amount, Decimal.new("1543"))
    end

    test "same currency conversion returns 1.0 without needing stored rates" do
      {:ok, conv} = Currencies.convert(42.5, "PLN", "PLN", ~D[2026-11-19])
      assert Decimal.equal?(conv.rate, Decimal.new("1"))
      assert Decimal.equal?(conv.converted_amount, Decimal.new("42.50"))
    end

    test "get_rates_snapshot rebases to requested base and filters symbols" do
      {:ok, _} = Currencies.sync_rates_for_date(~D[2026-11-10])

      {:ok, snap} =
        Currencies.get_rates_snapshot("EUR", date: ~D[2026-11-10], symbols: ["PLN", "USD"])

      assert snap.base == "EUR"
      assert snap.date == ~D[2026-11-10]
      assert Map.keys(snap.rates) |> Enum.sort() == ["PLN", "USD"]
      assert_in_delta snap.rates["PLN"], 4.4, 0.0001
      assert_in_delta snap.rates["USD"], 1.111111, 0.0001
    end

    test "get_rates_history returns daily series rebased to requested currency" do
      {:ok, _} = Currencies.backfill_rates(~D[2026-11-10], ~D[2026-11-11])

      {:ok, hist} =
        Currencies.get_rates_history("EUR", ~D[2026-11-10], ~D[2026-11-11], symbols: ["PLN"])

      assert hist.base == "EUR"
      assert_in_delta hist.rates["2026-11-10"]["PLN"], 4.4, 0.0001
      assert_in_delta hist.rates["2026-11-11"]["PLN"], 4.0 / 0.91, 0.0001
    end

    test "upsert_manual_rate supports USD base and cross-pair" do
      {:ok, r1} = Currencies.upsert_manual_rate("USD", "EUR", "0.80", ~D[2026-11-12])
      assert Decimal.equal?(r1.rate, Decimal.new("0.8"))

      {:ok, r2} = Currencies.upsert_manual_rate("EUR", "PLN", "5.00", ~D[2026-11-12])
      assert Decimal.equal?(r2.rate, Decimal.new("4.0"))
    end
  end
end
