defmodule SunderWeb.Authed.SyncController do
  use SunderWeb, :controller

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
