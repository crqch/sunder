package dev.crqch.sunder.data.repositories

import dev.crqch.sunder.data.local.EntryDao
import dev.crqch.sunder.data.local.EntryEntity
import dev.crqch.sunder.data.local.EntryWithDetails
import dev.crqch.sunder.ui.entries.EntryFormState
import dev.crqch.sunder.utils.Cuid2
import kotlinx.coroutines.flow.Flow
import java.time.Instant
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class EntryRepository @Inject constructor(private val entryDao: EntryDao) {
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
        date: Long
    ) {
        val now = Instant.now().toEpochMilli()
        entryDao.upsert(
            EntryEntity(
                Cuid2.generate(),
                title, date,
                "",
                amount, accountId, categoryId, description, createdAt = now, updatedAt = now
            )
        )
    }

    suspend fun saveEntry(state: EntryFormState, id: String? = null) {
        val now = System.currentTimeMillis()

        if (id == null) {
            val entry = state.toEntry(
                id = Cuid2.generate(),
                createdAt = now,
                updatedAt = now
            )
            entryDao.upsert(entry)
        } else {
            val existing = entryDao.getEntryByIdDirect(id)
            val updated = state.toEntry(
                id = id,
                createdAt = existing?.createdAt ?: now,
                updatedAt = now
            )
            entryDao.upsert(updated)
        }
    }

    suspend fun updateEntry(
        entry: EntryEntity
    ) {
        entryDao.upsert(entry.copy(updatedAt = System.currentTimeMillis()))
    }

    suspend fun deleteEntry(id: String) {
        entryDao.softDelete(id)
    }
}
