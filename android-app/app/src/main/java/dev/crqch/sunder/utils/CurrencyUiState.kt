package dev.crqch.sunder.utils

import dev.crqch.sunder.data.local.CurrencyEntity
import java.time.Instant
import java.time.ZoneOffset

/**
 * Everything the UI needs to render amounts across currencies: the cached server
 * catalog, the user's default display currency, and the cached daily rates.
 *
 * Conversion is always performed against the entry's own date so historical
 * totals and per-entry previews stay stable when rates change.
 */
data class CurrencyUiState(
    val currencies: List<CurrencyEntity> = emptyList(),
    val defaultCurrencyCode: String = CANONICAL_BASE_CURRENCY,
    val ratesByDate: Map<String, Map<String, Double>> = emptyMap()
) {
    val defaultCurrency: CurrencyEntity
        get() = currency(defaultCurrencyCode)

    val rateDayCount: Int
        get() = ratesByDate.size

    /** Resolves cached metadata, falling back to a neutral placeholder until the catalog loads. */
    fun currency(code: String?): CurrencyEntity {
        val normalized = normalizeCurrencyCode(code)
        return currencies.firstOrNull { it.code.equals(normalized, ignoreCase = true) }
            ?: fallback(normalized)
    }

    fun isDefault(code: String?): Boolean =
        normalizeCurrencyCode(code) == normalizeCurrencyCode(defaultCurrencyCode)

    fun convert(amount: Double, fromCode: String?, dateMillis: Long): Double? =
        CurrencyConverter.convert(
            amount = amount,
            fromCode = fromCode,
            toCode = defaultCurrencyCode,
            date = dateKey(dateMillis),
            rates = ratesByDate
        )

    /** Converts for aggregate totals, keeping the original amount when no rate is cached yet. */
    fun convertOrSame(amount: Double, fromCode: String?, dateMillis: Long): Double =
        CurrencyConverter.convertOrSame(
            amount = amount,
            fromCode = fromCode,
            toCode = defaultCurrencyCode,
            date = dateKey(dateMillis),
            rates = ratesByDate
        )

    /**
     * Converted preview for a single entry, or null when the entry already uses the
     * default display currency or no rate is available. Never fabricates a value.
     */
    fun convertedPreview(amount: Double, fromCode: String?, dateMillis: Long): String? {
        if (isDefault(fromCode)) return null
        val converted = convert(amount, fromCode, dateMillis) ?: return null
        return CurrencyFormatter.format(converted, defaultCurrency)
    }

    fun format(amount: Double, code: String?, showPlusSign: Boolean = false): String =
        CurrencyFormatter.format(
            amount = amount,
            currency = currency(code),
            showPlusSign = showPlusSign
        )

    fun formatSigned(amount: Double, code: String?): String =
        CurrencyFormatter.formatSigned(amount, currency(code))

    fun dateKey(dateMillis: Long): String = Instant.ofEpochMilli(dateMillis)
        .atZone(ZoneOffset.UTC)
        .toLocalDate()
        .toString()

    companion object {
        fun fallback(code: String): CurrencyEntity {
            val normalized = normalizeCurrencyCode(code)
            return CurrencyEntity(
                code = normalized,
                name = normalized,
                symbol = "",
                symbolAfter = false,
                decimalDigits = 2,
                isActive = true
            )
        }
    }
}
