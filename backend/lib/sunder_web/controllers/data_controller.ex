defmodule SunderWeb.DataController do
  use SunderWeb, :controller
  import SunderWeb.ApiMacros
  alias Sunder.Eco.{Account, Category, Entry}
  alias Sunder.Repo
  import Ecto.Query

  authed_operation(:export,
    summary: "Export all user data as JSON",
    responses: [
      ok: {
        %{type: :object},
        [description: "Sunder export data"]
      }
    ]
  )
  def export(%{assigns: %{eco_user: eco_user}} = conn, _params) do
    accounts = Repo.all(from a in Account, where: a.eco_user_id == ^eco_user.id and is_nil(a.deleted_at))
    categories = Repo.all(from c in Category, where: c.eco_user_id == ^eco_user.id and is_nil(c.deleted_at))
    entries = Repo.all(from e in Entry, where: e.eco_user_id == ^eco_user.id and is_nil(e.deleted_at))

    clean_accounts = Enum.map(accounts, fn a -> %{id: a.id, name: a.name} end)
    clean_categories = Enum.map(categories, fn c -> %{id: c.id, title: c.title, color: c.color, description: c.description} end)
    clean_entries = Enum.map(entries, fn e -> 
      %{
        date: e.date,
        amount: e.amount,
        title: e.title,
        description: e.description,
        location: e.location,
        account_id: e.account_id,
        category_id: e.category_id
      }
    end)

    data = %{
      version: 1,
      format: "sunder",
      accounts: clean_accounts,
      categories: clean_categories,
      entries: clean_entries
    }

    conn
    |> put_resp_content_type("application/json")
    |> put_resp_header("content-disposition", "attachment; filename=\"sunder_export.json\"")
    |> send_resp(200, Jason.encode!(data))
  end

  authed_operation(:import,
    summary: "Import Sunder JSON data",
    request_body: {
      "multipart/form-data",
      %{
        type: :object,
        properties: %{
          file: %{type: :string, format: :binary}
        }
      },
      [required: true, description: "Sunder JSON file"]
    },
    responses: [
      ok: {
        %{
          type: :object,
          properties: %{
            success: %{type: :boolean},
            stats: %{
              type: :object,
              properties: %{
                accounts: %{type: :integer},
                categories: %{type: :integer},
                entries: %{type: :integer}
              }
            }
          }
        },
        [description: "Import successful"]
      },
      bad_request: {%{type: :object, properties: %{error: %{type: :string}}}, [description: "Invalid file"]}
    ]
  )
  def import(%{assigns: %{eco_user: eco_user}} = conn, %{"file" => %Plug.Upload{path: path}}) do
    case File.read(path) do
      {:ok, content} ->
        case Jason.decode(content) do
          {:ok, %{"format" => "sunder"} = data} ->
            stats = Sunder.Eco.SunderImporter.import_data(eco_user.id, data)
            json(conn, %{success: true, stats: stats})
          _ ->
            conn |> put_status(400) |> json(%{error: "Invalid file format"})
        end
      _ ->
        conn |> put_status(400) |> json(%{error: "File read error"})
    end
  end

  authed_operation(:import_bagels,
    summary: "Import Bagels SQLite data",
    request_body: {
      "multipart/form-data",
      %{
        type: :object,
        properties: %{
          file: %{type: :string, format: :binary}
        }
      },
      [required: true, description: "Bagels SQLite database file"]
    },
    responses: [
      ok: {
        %{
          type: :object,
          properties: %{
            success: %{type: :boolean},
            stats: %{
              type: :object,
              properties: %{
                accounts: %{type: :integer},
                categories: %{type: :integer},
                entries: %{type: :integer}
              }
            }
          }
        },
        [description: "Import successful"]
      },
      internal_server_error: {%{type: :object, properties: %{error: %{type: :string}}}, [description: "Import failed"]}
    ]
  )
  def import_bagels(%{assigns: %{eco_user: eco_user}} = conn, %{"file" => %Plug.Upload{path: path}}) do
    case Sunder.Eco.BagelsImporter.import_db(eco_user.id, path) do
      {:ok, stats} -> json(conn, %{success: true, stats: stats})
      _ -> conn |> put_status(500) |> json(%{error: "Import failed"})
    end
  end
end
