package dev.crqch.sunder.data.repositories

import androidx.room.withTransaction
import dev.crqch.sunder.data.local.CurrencyDao
import dev.crqch.sunder.data.local.EntryDao
import dev.crqch.sunder.data.local.EntryEntity
import dev.crqch.sunder.data.local.EntryWithDetails
import dev.crqch.sunder.data.local.SunderDatabase
import dev.crqch.sunder.ui.entries.EntryFormState
import dev.crqch.sunder.utils.CANONICAL_BASE_CURRENCY
import dev.crqch.sunder.utils.Cuid2
import dev.crqch.sunder.utils.normalizeCurrencyCode
import dev.crqch.sunder.data.sync.SyncManager
import kotlinx.coroutines.flow.Flow
import java.time.Instant
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class EntryRepository @Inject constructor(
    private val entryDao: EntryDao,
    private val currencyDao: CurrencyDao,
    private val database: SunderDatabase,
    private val syncManager: SyncManager
) {
    companion object {
        private const val SQL_BIND_CHUNK_SIZE = 500
    }

    val allEntries: Flow<List<EntryWithDetails>> = entryDao.getAllEntries(null)

    fun getEntries(
        accountId: String? = null,
        categoryId: String? = null,
        query: String? = null
    ): Flow<List<EntryWithDetails>> {
        return when {
            accountId != null && categoryId != null -> {
                entryDao.getEntriesByAccountAndCategoryId(accountId, categoryId, query)
            }

            accountId != null -> {
                entryDao.getEntriesByAccountId(accountId, query)
            }

            categoryId != null -> {
                entryDao.getEntriesByCategoryId(categoryId, query)
            }

            else -> {
                entryDao.getAllEntries(query)
            }
        }
    }

    fun getEntry(id: String) = entryDao.getEntryById(id)

    fun getEntryWithDetails(id: String) = entryDao.getEntryWithDetailsById(id)

    suspend fun createEntry(
        title: String,
        description: String,
        amount: Float,
        accountId: String,
        categoryId: String,
        date: Long,
        currencyCode: String = CANONICAL_BASE_CURRENCY
    ) {
        val validatedCurrency = validateCurrencyCode(currencyCode)
        val now = Instant.now().toEpochMilli()
        entryDao.upsert(
            EntryEntity(
                Cuid2.generate(),
                title, date,
                "",
                amount, accountId, categoryId, description, createdAt = now, updatedAt = now,
                currencyCode = validatedCurrency
            )
        )
        syncManager.triggerSync()
    }

    suspend fun saveEntry(
        state: EntryFormState,
        id: String? = null,
        currencyCode: String? = null
    ) {
        val now = System.currentTimeMillis()

        if (id == null) {
            val entry = state.toEntry(
                id = Cuid2.generate(),
                createdAt = now,
                updatedAt = now
            )
            val validatedCurrency = validateCurrencyCode(currencyCode ?: entry.currencyCode)
            entryDao.upsert(entry.copy(currencyCode = validatedCurrency))
        } else {
            val existing = entryDao.getEntryByIdDirect(id)
            val updated = state.toEntry(
                id = id,
                createdAt = existing?.createdAt ?: now,
                updatedAt = now
            )
            val resolvedCurrency = currencyCode
                ?: existing?.currencyCode
                ?: updated.currencyCode
            val validatedCurrency = validateCurrencyCode(
                code = resolvedCurrency,
                unchangedExistingCode = existing?.currencyCode
            )
            entryDao.upsert(updated.copy(currencyCode = validatedCurrency))
        }
        syncManager.triggerSync()
    }

    suspend fun updateEntry(
        entry: EntryEntity
    ) {
        val existing = entryDao.getEntryByIdDirect(entry.id)
        val validatedCurrency = validateCurrencyCode(
            code = entry.currencyCode,
            unchangedExistingCode = existing?.currencyCode
        )
        entryDao.upsert(
            entry.copy(
                currencyCode = validatedCurrency,
                updatedAt = System.currentTimeMillis()
            )
        )
        syncManager.triggerSync()
    }

    suspend fun deleteEntry(id: String) {
        entryDao.softDelete(id)
        syncManager.triggerSync()
    }

    suspend fun deleteEntries(ids: List<String>) {
        if (ids.isEmpty()) return
        val timestamp = System.currentTimeMillis()
        database.withTransaction {
            ids.chunked(SQL_BIND_CHUNK_SIZE).forEach { chunk ->
                entryDao.softDeleteAll(chunk, timestamp)
            }
        }
        syncManager.triggerSync()
    }

    /**
     * Re-labels entries to another currency without touching their numeric amount,
     * then queues a sync so the change reaches every other client.
     */
    suspend fun updateEntriesCurrency(ids: List<String>, currencyCode: String) {
        if (ids.isEmpty()) return
        val validatedCurrency = validateCurrencyCode(currencyCode)
        val timestamp = System.currentTimeMillis()
        database.withTransaction {
            ids.chunked(SQL_BIND_CHUNK_SIZE).forEach { chunk ->
                entryDao.updateCurrency(
                    ids = chunk,
                    currencyCode = validatedCurrency,
                    timestamp = timestamp
                )
            }
        }
        syncManager.triggerSync()
    }

    suspend fun countActiveByCategoryId(categoryId: String): Int {
        return entryDao.countActiveByCategoryId(categoryId)
    }

    private suspend fun validateCurrencyCode(
        code: String,
        unchangedExistingCode: String? = null
    ): String {
        val normalized = normalizeCurrencyCode(code)
        if (unchangedExistingCode != null && normalizeCurrencyCode(unchangedExistingCode) == normalized) {
            return normalized
        }
        if (currencyDao.count() > 0) {
            val cached = currencyDao.getCurrency(normalized)
            require(cached != null && cached.isActive) {
                "Currency '$normalized' is not supported or is inactive."
            }
        }
        return normalized
    }
}
