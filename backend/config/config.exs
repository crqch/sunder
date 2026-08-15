# This file is responsible for configuring your application
# and its dependencies with the aid of the Mix.Config module.
#
# This configuration file is loaded before any dependency and
# is restricted to this project.

import Config
# General application configuration

config :sunder,
  ecto_repos: [Sunder.Repo]

config :sunder, Sunder.Repo, migration_primary_key: [name: :id, type: :string]

# Configures the endpoint
config :sunder, SunderWeb.Endpoint,
  url: [host: "localhost"],
  secret_key_base: "ZAAn2DV5odejNJkR0QjHk3EVBbIYv3rzZw5BVzebypfSb2IkjElP8iqn0ZwdKueG",
  render_errors: [view: SunderWeb.ErrorView, accepts: ~w(html json), layout: false],
  pubsub_server: Sunder.PubSub,
  live_view: [signing_salt: "EYslyk0K"],
  cors_origins: ["*"]


# Configures Elixir's Logger
config :logger, :console,
  format: "$time $metadata[$level] $message\n",
  metadata: [:request_id]

# Use Jason for JSON parsing in Phoenix
config :phoenix, :json_library, Jason

# Import environment specific config. This must remain at the bottom
# of this file so it overrides the configuration defined above.
import_config "#{Mix.env()}.exs"
