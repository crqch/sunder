package dev.crqch.sunder.data.repositories

import dev.crqch.sunder.data.local.AccountDao
import dev.crqch.sunder.data.local.AccountEntity
import dev.crqch.sunder.data.local.AccountWithBalance
import dev.crqch.sunder.utils.Cuid2
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class AccountRepository @Inject constructor(private val accountDao: AccountDao) {
    val allAccounts: Flow<List<AccountEntity>> = accountDao.getAccounts()

    val accountsWithBalance: Flow<List<AccountWithBalance>> = accountDao.getAccountsWithBalance()


    suspend fun createAccount(name: String) {
        val now = System.currentTimeMillis()
        val account = AccountEntity(
            id = Cuid2.generate(),
            name = name,
            createdAt = now,
            updatedAt = now
        )
        accountDao.insertAccount(account)
    }
}
