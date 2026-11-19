package dev.crqch.sunder.ui.entries

import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.input.OffsetMapping
import androidx.compose.ui.text.input.TransformedText
import androidx.compose.ui.text.input.VisualTransformation
import dev.crqch.sunder.data.local.EntryEntity
import java.time.Instant
import kotlin.math.abs
import kotlin.math.roundToLong

private const val MAX_AMOUNT_CENTS_DIGITS = 9

data class ParsedCentsAmount(
    val digits: String,
    val amount: Float,
    val isExpense: Boolean
)

fun amountToCentsDigits(amount: Float): String {
    val cents = (abs(amount.toDouble()) * 100.0).roundToLong()
    return if (cents <= 0L) "" else cents.toString().take(MAX_AMOUNT_CENTS_DIGITS)
}

fun parseCentsAmountInput(input: String, currentIsExpense: Boolean): ParsedCentsAmount {
    var isExpense = currentIsExpense
    if (input.contains('-')) {
        isExpense = true
    }
    if (input.contains('+') || input.contains('=')) {
        isExpense = false
    }
    val digits = input
        .filter { it.isDigit() }
        .trimStart('0')
        .take(MAX_AMOUNT_CENTS_DIGITS)
    val amount = (digits.toLongOrNull() ?: 0L) / 100f
    return ParsedCentsAmount(
        digits = digits,
        amount = amount,
        isExpense = isExpense
    )
}

object CentsAmountVisualTransformation : VisualTransformation {
    override fun filter(text: AnnotatedString): TransformedText {
        val raw = text.text
        if (raw.isEmpty()) {
            return TransformedText(AnnotatedString(""), OffsetMapping.Identity)
        }

        val intPart = if (raw.length > 2) raw.dropLast(2) else "0"
        val decPart = raw.takeLast(2).padStart(2, '0')
        val formatted = "$intPart.$decPart"

        val origLen = raw.length
        val transLen = formatted.length

        val offsetMapping = object : OffsetMapping {
            override fun originalToTransformed(offset: Int): Int {
                return if (origLen <= 2) {
                    (transLen - origLen + offset).coerceIn(0, transLen)
                } else {
                    val dotIndex = origLen - 2
                    (if (offset <= dotIndex) offset else offset + 1).coerceIn(0, transLen)
                }
            }

            override fun transformedToOriginal(offset: Int): Int {
                return if (origLen <= 2) {
                    (offset - (transLen - origLen)).coerceIn(0, origLen)
                } else {
                    val dotIndex = origLen - 2
                    (if (offset <= dotIndex) offset else offset - 1).coerceIn(0, origLen)
                }
            }
        }

        return TransformedText(AnnotatedString(formatted), offsetMapping)
    }
}

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
            title = entry.title,
            date = entry.date,
            location = entry.location,
            amount = abs(entry.amount),
            accountId = entry.accountId,
            categoryId = entry.categoryId,
            description = entry.description,
            isExpense = entry.amount < 0
        )
    }

    fun isFilled(): Boolean {
        if (title.isBlank()) return false
        if (accountId.isBlank() || categoryId.isBlank()) return false
        return amount > 0f
    }

    fun toEntry(id: String, createdAt: Long, updatedAt: Long): EntryEntity = EntryEntity(
        id = id,
        title = title.trim(),
        date = date,
        location = location.trim(),
        amount = if (isExpense) -abs(amount) else abs(amount),
        accountId = accountId,
        categoryId = categoryId,
        description = description.trim(),
        createdAt = createdAt,
        updatedAt = updatedAt
    )
}

enum class SelectionType { NONE, ACCOUNT, CATEGORY }
