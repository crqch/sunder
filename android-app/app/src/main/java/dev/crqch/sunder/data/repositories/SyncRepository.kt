package dev.crqch.sunder.data.repositories

import android.util.Log
import androidx.room.withTransaction
import dev.crqch.sunder.api.SyncApi
import dev.crqch.sunder.data.local.*
import dev.crqch.sunder.data.sync.*
import java.time.Instant
import java.time.ZoneOffset
import java.time.format.DateTimeFormatter
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class SyncRepository @Inject constructor(
    private val database: SunderDatabase,
    private val syncApi: SyncApi,
    private val accountDao: AccountDao,
    private val categoryDao: CategoryDao,
    private val entryDao: EntryDao,
    private val syncMetadataDao: SyncMetadataDao
) {
    private val formatter = DateTimeFormatter.ISO_INSTANT.withZone(ZoneOffset.UTC)

    suspend fun performSync() {
        val lastSyncedAt = syncMetadataDao.getLastSyncedAt() ?: 0L
        val syncStartTime = System.currentTimeMillis()

        val dirtyAccounts = accountDao.getModifiedSince(lastSyncedAt)
        val dirtyCategories = categoryDao.getModifiedSince(lastSyncedAt)
        val dirtyEntries = entryDao.getModifiedSince(lastSyncedAt)

        val request = SyncRequestDto(
            accounts = dirtyAccounts.map { it.toDto() },
            categories = dirtyCategories.map { it.toDto() },
            entries = dirtyEntries.map { it.toDto() }
        )

        val lastTimestamp = if (lastSyncedAt == 0L) "1970-01-01T00:00:00Z" else formatter.format(
            Instant.ofEpochMilli(lastSyncedAt)
        ) ?: "1970-01-01T00:00:00Z"

        try {
            val response = syncApi.sync(lastTimestamp, request)
            if (response.isSuccessful) {
                val data = response.body()
                if (data == null) {
                    Log.e("SyncRepository", "Sync successful but response body is null")
                    return
                }

                database.withTransaction {
                    if (data.accounts.isNotEmpty()) accountDao.upsertAll(data.accounts.map { it.toEntity() })
                    if (data.categories.isNotEmpty()) categoryDao.upsertAll(data.categories.map { it.toEntity() })
                    if (data.entries.isNotEmpty()) entryDao.upsertAll(data.entries.map { it.toEntity() })

                    syncMetadataDao.saveMetadata(
                        SyncMetadataEntity(
                            id = 0,
                            lastSyncedAt = syncStartTime
                        )
                    )
                }
            } else {
                Log.e(
                    "SyncRepository",
                    "Sync failed with code: ${response.code()}, error: ${
                        response.errorBody()?.string()
                    }"
                )
                throw Exception("Sync failed: ${response.code()}")
            }
        } catch (e: Exception) {
            Log.e("SyncRepository", "Exception during sync: ${e.message}", e)
            throw e
        }
    }

    suspend fun resetSync() {
        syncMetadataDao.saveMetadata(SyncMetadataEntity(lastSyncedAt = null))
    }
}
