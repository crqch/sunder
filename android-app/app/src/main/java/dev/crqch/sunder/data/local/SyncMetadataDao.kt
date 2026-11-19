package dev.crqch.sunder.data.local

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import kotlinx.coroutines.flow.Flow

@Dao
interface SyncMetadataDao {
    @Query("SELECT lastSyncedAt FROM sync_metadata WHERE id = 0")
    suspend fun getLastSyncedAt(): Long?

    @Query("SELECT lastSyncedAt FROM sync_metadata WHERE id = 0")
    fun observeLastSyncedAt(): Flow<Long?>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun saveMetadata(metadata: SyncMetadataEntity)
}
