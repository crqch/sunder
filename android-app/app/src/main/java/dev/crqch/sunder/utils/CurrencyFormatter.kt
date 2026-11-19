package dev.crqch.sunder.utils

import dev.crqch.sunder.data.local.CurrencyEntity
import java.util.Locale
import kotlin.math.abs

/**
 * Formats and parses monetary amounts using the metadata of a cached [CurrencyEntity].
 *
 * Formatting mirrors the rest of the Sunder clients: the sign is always rendered
 * first, the amount uses exactly the currency's decimal digits, and symbol placement
 * honours [CurrencyEntity.symbolAfter]. Multi-character alphabetic symbols such as
 * "CHF" get a separating space when placed before the number.
 */
object CurrencyFormatter {

    private const val MAX_DECIMAL_DIGITS = 6

    fun decimalDigits(currency: CurrencyEntity): Int =
        currency.decimalDigits.coerceIn(0, MAX_DECIMAL_DIGITS)

    fun format(
        amount: Double,
        currency: CurrencyEntity,
        showPlusSign: Boolean = false,
        showMinusSign: Boolean = true
    ): String {
        val digits = decimalDigits(currency)
        val number = String.format(Locale.US, "%.${digits}f", abs(amount))
        val sign = when {
            amount < 0.0 && showMinusSign -> "-"
            amount > 0.0 && showPlusSign -> "+"
            else -> ""
        }
        val symbol = currency.symbol.trim()

        return when {
            symbol.isEmpty() -> "$sign$number"
            currency.symbolAfter -> "$sign$number $symbol"
            needsSpaceBefore(symbol) -> "$sign$symbol $number"
            else -> "$sign$symbol$number"
        }
    }

    fun format(
        amount: Float,
        currency: CurrencyEntity,
        showPlusSign: Boolean = false,
        showMinusSign: Boolean = true
    ): String = format(
        amount = amount.toDouble(),
        currency = currency,
        showPlusSign = showPlusSign,
        showMinusSign = showMinusSign
    )

    /** Formats with an explicit sign for non-zero values ("+$5.23", "-$5.23"). */
    fun formatSigned(
        amount: Double,
        currency: CurrencyEntity
    ): String = format(amount, currency, showPlusSign = true)

    fun formatSigned(
        amount: Float,
        currency: CurrencyEntity
    ): String = format(amount, currency, showPlusSign = true)

    /** Formats without a sign, for zero or absolute values. */
    fun formatAbsolute(
        amount: Double,
        currency: CurrencyEntity
    ): String = format(abs(amount), currency)

    /**
     * Parses a user-entered amount using the currency's decimal precision.
     *
     * Accepts the currency symbol anywhere in the input, thousands separators, and
     * either "." or "," as the decimal separator. Returns null for unusable input.
     */
    fun parse(input: String, currency: CurrencyEntity): Double? {
        val digits = decimalDigits(currency)
        var cleaned = input
            .replace("\u00A0", "")
            .replace(" ", "")
            .trim()

        val symbol = currency.symbol.trim()
        if (symbol.isNotEmpty()) {
            cleaned = cleaned.replace(symbol, "")
        }
        if (cleaned.isEmpty()) return null

        val commaIndex = cleaned.indexOf(',')
        val dotIndex = cleaned.indexOf('.')
        cleaned = when {
            commaIndex >= 0 && dotIndex >= 0 -> {
                // Both present: the later one is the decimal separator, the other groups.
                if (commaIndex > dotIndex) {
                    cleaned.replace(".", "").replace(',', '.')
                } else {
                    cleaned.replace(",", "")
                }
            }

            commaIndex >= 0 -> {
                val fractionLength = cleaned.length - commaIndex - 1
                if (digits > 0 && commaIndex == cleaned.lastIndexOf(',') && fractionLength <= digits) {
                    cleaned.replace(',', '.')
                } else {
                    cleaned.replace(",", "")
                }
            }

            else -> cleaned
        }

        val sanitized = buildString {
            cleaned.forEachIndexed { index, char ->
                when {
                    char.isDigit() -> append(char)
                    char == '.' -> append(char)
                    (char == '-' || char == '+') && index == 0 -> append(char)
                }
            }
        }

        if (sanitized.isEmpty() || sanitized == "-" || sanitized == "+") return null
        return sanitized.toDoubleOrNull()
    }

    private fun needsSpaceBefore(symbol: String): Boolean =
        symbol.length > 1 && symbol.all { it.isLetter() || it == '.' }
}
