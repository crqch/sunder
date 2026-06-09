defmodule SunderWeb.Authed.SyncController do
  import SunderWeb.ApiMacros
  use SunderWeb, :controller

  authed_operation(:sync,
    summary: "Sync data",
    parameters: [
      last: [
        in: :query,
        schema: %{type: :string},
        required: true,
        description: "Last sync timestamp"
      ]
    ],
    request_body: {
      %{
        type: :object,
        properties: %{
          accounts: %{type: :array, items: %{type: :object}},
          categories: %{type: :array, items: %{type: :object}},
          entries: %{type: :array, items: %{type: :object}}
        }
      },
      [required: true, description: "Transactions to sync"]
    },
    responses: [
      ok: {
        %{
          type: :object,
          properties: %{
            accounts: %{
              type: :array,
              items: %{
                type: :object,
                properties: %{
                  id: %{type: :string},
                  name: %{type: :string},
                  eco_user_id: %{type: :string},
                  deleted_at: %{type: :string, format: :"date-time", nullable: true},
                  inserted_at: %{type: :string, format: :"date-time"},
                  updated_at: %{type: :string, format: :"date-time"}
                }
              }
            },
            categories: %{
              type: :array,
              items: %{
                type: :object,
                properties: %{
                  id: %{type: :string},
                  title: %{type: :string},
                  description: %{type: :string},
                  color: %{type: :string},
                  eco_user_id: %{type: :string},
                  deleted_at: %{type: :string, format: :"date-time", nullable: true},
                  inserted_at: %{type: :string, format: :"date-time"},
                  updated_at: %{type: :string, format: :"date-time"}
                }
              }
            },
            entries: %{
              type: :array,
              items: %{
                type: :object,
                properties: %{
                  id: %{type: :string},
                  date: %{type: :string, format: :"date-time"},
                  amount: %{type: :number},
                  title: %{type: :string},
                  description: %{type: :string},
                  location: %{type: :string},
                  account_id: %{type: :string},
                  category_id: %{type: :string},
                  eco_user_id: %{type: :string},
                  deleted_at: %{type: :string, format: :"date-time", nullable: true},
                  inserted_at: %{type: :string, format: :"date-time"},
                  updated_at: %{type: :string, format: :"date-time"}
                }
              }
            }
          }
        },
        [description: "Synced data"]
      },
      bad_request: {
        %{
          type: :object,
          properties: %{
            error: %{type: :string}
          }
        },
        [description: "Bad request"]
      },
      internal_server_error: {
        %{
          type: :object,
          properties: %{
            error: %{type: :string}
          }
        },
        [description: "Internal server error"]
      }
    ]
  )
  def sync(%{assigns: %{eco_user: eco_user, user: user}} = conn, %{"last" => timestamp} = params) do
    case SunderWeb.Sync.Pipeline.process(%{eco_user: eco_user, user: user}, params, timestamp) do
      {:ok, data} ->
        conn |> put_status(200) |> json(data.reader)

      {:error, reason} ->
        conn |> put_status(500) |> json(%{error: reason})

      {:error, :guard, :txs_invalid_format, _} ->
        conn |> put_status(400) |> json(%{error: "txs_invalid_format"})
    end
  end

  def sync(conn, _params) do
    conn |> put_status(400) |> json(%{error: "missing last timestamp"})
  end
end
