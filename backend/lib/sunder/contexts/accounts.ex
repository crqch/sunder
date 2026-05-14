defmodule SunderWeb.Contexts.Accounts do
  import Ecto.Query

  alias Sunder.Eco.Account
  alias Sunder.Repo

  def accounts_of_user(eco_user) do
    Repo.all(
      from(
        a in Account,
        where: a.eco_user_id == ^eco_user.id
      )
    )
  end

  def account(eco_user, id) do
    Repo.one(
      from(
        a in Account,
        where: a.eco_user_id == ^eco_user.id and a.id == ^id
      )
    )
  end

  def create(eco_user, data) do
    Ecto.Multi.new()
    |> Ecto.Multi.insert(:account, fn _changes ->
      Account.create_changeset(%Account{eco_user_id: eco_user.id}, data)
    end)
    |> Repo.transact()
  end

  def update(eco_user, data) do
    Ecto.Multi.new()
    |> Ecto.Multi.run(:account, fn repo, _changes ->
      case repo.one(
             from(
               a in Account,
               where: a.eco_user_id == ^eco_user.id and a.id == ^data.id
             )
           ) do
        nil -> {:error, :account_not_found}
        account -> {:ok, account}
      end
    end)
    |> Ecto.Multi.update(:update_account, fn %{account: account} ->
      Account.update_changeset(account, data)
    end)
    |> Repo.transact()
  end

  def delete(eco_user, id) do
    Ecto.Multi.new()
    |> Ecto.Multi.run(:account, fn repo, _changes ->
      case repo.one(
             from(
               a in Account,
               where: a.eco_user_id == ^eco_user.id and a.id == ^id
             )
           ) do
        nil -> {:error, :account_not_found}
        account -> {:ok, account}
      end
    end)
    |> Ecto.Multi.delete(:delete, fn %{account: account} -> account end)
    |> Repo.transact()
  end
end
