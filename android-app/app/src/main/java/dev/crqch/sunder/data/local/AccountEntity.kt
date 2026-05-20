package dev.crqch.sunder.data.local

import androidx.room.Entity
import androidx.room.PrimaryKey


@Entity(tableName = "accounts")
data class AccountEntity(
    @PrimaryKey val id: String,
    val name: String,

    override val updatedAt: Long,
    override val createdAt: Long,
    override val deletedAt: Long? = null
) : Timestamps
