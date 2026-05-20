package dev.crqch.sunder.data.local

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy.Companion.REPLACE
import androidx.room.Query
import kotlinx.coroutines.flow.Flow

@Dao
interface EntryDao {
    @Query("select * from entries where deletedAt is null")
    fun getAllEntries(): Flow<List<EntryEntity>>

    @Query("select * from entries where deletedAt is null and id = :id")
    fun getEntryById(id: String): EntryEntity

    @Query("select * from entries where accountId = :accountId")
    fun getEntriesByAccountId(accountId: String): List<EntryEntity>

    @Insert(onConflict = REPLACE)
    suspend fun upsert(entryEntity: EntryEntity)

    @Insert(onConflict = REPLACE)
    suspend fun upsertAll(entries: List<EntryEntity>)

    @Query("update entries set deletedAt = :timestamp, updatedAt = :timestamp where id = :id")
    suspend fun softDelete(id: String, timestamp: Long = System.currentTimeMillis())
}
