package dev.crqch.sunder.ui.entries

import androidx.compose.runtime.Composable
import dev.crqch.sunder.data.local.EntryEntity
import java.time.Instant
import kotlin.math.abs
import kotlin.math.absoluteValue


data class EntryFormState(
    val title: String = "",
    val date: Long = Instant.now().toEpochMilli(),
    val location: String = "",
    val amount: Float = 0.00f,
    val accountId: String = "",
    val categoryId: String = "",
    val description: String = "",
    val isExpense: Boolean = true,
) {
    companion object {
        fun ofEntity(entry: EntryEntity): EntryFormState = EntryFormState(
            entry.title,
            entry.date,
            entry.location,
            abs(entry.amount),
            entry.accountId,
            entry.categoryId,
            entry.description,
            entry.amount < 0
        )

    }

    fun isFilled(): Boolean {
        if (title.isBlank()) return false
        if (accountId.isBlank() || categoryId.isBlank()) return false
        return amount > 0
    }

    fun toEntry(id: String, createdAt: Long, updatedAt: Long): EntryEntity = EntryEntity(
        id = id,
        title = title,
        date = date,
        location = location,
        amount = if (isExpense) -abs(amount) else abs(amount),
        accountId = accountId,
        categoryId = categoryId,
        description = description,
        createdAt = createdAt,
        updatedAt = updatedAt
    )
}

enum class SelectionType { NONE, ACCOUNT, CATEGORY }

