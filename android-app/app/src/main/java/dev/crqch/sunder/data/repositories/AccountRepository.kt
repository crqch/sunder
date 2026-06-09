package dev.crqch.sunder.data.repositories

import dev.crqch.sunder.data.local.AccountDao
import dev.crqch.sunder.data.local.AccountEntity
import dev.crqch.sunder.data.local.AccountWithBalance
import dev.crqch.sunder.data.local.CategoryEntity
import dev.crqch.sunder.ui.accounts.AccountFormState
import dev.crqch.sunder.ui.categories.CategoryFormState
import dev.crqch.sunder.utils.Cuid2
import dev.crqch.sunder.data.sync.SyncManager
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject
import javax.inject.Singleton


@Singleton
class AccountRepository @Inject constructor(
    private val accountDao: AccountDao,
    private val syncManager: SyncManager
) {
    val allAccounts: Flow<List<AccountEntity>> = accountDao.getAccounts()

    val accountsWithBalance: Flow<List<AccountWithBalance>> = accountDao.getAccountsWithBalance()

    suspend fun getAccount(id: String): Flow<AccountEntity?> = accountDao.getAccount(id)

    suspend fun createAccount(name: String) {
        val now = System.currentTimeMillis()
        val account = AccountEntity(
            id = Cuid2.generate(),
            name = name,
            createdAt = now,
            updatedAt = now
        )
        accountDao.insert(account)
        syncManager.triggerSync()
    }


    suspend fun saveAccount(state: AccountFormState, id: String?) {
        val now = System.currentTimeMillis()

        if (id == null) {
            val entry = state.toAccount(
                id = Cuid2.generate(),
                createdAt = now,
                updatedAt = now
            )
            accountDao.upsert(entry)
        } else {
            val existing = accountDao.getAccountById(id)
            val updated = state.toAccount(
                id = id,
                createdAt = existing?.createdAt ?: now,
                updatedAt = now
            )
            accountDao.upsert(updated)
        }
        syncManager.triggerSync()
    }

    suspend fun deleteAccount(id: String) {
        accountDao.softDelete(id)
        syncManager.triggerSync()
    }
}
