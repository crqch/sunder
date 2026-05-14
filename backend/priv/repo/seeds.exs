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


alias Sunder.Accounts.{User}
alias Sunder.Eco.{EcoUser}

case Sunder.Repo.insert(%User{
  email: "admin",
  username: "admin",
  pass: Argon2.hash_pwd_salt("admin"),
  flags: ["is_admin"]
}) do
  {:ok, user} -> Sunder.Repo.insert(%EcoUser{user_id: user.id})
  nil -> false
end
