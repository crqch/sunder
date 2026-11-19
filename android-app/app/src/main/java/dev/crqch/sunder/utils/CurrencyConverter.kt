package dev.crqch.sunder.utils

import dev.crqch.sunder.data.local.ExchangeRateEntity
import java.time.LocalDate
import java.time.ZoneOffset

/**
 * Converts between currencies using cached USD-based daily rates.
 *
 * Rate lookup matches the backend and the other Sunder clients: use the exact
 * date when present, otherwise the nearest earlier cached date, otherwise the
 * nearest later cached date. That keeps historical entry totals stable even when
 * the server has no snapshot for a particular day (weekends, holidays, outages).
 */
object CurrencyConverter {

    fun ratesByDate(rates: List<ExchangeRateEntity>): Map<String, Map<String, Double>> {
        val result = linkedMapOf<String, MutableMap<String, Double>>()
        for (rate in rates) {
            if (!rate.rate.isFinite() || rate.rate <= 0.0) continue
            val date = normalizeDate(rate.rateDate)
            val code = normalizeCurrencyCode(rate.currencyCode)
            result.getOrPut(date) { linkedMapOf() }[code] = rate.rate
        }
        return result
    }

    /**
     * Returns the rate that converts one unit of [fromCode] into [toCode] on [date],
     * or null when either side has no usable cached rate.
     */
    fun rate(
        fromCode: String?,
        toCode: String? = CANONICAL_BASE_CURRENCY,
        date: String?,
        rates: Map<String, Map<String, Double>>
    ): Double? {
        val from = normalizeCurrencyCode(fromCode)
        val to = normalizeCurrencyCode(toCode)
        if (from == to) return 1.0

        val dateKey = normalizeDate(date)
        val usdToFrom = findUsdRate(from, dateKey, rates) ?: return null
        val usdToTo = findUsdRate(to, dateKey, rates) ?: return null
        if (usdToFrom <= 0.0 || usdToTo <= 0.0) return null

        return usdToTo / usdToFrom
    }

    fun convert(
        amount: Double,
        fromCode: String?,
        toCode: String? = CANONICAL_BASE_CURRENCY,
        date: String?,
        rates: Map<String, Map<String, Double>>
    ): Double? {
        val rate = rate(fromCode, toCode, date, rates) ?: return null
        return amount * rate
    }

    /**
     * Converts the amount, falling back to the original amount when no rate is
     * cached yet. Use [convert] when the caller must distinguish missing data.
     */
    fun convertOrSame(
        amount: Double,
        fromCode: String?,
        toCode: String? = CANONICAL_BASE_CURRENCY,
        date: String?,
        rates: Map<String, Map<String, Double>>
    ): Double = convert(amount, fromCode, toCode, date, rates) ?: amount

    /**
     * Number of units of [code] that equal one USD on [date], following the
     * exact -> nearest earlier -> nearest later fallback chain.
     */
    fun findUsdRate(
        code: String?,
        date: String?,
        rates: Map<String, Map<String, Double>>
    ): Double? {
        val normalized = normalizeCurrencyCode(code)
        if (normalized == CANONICAL_BASE_CURRENCY) return 1.0

        val dateKey = normalizeDate(date)
        val sortedDates = rates.keys
            .filter { validRate(it, normalized, rates) }
            .sorted()

        if (sortedDates.isEmpty()) return null

        val exact = sortedDates.firstOrNull { it == dateKey }
        if (exact != null) return rates.getValue(exact).getValue(normalized)

        val earlier = sortedDates.lastOrNull { it < dateKey }
        if (earlier != null) return rates.getValue(earlier).getValue(normalized)

        val later = sortedDates.firstOrNull { it > dateKey }
        return later?.let { rates.getValue(it).getValue(normalized) }
    }

    fun normalizeDate(date: String?): String {
        val trimmed = date?.trim().orEmpty()
        if (trimmed.length >= 10) return trimmed.substring(0, 10)
        return trimmed.ifEmpty { LocalDate.now(ZoneOffset.UTC).toString() }
    }

    private fun validRate(
        date: String,
        code: String,
        rates: Map<String, Map<String, Double>>
    ): Boolean {
        val value = rates[date]?.get(code) ?: return false
        return value.isFinite() && value > 0.0
    }
}
