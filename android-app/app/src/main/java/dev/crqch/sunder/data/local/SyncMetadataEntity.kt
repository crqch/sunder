package dev.crqch.sunder.data.local

import androidx.room.Entity
import androidx.room.PrimaryKey

@Entity(tableName = "sync_metadata")
data class SyncMetadataEntity(
    @PrimaryKey val id: Int = 0,
    val lastSyncedAt: Long?
)
