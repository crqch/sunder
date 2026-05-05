defmodule SunderWeb.ApiSpec do
  alias Oaskit.Spec.Paths
  alias Oaskit.Spec.Server
  use Oaskit

  @impl true
  def spec do
    %{
      openapi: "3.1.1",
      info: %{
        title: "Sunder API",
        version: "1.0"
      },
      servers: [
        %{url: "http://localhost:4000", description: "Local Development Server"}
      ],
      paths: Paths.from_router(SunderWeb.Router)
    }
  end
end
