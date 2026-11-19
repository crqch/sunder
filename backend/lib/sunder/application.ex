defmodule Sunder.Application do
  # See https://hexdocs.pm/elixir/Application.html
  # for more information on OTP Applications
  @moduledoc false

  use Application

  def start(_type, _args) do
    poller_enabled =
      Application.get_env(:sunder, Sunder.Currencies.Poller, [])
      |> Keyword.get(:enabled, true)

    children =
      [
        # Start the Ecto repository
        Sunder.Repo,
        # Start the Telemetry supervisor
        SunderWeb.Telemetry,
        # Start the PubSub system
        {Phoenix.PubSub, name: Sunder.PubSub},
        # Start the Endpoint (http/https)
        SunderWeb.Endpoint
      ] ++
        if(poller_enabled, do: [Sunder.Currencies.Poller], else: [])

    # See https://hexdocs.pm/elixir/Supervisor.html
    # for other strategies and supported options
    opts = [strategy: :one_for_one, name: Sunder.Supervisor]
    Supervisor.start_link(children, opts)
  end

  # Tell Phoenix to update the endpoint configuration
  # whenever the application is updated.
  def config_change(changed, _new, removed) do
    SunderWeb.Endpoint.config_change(changed, removed)
    :ok
  end
end
