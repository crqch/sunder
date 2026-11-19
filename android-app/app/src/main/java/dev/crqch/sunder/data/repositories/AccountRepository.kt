package dev.crqch.sunder.data.repositories

import androidx.room.withTransaction
import dev.crqch.sunder.data.local.AccountDao
import dev.crqch.sunder.data.local.AccountEntity
import dev.crqch.sunder.data.local.AccountWithBalance
import dev.crqch.sunder.data.local.CurrencyDao
import dev.crqch.sunder.data.local.SunderDatabase
import dev.crqch.sunder.ui.accounts.AccountFormState
import dev.crqch.sunder.utils.Cuid2
import dev.crqch.sunder.utils.normalizeCurrencyCode
import dev.crqch.sunder.data.sync.SyncManager
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject
import javax.inject.Singleton


@Singleton
class AccountRepository @Inject constructor(
    private val accountDao: AccountDao,
    private val currencyDao: CurrencyDao,
    private val database: SunderDatabase,
    private val syncManager: SyncManager
) {
    companion object {
        private const val SQL_BIND_CHUNK_SIZE = 500
    }

    val allAccounts: Flow<List<AccountEntity>> = accountDao.getAccounts()

    val accountsWithBalance: Flow<List<AccountWithBalance>> = accountDao.getAccountsWithBalance()

    suspend fun getAccount(id: String): Flow<AccountEntity?> = accountDao.getAccount(id)

    suspend fun createAccount(name: String, currencyCode: String? = null) {
        val validatedCurrency = validateOptionalCurrencyCode(currencyCode)
        val now = System.currentTimeMillis()
        val account = AccountEntity(
            id = Cuid2.generate(),
            name = name,
            createdAt = now,
            updatedAt = now,
            currencyCode = validatedCurrency
        )
        accountDao.insert(account)
        syncManager.triggerSync()
    }


    suspend fun saveAccount(
        state: AccountFormState,
        id: String?,
        currencyCode: String? = null
    ) {
        val now = System.currentTimeMillis()

        if (id == null) {
            val validatedCurrency = validateOptionalCurrencyCode(currencyCode)
            val account = state.toAccount(
                id = Cuid2.generate(),
                createdAt = now,
                updatedAt = now
            )
            accountDao.upsert(account.copy(currencyCode = validatedCurrency))
        } else {
            val existing = accountDao.getAccountById(id)
            val validatedCurrency = validateOptionalCurrencyCode(
                code = currencyCode,
                unchangedExistingCode = existing?.currencyCode
            )
            val updated = state.toAccount(
                id = id,
                createdAt = existing?.createdAt ?: now,
                updatedAt = now
            )
            accountDao.upsert(updated.copy(currencyCode = validatedCurrency))
        }
        syncManager.triggerSync()
    }

    suspend fun deleteAccount(id: String) {
        accountDao.softDelete(id)
        syncManager.triggerSync()
    }

    suspend fun deleteAccounts(ids: List<String>) {
        if (ids.isEmpty()) return
        val timestamp = System.currentTimeMillis()
        database.withTransaction {
            ids.chunked(SQL_BIND_CHUNK_SIZE).forEach { chunk ->
                accountDao.softDeleteAll(chunk, timestamp)
            }
        }
        syncManager.triggerSync()
    }

    private suspend fun validateOptionalCurrencyCode(
        code: String?,
        unchangedExistingCode: String? = null
    ): String? {
        val trimmed = code?.trim()?.takeIf { it.isNotEmpty() } ?: return null
        val normalized = normalizeCurrencyCode(trimmed)
        if (unchangedExistingCode != null && normalizeCurrencyCode(unchangedExistingCode) == normalized) {
            return normalized
        }
        if (currencyDao.count() > 0) {
            val cached = currencyDao.getCurrency(normalized)
            require(cached != null) {
                "Currency '$normalized' is not supported."
            }
        }
        return normalized
    }
}
