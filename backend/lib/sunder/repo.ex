defmodule Sunder.Repo do
  use Ecto.Repo,
    otp_app: :sunder,
    adapter: Ecto.Adapters.Postgres
end
