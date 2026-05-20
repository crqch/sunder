package dev.crqch.sunder.data.local

import androidx.room.Entity
import androidx.room.PrimaryKey

@Entity(tableName = "entries")
data class EntryEntity(
    @PrimaryKey val id: String,
    val title: String,
    val date: String,
    val location: String,
    val amount: Float,
    val accountId: String,
    val categoryId: String,
    val description: String,

    override val createdAt: Long,
    override val updatedAt: Long,
    override val deletedAt: Long? = null
) : Timestamps
