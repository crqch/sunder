defmodule SunderWeb.AdminController do
  use SunderWeb, :controller
  import SunderWeb.ApiMacros
  import Ecto.Query
  alias Sunder.Repo
  alias Sunder.Accounts.{User, Invite}

  def list_users(conn, _params) do
    users = Repo.all(from u in User, select: %{id: u.id, username: u.username, email: u.email, flags: u.flags})
    json(conn, %{users: users})
  end

  def delete_user(conn, %{"id" => id}) do
    case Repo.get(User, id) do
      nil ->
        conn |> put_status(404) |> json(%{error: "Not found"})
      user ->
        Repo.delete!(user)
        json(conn, %{success: true})
    end
  end

  def update_user(conn, %{"id" => id} = params) do
    user_params = if params["pass"] in [nil, ""], do: Map.delete(params, "pass"), else: params

    case Repo.get(User, id) do
      nil ->
        conn |> put_status(404) |> json(%{error: "Not found"})
      user ->
        user
        |> User.changeset(user_params)
        |> Repo.update()
        |> case do
          {:ok, updated_user} -> 
             json(conn, %{success: true, user: %{id: updated_user.id, username: updated_user.username, email: updated_user.email, flags: updated_user.flags}})
          {:error, _changeset} -> 
             conn |> put_status(400) |> json(%{error: "Invalid input"})
        end
    end
  end

  def list_invites(conn, _params) do
    invites = Repo.all(from i in Invite, select: %{id: i.id, token: i.token, used_by: i.used_by})
    json(conn, %{invites: invites})
  end

  def create_invite(conn, %{"token" => token}) do
    %Invite{}
    |> Invite.changeset(%{token: token})
    |> Repo.insert()
    |> case do
      {:ok, invite} -> json(conn, %{id: invite.id, token: invite.token, used_by: invite.used_by})
      {:error, _} -> conn |> put_status(400) |> json(%{error: "Invalid token"})
    end
  end

  def delete_invite(conn, %{"id" => id}) do
    case Repo.get(Invite, id) do
      nil ->
        conn |> put_status(404) |> json(%{error: "Not found"})
      invite ->
        Repo.delete!(invite)
        json(conn, %{success: true})
    end
  end

  authed_operation(:list_currencies,
    summary: "Admin: list all currencies (active and inactive)",
    responses: [
      ok: {%{type: :object, properties: %{currencies: %{type: :array, items: %{type: :object}}}}, [description: "All server currencies"]}
    ]
  )

  def list_currencies(conn, _params) do
    currencies = Sunder.Contexts.Currencies.list_all_currencies()
    json(conn, %{currencies: currencies})
  end

  authed_operation(:create_currency,
    summary: "Admin: add a new supported currency",
    request_body: {
      %{
        type: :object,
        properties: %{
          code: %{type: :string},
          name: %{type: :string},
          symbol: %{type: :string},
          symbol_after: %{type: :boolean},
          decimal_digits: %{type: :integer},
          is_active: %{type: :boolean}
        }
      },
      [required: true, description: "Currency definition"]
    },
    responses: [
      created: {%{type: :object, properties: %{success: %{type: :boolean}, currency: %{type: :object}}}, [description: "Created currency"]},
      bad_request: {%{type: :object, properties: %{error: %{type: :string}}}, [description: "Validation error"]}
    ]
  )

  def create_currency(conn, params) do
    case Sunder.Contexts.Currencies.create_currency(params) do
      {:ok, currency} ->
        conn |> put_status(201) |> json(%{success: true, currency: currency})

      {:error, changeset} ->
        errors = Ecto.Changeset.traverse_errors(changeset, fn {msg, _opts} -> msg end)
        conn |> put_status(400) |> json(%{error: "Invalid currency", details: errors})
    end
  end

  authed_operation(:update_currency,
    summary: "Admin: update currency metadata or enable/disable it",
    parameters: [
      code: [in: :path, schema: %{type: :string}, required: true, description: "Currency code"]
    ],
    request_body: {
      %{
        type: :object,
        properties: %{
          name: %{type: :string},
          symbol: %{type: :string},
          symbol_after: %{type: :boolean},
          decimal_digits: %{type: :integer},
          is_active: %{type: :boolean}
        }
      },
      [required: true, description: "Fields to update"]
    },
    responses: [
      ok: {%{type: :object, properties: %{success: %{type: :boolean}, currency: %{type: :object}}}, [description: "Updated currency"]},
      bad_request: {%{type: :object, properties: %{error: %{type: :string}}}, [description: "Validation error"]},
      not_found: {%{type: :object, properties: %{error: %{type: :string}}}, [description: "Not found"]}
    ]
  )

  def update_currency(conn, %{"code" => code} = params) do
    case Sunder.Contexts.Currencies.get_currency(code) do
      nil ->
        conn |> put_status(404) |> json(%{error: "Not found"})

      currency ->
        case Sunder.Contexts.Currencies.update_currency(currency, params) do
          {:ok, updated} ->
            json(conn, %{success: true, currency: updated})

          {:error, changeset} ->
            errors = Ecto.Changeset.traverse_errors(changeset, fn {msg, _opts} -> msg end)
            conn |> put_status(400) |> json(%{error: "Invalid currency", details: errors})
        end
    end
  end

  authed_operation(:sync_rates,
    summary: "Admin: trigger exchange rate sync (latest, specific date, or backfill)",
    parameters: [
      date: [in: :query, schema: %{type: :string, format: :date}, required: false, description: "Optional specific YYYY-MM-DD date"],
      days: [in: :query, schema: %{type: :integer}, required: false, description: "Optional number of trailing days to backfill (1..366)"],
      force: [in: :query, schema: %{type: :boolean}, required: false, description: "Force re-fetch of existing dates during backfill"]
    ],
    responses: [
      ok: {%{type: :object, properties: %{success: %{type: :boolean}}}, [description: "Sync summary"]},
      bad_request: {%{type: :object, properties: %{error: %{type: :string}}}, [description: "Invalid parameters"]},
      bad_gateway: {%{type: :object, properties: %{error: %{type: :string}}}, [description: "Upstream provider failure"]}
    ]
  )

  def sync_rates(conn, params) do
    cond do
      is_binary(params["date"]) and params["date"] != "" ->
        case Date.from_iso8601(params["date"]) do
          {:ok, date} ->
            case Sunder.Contexts.Currencies.sync_rates_for_date(date) do
              {:ok, result} ->
                json(conn, %{
                  success: true,
                  date: Date.to_iso8601(result.date),
                  fetched_date: Date.to_iso8601(result.fetched_date),
                  source: result.source,
                  stored_count: result.stored_count
                })

              {:error, reason} ->
                conn |> put_status(502) |> json(%{error: inspect(reason)})
            end

          {:error, _} ->
            conn |> put_status(400) |> json(%{error: "invalid_date"})
        end

      params["days"] != nil ->
        case parse_positive_int(params["days"]) do
          {:ok, days} when days <= 366 ->
            to_date = Date.utc_today()
            from_date = Date.add(to_date, -(days - 1))
            force = params["force"] in [true, "true", "1", 1]
            {:ok, stats} = Sunder.Contexts.Currencies.backfill_rates(from_date, to_date, force: force)
            json(conn, %{success: true, from: Date.to_iso8601(from_date), to: Date.to_iso8601(to_date), stats: stats})

          _ ->
            conn |> put_status(400) |> json(%{error: "invalid_days"})
        end

      true ->
        case Sunder.Contexts.Currencies.sync_latest_rates() do
          {:ok, result} ->
            json(conn, %{
              success: true,
              date: Date.to_iso8601(result.date),
              source: result.source,
              stored_count: result.stored_count
            })

          {:error, reason} ->
            conn |> put_status(502) |> json(%{error: inspect(reason)})
        end
    end
  end

  authed_operation(:upsert_rate,
    summary: "Admin: manually set an exchange rate for a date",
    request_body: {
      %{
        type: :object,
        properties: %{
          base_currency_code: %{type: :string},
          target_currency_code: %{type: :string},
          rate: %{anyOf: [%{type: :number}, %{type: :string}]},
          rate_date: %{type: :string, format: :date}
        }
      },
      [required: true, description: "Manual rate payload"]
    },
    responses: [
      ok: {%{type: :object, properties: %{success: %{type: :boolean}, rate: %{type: :object}}}, [description: "Upserted rate"]},
      bad_request: {%{type: :object, properties: %{error: %{type: :string}}}, [description: "Invalid rate input"]}
    ]
  )

  def upsert_rate(conn, %{
        "target_currency_code" => target_code,
        "rate" => rate
      } = params) do
    base_code = Map.get(params, "base_currency_code", Sunder.Contexts.Currencies.canonical_base())

    date_result =
      case Map.get(params, "rate_date") do
        nil -> {:ok, Date.utc_today()}
        "" -> {:ok, Date.utc_today()}
        d when is_binary(d) -> Date.from_iso8601(d)
        _ -> {:error, :invalid_date}
      end

    with {:ok, rate_date} <- date_result,
         {:ok, stored} <- Sunder.Contexts.Currencies.upsert_manual_rate(base_code, target_code, rate, rate_date) do
      json(conn, %{
        success: true,
        rate: %{
          id: stored.id,
          base_currency_code: stored.base_currency_code,
          target_currency_code: stored.target_currency_code,
          rate: Decimal.to_float(stored.rate),
          rate_date: Date.to_iso8601(stored.rate_date),
          source: stored.source
        }
      })
    else
      {:error, reason} ->
        conn |> put_status(400) |> json(%{error: to_string(reason)})
    end
  end

  def upsert_rate(conn, _params) do
    conn |> put_status(400) |> json(%{error: "missing_required_fields"})
  end

  defp parse_positive_int(n) when is_integer(n) and n > 0, do: {:ok, n}

  defp parse_positive_int(n) when is_binary(n) do
    case Integer.parse(n) do
      {val, ""} when val > 0 -> {:ok, val}
      _ -> :error
    end
  end

  defp parse_positive_int(_), do: :error
end
