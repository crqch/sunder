defmodule SunderWeb.CurrencyControllerTest do
  use SunderWeb.ConnCase, async: false

  alias Sunder.Accounts.{AccessToken, User}
  alias Sunder.Contexts.Currencies
  alias Sunder.Currencies.Catalog
  alias Sunder.Eco.{Account, EcoUser, Entry}
  alias Sunder.Repo

  defmodule ControllerMockFetcher do
    @behaviour Sunder.Currencies.FetcherBehaviour

    @impl true
    def fetch_rates(_) do
      {:ok,
       %{
         date: ~D[2026-11-19],
         base: "USD",
         source: "controller_mock",
         rates: %{
           "USD" => Decimal.new("1"),
           "EUR" => Decimal.new("0.90"),
           "PLN" => Decimal.new("4.05")
         }
       }}
    end
  end

  setup do
    Catalog.sync_catalog!()
    prev_fetcher = Application.get_env(:sunder, :currency_fetcher)
    Application.put_env(:sunder, :currency_fetcher, ControllerMockFetcher)

    on_exit(fn ->
      if prev_fetcher do
        Application.put_env(:sunder, :currency_fetcher, prev_fetcher)
      else
        Application.delete_env(:sunder, :currency_fetcher)
      end
    end)

    :ok
  end

  defp auth_conn(conn, flags \\ []) do
    uniq = System.unique_integer([:positive])

    user =
      Repo.insert!(%User{
        email: "u#{uniq}@example.com",
        username: "u#{uniq}",
        pass: "hashed",
        flags: flags
      })

    eco_user = Repo.insert!(%EcoUser{user_id: user.id})
    token = "tok_#{uniq}"

    Repo.insert!(%AccessToken{
      user_id: user.id,
      token: token,
      expires_at:
        NaiveDateTime.utc_now()
        |> NaiveDateTime.add(3600, :second)
        |> NaiveDateTime.truncate(:second)
    })

    conn =
      conn
      |> put_req_header("authorization", "Bearer #{token}")
      |> put_req_header("content-type", "application/json")
      |> put_req_header("accept", "application/json")

    {conn, user, eco_user}
  end

  describe "public currency endpoints" do
    test "GET /currencies lists active currencies and GET /currencies/:code returns details", %{
      conn: conn
    } do
      resp = conn |> get("/currencies") |> json_response(200)
      codes = Enum.map(resp["currencies"], & &1["code"])
      assert "USD" in codes
      assert "EUR" in codes
      assert "PLN" in codes
      assert length(codes) >= 150

      pln = conn |> get("/currencies/pln") |> json_response(200)
      assert pln["code"] == "PLN"
      assert pln["symbol"] == "zł"
      assert pln["symbol_after"] == true
      assert pln["decimal_digits"] == 2

      assert conn |> get("/currencies/ZZZ") |> json_response(404) == %{
               "error" => "currency_not_found"
             }
    end

    test "GET /currencies/rates, /history, and /convert return historical exchange data", %{
      conn: conn
    } do
      {:ok, _} =
        Currencies.store_rates(
          ~D[2026-11-10],
          %{"EUR" => "0.90", "PLN" => "3.96"},
          "test"
        )

      {:ok, _} =
        Currencies.store_rates(
          ~D[2026-11-15],
          %{"EUR" => "0.92", "PLN" => "4.14"},
          "test"
        )

      rates_resp =
        conn
        |> get("/currencies/rates", base: "EUR", date: "2026-11-10", symbols: "PLN,USD")
        |> json_response(200)

      assert rates_resp["base"] == "EUR"
      assert rates_resp["date"] == "2026-11-10"
      assert_in_delta rates_resp["rates"]["PLN"], 4.4, 0.0001

      hist_resp =
        conn
        |> get("/currencies/rates/history",
          base: "EUR",
          from: "2026-11-10",
          to: "2026-11-15",
          symbols: "PLN"
        )
        |> json_response(200)

      assert_in_delta hist_resp["rates"]["2026-11-10"]["PLN"], 4.4, 0.0001
      assert_in_delta hist_resp["rates"]["2026-11-15"]["PLN"], 4.5, 0.0001

      conv_resp =
        conn
        |> get("/currencies/convert", amount: "100", from: "EUR", to: "PLN", date: "2026-11-15")
        |> json_response(200)

      assert conv_resp["from"] == "EUR"
      assert conv_resp["to"] == "PLN"
      assert_in_delta conv_resp["converted_amount"], 450.0, 0.0001
      assert conv_resp["rate_date"] == "2026-11-15"
    end
  end

  describe "user preferences & dashboard" do
    test "GET/PUT /dashboard/eco/preferences and GET /dashboard reflect default currency", %{
      conn: conn
    } do
      {conn, _user, _eco_user} = auth_conn(conn)

      dash1 = conn |> get("/dashboard") |> json_response(200)
      assert dash1["default_currency_code"] == "USD"

      prefs1 = conn |> get("/dashboard/eco/preferences") |> json_response(200)
      assert prefs1["default_currency_code"] == "USD"
      assert prefs1["default_currency"]["symbol"] == "$"

      prefs2 =
        conn
        |> put("/dashboard/eco/preferences", %{"default_currency_code" => "pln"})
        |> json_response(200)

      assert prefs2["default_currency_code"] == "PLN"
      assert prefs2["default_currency"]["symbol"] == "zł"

      dash2 = conn |> get("/dashboard") |> json_response(200)
      assert dash2["default_currency_code"] == "PLN"

      bad =
        conn
        |> put("/dashboard/eco/preferences", %{"default_currency_code" => "INVALID"})
        |> json_response(400)

      assert bad["error"] == "currency_not_allowed"
    end
  end

  describe "multi-currency sync and export" do
    test "sync stores entries in different currencies, defaults missing currency_code, and rejects disallowed currencies",
         %{conn: conn} do
      {conn, _user, eco_user} = auth_conn(conn)
      {:ok, _} = Currencies.set_user_default_currency(eco_user, "PLN")

      acc_id = ExCuid2.generate()
      cat_id = ExCuid2.generate()
      entry_eur_id = ExCuid2.generate()
      entry_default_id = ExCuid2.generate()
      now = NaiveDateTime.utc_now() |> NaiveDateTime.truncate(:second) |> NaiveDateTime.to_iso8601()

      sync_resp =
        conn
        |> post("/dashboard/eco/sync?last=1970-01-01T00:00:00", %{
          accounts: [%{id: acc_id, name: "Multi-Currency Card", currency_code: "EUR"}],
          categories: [%{id: cat_id, title: "Travel", color: "#3b82f6", description: ""}],
          entries: [
            %{
              id: entry_eur_id,
              title: "Train ticket",
              amount: -25.0,
              currency_code: "eur",
              date: now,
              account_id: acc_id,
              category_id: cat_id,
              description: "",
              location: ""
            },
            %{
              id: entry_default_id,
              title: "Local coffee",
              amount: -18.5,
              date: now,
              account_id: acc_id,
              category_id: cat_id,
              description: "",
              location: ""
            }
          ]
        })
        |> json_response(200)

      assert sync_resp["default_currency_code"] == "PLN"

      saved_eur = Repo.get!(Entry, entry_eur_id)
      saved_default = Repo.get!(Entry, entry_default_id)
      saved_acc = Repo.get!(Account, acc_id)

      assert saved_eur.currency_code == "EUR"
      assert saved_default.currency_code == "PLN"
      assert saved_acc.currency_code == "EUR"

      # Reject unknown currency on sync
      bad_sync =
        conn
        |> post("/dashboard/eco/sync?last=1970-01-01T00:00:00", %{
          accounts: [],
          categories: [],
          entries: [
            %{
              id: ExCuid2.generate(),
              title: "Bad",
              amount: -10.0,
              currency_code: "NOPE",
              date: now,
              account_id: acc_id,
              category_id: cat_id
            }
          ]
        })
        |> json_response(400)

      assert bad_sync["error"] == "currency_not_allowed"

      # Export includes currency_code and default_currency_code
      export_resp = conn |> get("/dashboard/eco/data/export") |> json_response(200)
      assert export_resp["default_currency_code"] == "PLN"
      assert Enum.any?(export_resp["entries"], &(&1["currency_code"] == "EUR"))
      assert Enum.any?(export_resp["entries"], &(&1["currency_code"] == "PLN"))
    end
  end

  describe "admin currency management and OpenAPI spec" do
    test "admin can create, disable, manually rate, and trigger rate sync", %{conn: conn} do
      {conn, _user, _eco_user} = auth_conn(conn, ["is_admin"])

      created =
        conn
        |> post("/admin/currencies", %{
          code: "CUSTOM",
          name: "Custom Token",
          symbol: "CTK",
          symbol_after: true,
          decimal_digits: 2
        })
        |> json_response(201)

      assert created["currency"]["code"] == "CUSTOM"

      updated =
        conn
        |> put("/admin/currencies/CUSTOM", %{is_active: false})
        |> json_response(200)

      assert updated["currency"]["is_active"] == false

      # Inactive currency hidden from public GET /currencies
      public = conn |> get("/currencies") |> json_response(200)
      refute Enum.any?(public["currencies"], &(&1["code"] == "CUSTOM"))

      # Visible in admin list
      admin_list = conn |> get("/admin/currencies") |> json_response(200)
      assert Enum.any?(admin_list["currencies"], &(&1["code"] == "CUSTOM"))

      # Trigger rate sync via admin endpoint
      sync = conn |> post("/admin/currencies/sync-rates") |> json_response(200)
      assert sync["success"] == true
      assert sync["source"] == "controller_mock"
      assert sync["stored_count"] == 3

      # Manual rate override
      manual =
        conn
        |> put("/admin/currencies/rates", %{
          base_currency_code: "USD",
          target_currency_code: "PLN",
          rate: "4.20",
          rate_date: "2026-11-18"
        })
        |> json_response(200)

      assert manual["success"] == true
      assert_in_delta manual["rate"]["rate"], 4.2, 0.0001
    end

    test "OpenAPI spec includes currency and preferences endpoints", %{conn: conn} do
      spec = conn |> get("/api/openapi.json") |> json_response(200)
      paths = Map.keys(spec["paths"])

      assert "/currencies" in paths
      assert "/currencies/rates" in paths
      assert "/currencies/rates/history" in paths
      assert "/currencies/convert" in paths
      assert "/dashboard/eco/preferences" in paths
    end
  end
end
