# Script for populating the database. You can run it as:
#
#     mix run priv/repo/seeds.exs
#
# Inside the script, you can read and write to any of your
# repositories directly:
#
#     Sunder.Repo.insert!(%Sunder.SomeSchema{})
#
# We recommend using the bang functions (`insert!`, `update!`
# and so on) as they will fail if something goes wrong.

Sunder.Repo.insert!(%Sunder.Accounts.User{
  email: "admin",
  username: "admin",
  pass: Argon2.hash_pwd_salt("admin"),
})


with {:ok, user} <- Sunder.Repo.get_by(Sunder.Accounts.User, email: "admin"),
     do: Sunder.Repo.update!(user, set: %{flags: {:is_admin}})
