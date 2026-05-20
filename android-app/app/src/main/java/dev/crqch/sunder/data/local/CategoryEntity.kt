package dev.crqch.sunder.data.local

import androidx.room.Entity
import androidx.room.PrimaryKey
import java.util.Date

@Entity(tableName = "categories")
data class CategoryEntity(
    @PrimaryKey val id: String,
    val title: String,
    val description: String,
    val color: String,

    override val updatedAt: Long,
    override val createdAt: Long,
    override val deletedAt: Long? = null
) : Timestamps
