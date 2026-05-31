package dev.crqch.sunder.data.repositories

import dev.crqch.sunder.data.local.EntryDao
import dev.crqch.sunder.data.local.EntryEntity
import dev.crqch.sunder.utils.Cuid2
import kotlinx.coroutines.flow.Flow
import java.time.Instant
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class EntryRepository @Inject constructor(private val entryDao: EntryDao) {
    val allEntries: Flow<List<EntryEntity>> = entryDao.getAllEntries()

    fun getEntries(accountId: String? = null, categoryId: String? = null): Flow<List<EntryEntity>> {
        return when {
            accountId != null && categoryId != null -> {
                entryDao.getEntriesByAccountAndCategoryId(accountId, categoryId)
            }

            accountId != null -> {
                entryDao.getEntriesByAccountId(accountId)
            }

            categoryId != null -> {
                entryDao.getEntriesByCategoryId(categoryId)
            }

            else -> {
                allEntries
            }
        }
    }

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
}
