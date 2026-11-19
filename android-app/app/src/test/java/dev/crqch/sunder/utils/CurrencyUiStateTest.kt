package dev.crqch.sunder.utils

import dev.crqch.sunder.data.local.CurrencyEntity
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import java.time.LocalDate
import java.time.ZoneOffset

class CurrencyUiStateTest {

    private val usd = CurrencyEntity("USD", "US Dollar", "$", false, 2, true)
    private val eur = CurrencyEntity("EUR", "Euro", "€", false, 2, true)
    private val gbp = CurrencyEntity("GBP", "Pound Sterling", "£", false, 2, true)
    private val jpy = CurrencyEntity("JPY", "Japanese Yen", "¥", false, 0, true)

    private val state = CurrencyUiState(
        currencies = listOf(usd, eur, gbp, jpy),
        defaultCurrencyCode = "EUR",
        ratesByDate = mapOf(
            "2026-11-19" to mapOf("USD" to 1.0, "EUR" to 0.9, "JPY" to 150.0)
        )
    )

    private val dateMillis = LocalDate.parse("2026-11-19")
        .atStartOfDay(ZoneOffset.UTC)
        .toInstant()
        .toEpochMilli()

    @Test
    fun convertedPreview_convertsHistoricalAmounts() {
        assertEquals("€9.00", state.convertedPreview(10.0, "USD", dateMillis))
        assertEquals("€6.00", state.convertedPreview(1000.0, "JPY", dateMillis))
    }

    @Test
    fun convertedPreview_isNullForDefaultCurrency() {
        assertNull(state.convertedPreview(10.0, "EUR", dateMillis))
    }

    @Test
    fun convertedPreview_isNullWhenNoRateExists() {
        assertNull(state.convertedPreview(10.0, "GBP", dateMillis))
    }

    @Test
    fun convertOrSame_fallsBackOnlyWhenRateIsMissing() {
        assertEquals(9.0, state.convertOrSame(10.0, "USD", dateMillis), 0.0001)
        assertEquals(10.0, state.convertOrSame(10.0, "GBP", dateMillis), 0.0001)
    }

    @Test
    fun currency_resolvesCachedMetadataAndFallsBackSafely() {
        assertEquals(eur, state.currency("eur"))
        assertEquals("USD", state.currency(null).code)
        assertEquals("", state.currency("XXX").symbol)
    }
}
