package dev.crqch.sunder.utils

import dev.crqch.sunder.data.local.ExchangeRateEntity
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class CurrencyConverterTest {

    private val rates = listOf(
        rate("2026-01-01", "USD", 1.0),
        rate("2026-01-01", "EUR", 0.8),
        rate("2026-01-01", "PLN", 4.0),
        rate("2026-01-10", "USD", 1.0),
        rate("2026-01-10", "EUR", 0.9),
        rate("2026-01-10", "PLN", 4.5),
        rate("2026-02-01", "USD", 1.0),
        rate("2026-02-01", "EUR", 0.85),
        rate("2026-02-01", "PLN", 4.2),
        rate("2026-02-01", "GBP", 0.75)
    )

    private val ratesByDate = CurrencyConverter.ratesByDate(rates)

    @Test
    fun ratesByDate_groupsSnapshotsAndDropsInvalidValues() {
        val grouped = CurrencyConverter.ratesByDate(
            listOf(
                rate("2026-01-01", "USD", 1.0),
                rate("2026-01-01", "EUR", 0.8),
                rate("2026-01-01", "BAD", 0.0),
                rate("2026-01-01", "NEG", -1.0)
            )
        )

        assertEquals(1, grouped.size)
        assertEquals(0.8, grouped.getValue("2026-01-01").getValue("EUR"), 0.000001)
        assertEquals(1.0, grouped.getValue("2026-01-01").getValue("USD"), 0.000001)
        assertEquals(false, grouped.getValue("2026-01-01").containsKey("BAD"))
        assertEquals(false, grouped.getValue("2026-01-01").containsKey("NEG"))
    }

    @Test
    fun convert_usesExactDateRate() {
        val converted = CurrencyConverter.convert(100.0, "EUR", "USD", "2026-01-10", ratesByDate)
        assertEquals(111.111111, converted!!, 0.0001)
    }

    @Test
    fun convert_fallsBackToNearestEarlierDate() {
        val converted = CurrencyConverter.convert(100.0, "EUR", "USD", "2026-01-05", ratesByDate)
        assertEquals(125.0, converted!!, 0.0001)
    }

    @Test
    fun convert_fallsBackToNearestLaterDateWhenBeforeHistory() {
        val converted = CurrencyConverter.convert(100.0, "EUR", "USD", "2025-12-01", ratesByDate)
        assertEquals(125.0, converted!!, 0.0001)
    }

    @Test
    fun convert_usesLatestCachedRateAfterLastSnapshot() {
        val converted = CurrencyConverter.convert(100.0, "EUR", "USD", "2026-06-01", ratesByDate)
        assertEquals(100.0 / 0.85, converted!!, 0.0001)
    }

    @Test
    fun convert_crossesTwoNonUsdCurrenciesHistorically() {
        val converted = CurrencyConverter.convert(100.0, "EUR", "PLN", "2026-01-05", ratesByDate)
        assertEquals(500.0, converted!!, 0.0001)

        val exact = CurrencyConverter.convert(100.0, "EUR", "PLN", "2026-01-10", ratesByDate)
        assertEquals(500.0, exact!!, 0.0001)
    }

    @Test
    fun convert_returnsOriginalForSameCurrency() {
        val converted = CurrencyConverter.convert(42.42, "EUR", "EUR", "2026-01-05", ratesByDate)
        assertEquals(42.42, converted!!, 0.000001)
    }

    @Test
    fun convert_handlesUsdOnEitherSide() {
        assertEquals(
            80.0,
            CurrencyConverter.convert(100.0, "USD", "EUR", "2026-01-05", ratesByDate)!!,
            0.0001
        )
        assertEquals(
            400.0,
            CurrencyConverter.convert(100.0, "USD", "PLN", "2026-01-05", ratesByDate)!!,
            0.0001
        )
    }

    @Test
    fun convert_usesLaterDateForCurrencyMissingFromEarlierSnapshots() {
        val converted = CurrencyConverter.convert(100.0, "GBP", "USD", "2026-01-05", ratesByDate)
        assertEquals(100.0 / 0.75, converted!!, 0.0001)
    }

    @Test
    fun convert_returnsNullWhenRateIsUnknown() {
        assertNull(CurrencyConverter.convert(100.0, "CAD", "USD", "2026-01-05", ratesByDate))
        assertNull(CurrencyConverter.convert(100.0, "EUR", "USD", "2026-01-05", emptyMap()))
    }

    @Test
    fun convertOrSame_keepsAmountWhenRateIsMissing() {
        assertEquals(
            100.0,
            CurrencyConverter.convertOrSame(100.0, "CAD", "USD", "2026-01-05", ratesByDate),
            0.000001
        )
    }

    @Test
    fun normalizeDate_truncatesTimestamps() {
        assertEquals("2026-01-05", CurrencyConverter.normalizeDate("2026-01-05T23:59:00Z"))
        assertEquals("2026-01-05", CurrencyConverter.normalizeDate(" 2026-01-05 "))
    }

    private fun rate(date: String, code: String, value: Double) =
        ExchangeRateEntity(rateDate = date, currencyCode = code, rate = value)
}
