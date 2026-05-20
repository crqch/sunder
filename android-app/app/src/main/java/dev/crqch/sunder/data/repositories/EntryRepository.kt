package dev.crqch.sunder.data.repositories

import dev.crqch.sunder.data.local.EntryDao
import dev.crqch.sunder.data.local.EntryEntity
import kotlinx.coroutines.flow.Flow

class EntryRepository(private val entryDao: EntryDao) {
    val allEntries: Flow<List<EntryEntity>> = entryDao.getAllEntries()


}
