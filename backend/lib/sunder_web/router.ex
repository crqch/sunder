defmodule SunderWeb.Router do
  use SunderWeb, :router

  pipeline :browser do
    plug(:accepts, ["html"])
    plug(:fetch_session)
    plug(:fetch_flash)
    plug(:protect_from_forgery)
    plug(:put_secure_browser_headers)
  end

  pipeline :api do
    plug(:accepts, ["json"])
    plug(Oaskit.Plugs.SpecProvider, spec: SunderWeb.ApiSpec)
  end

  scope "/api" do
    pipe_through(:api)

    get("/openapi.json", Oaskit.SpecController, spec: SunderWeb.ApiSpec)
    get("/docs", Oaskit.SpecController, redoc: "/api/openapi.json")
  end

  scope "/", SunderWeb do
    pipe_through(:api)

    get("/", StatusController, :index)

    post("/auth/register", AuthController, :register)
    post("/auth/login", AuthController, :login)
    post("/auth/refresh_token", AuthController, :refresh_token)
  end

  scope "/dashboard", SunderWeb.Authed do
    pipe_through(:api)
    pipe_through(SunderWeb.Plugs.AuthPlug)

    get("/", DashboardController, :index)
  end

  scope "/dashboard/eco", SunderWeb.Authed do
    pipe_through(:api)
    pipe_through(SunderWeb.Plugs.AuthPlug)
    pipe_through(SunderWeb.Plugs.EcoUserPlug)

    post("/sync", SunderWeb.Authed.SyncController, :sync)
  end

  # Other scopes may use custom stacks.
  # scope "/api", SunderWeb do
  #   pipe_through :api
  # end

  # Enables LiveDashboard only for development
  #
  # If you want to use the LiveDashboard in production, you should put
  # it behind authentication and allow only admins to access it.
  # If your application does not have an admins-only section yet,
  # you can use Plug.BasicAuth to set up some basic authentication
  # as long as you are also using SSL (which you should anyway).
  if Mix.env() in [:dev, :test] do
    import Phoenix.LiveDashboard.Router

    scope "/" do
      pipe_through(:browser)
      live_dashboard("/dashboard", metrics: SunderWeb.Telemetry)
    end
  end
end
