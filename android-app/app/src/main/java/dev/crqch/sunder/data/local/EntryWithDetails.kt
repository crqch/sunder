package dev.crqch.sunder.data.local

import androidx.room.Embedded
import androidx.room.Relation

data class EntryWithDetails(
    @Embedded val entry: EntryEntity,
    @Relation(
        parentColumn = "accountId",
        entityColumn = "id"
    )
    val account: AccountEntity,
    @Relation(
        parentColumn = "categoryId",
        entityColumn = "id"
    )
    val category: CategoryEntity
)
