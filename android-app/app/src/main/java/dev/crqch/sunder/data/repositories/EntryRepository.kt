package dev.crqch.sunder.data.repositories

import dev.crqch.sunder.data.local.EntryDao
import dev.crqch.sunder.data.local.EntryEntity
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class EntryRepository @Inject constructor(private val entryDao: EntryDao) {
    val allEntries: Flow<List<EntryEntity>> = entryDao.getAllEntries()

    fun getEntries(accountId: String): Flow<List<EntryEntity>> =
        entryDao.getEntriesByAccountId(accountId)

}
