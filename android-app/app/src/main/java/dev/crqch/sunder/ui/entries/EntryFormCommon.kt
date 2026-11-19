package dev.crqch.sunder.ui.entries

import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.input.OffsetMapping
import androidx.compose.ui.text.input.TransformedText
import androidx.compose.ui.text.input.VisualTransformation
import dev.crqch.sunder.data.local.EntryEntity
import dev.crqch.sunder.utils.CANONICAL_BASE_CURRENCY
import dev.crqch.sunder.utils.normalizeCurrencyCode
import java.time.Instant
import kotlin.math.abs
import kotlin.math.pow
import kotlin.math.roundToLong

private const val MAX_AMOUNT_DIGITS = 9
private const val DEFAULT_DECIMAL_DIGITS = 2

data class ParsedCentsAmount(
    val digits: String,
    val amount: Float,
    val isExpense: Boolean
)

/** Raw digit input for [amount], scaled by the currency's decimal precision. */
fun amountToCurrencyDigits(amount: Float, decimalDigits: Int): String {
    val digits = decimalDigits.coerceIn(0, 6)
    val scaled = (abs(amount.toDouble()) * 10.0.pow(digits)).roundToLong()
    return if (scaled <= 0L) "" else scaled.toString().take(MAX_AMOUNT_DIGITS)
}

fun amountToCentsDigits(amount: Float): String =
    amountToCurrencyDigits(amount, DEFAULT_DECIMAL_DIGITS)

fun parseCurrencyAmountInput(
    input: String,
    currentIsExpense: Boolean,
    decimalDigits: Int
): ParsedCentsAmount {
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
        .take(MAX_AMOUNT_DIGITS)
    val precision = decimalDigits.coerceIn(0, 6)
    val amount = (digits.toLongOrNull() ?: 0L) / 10.0.pow(precision)
    return ParsedCentsAmount(
        digits = digits,
        amount = amount.toFloat(),
        isExpense = isExpense
    )
}

fun parseCentsAmountInput(input: String, currentIsExpense: Boolean): ParsedCentsAmount =
    parseCurrencyAmountInput(input, currentIsExpense, DEFAULT_DECIMAL_DIGITS)

class AmountVisualTransformation(private val decimalDigits: Int = DEFAULT_DECIMAL_DIGITS) :
    VisualTransformation {

    override fun filter(text: AnnotatedString): TransformedText {
        val raw = text.text
        if (raw.isEmpty()) {
            return TransformedText(AnnotatedString(""), OffsetMapping.Identity)
        }

        val precision = decimalDigits.coerceIn(0, 6)
        if (precision == 0) {
            return TransformedText(AnnotatedString(raw), OffsetMapping.Identity)
        }

        val intPart = if (raw.length > precision) raw.dropLast(precision) else "0"
        val decPart = raw.takeLast(precision).padStart(precision, '0')
        val formatted = "$intPart.$decPart"

        val origLen = raw.length
        val transLen = formatted.length
        val dotIndex = origLen - precision

        val offsetMapping = object : OffsetMapping {
            override fun originalToTransformed(offset: Int): Int {
                return if (origLen <= precision) {
                    (transLen - origLen + offset).coerceIn(0, transLen)
                } else {
                    (if (offset <= dotIndex) offset else offset + 1).coerceIn(0, transLen)
                }
            }

            override fun transformedToOriginal(offset: Int): Int {
                return if (origLen <= precision) {
                    (offset - (transLen - origLen)).coerceIn(0, origLen)
                } else {
                    (if (offset <= dotIndex) offset else offset - 1).coerceIn(0, origLen)
                }
            }
        }

        return TransformedText(AnnotatedString(formatted), offsetMapping)
    }
}

object CentsAmountVisualTransformation : VisualTransformation by AmountVisualTransformation()

data class EntryFormState(
    val title: String = "",
    val date: Long = Instant.now().toEpochMilli(),
    val location: String = "",
    val amount: Float = 0.00f,
    val accountId: String = "",
    val categoryId: String = "",
    val description: String = "",
    val isExpense: Boolean = true,
    val currencyCode: String = CANONICAL_BASE_CURRENCY
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
            isExpense = entry.amount < 0,
            currencyCode = normalizeCurrencyCode(entry.currencyCode)
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
        updatedAt = updatedAt,
        currencyCode = normalizeCurrencyCode(currencyCode)
    )
}

enum class SelectionType { NONE, ACCOUNT, CATEGORY, CURRENCY }
