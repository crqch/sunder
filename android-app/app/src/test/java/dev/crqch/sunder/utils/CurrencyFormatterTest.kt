package dev.crqch.sunder.utils

import dev.crqch.sunder.data.local.CurrencyEntity
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class CurrencyFormatterTest {

    private val usd = CurrencyEntity("USD", "US Dollar", "$", false, 2, true)
    private val eur = CurrencyEntity("EUR", "Euro", "€", false, 2, true)
    private val jpy = CurrencyEntity("JPY", "Japanese Yen", "¥", false, 0, true)
    private val pln = CurrencyEntity("PLN", "Polish Zloty", "zł", true, 2, true)
    private val chf = CurrencyEntity("CHF", "Swiss Franc", "CHF", false, 2, true)
    private val unknown = CurrencyEntity("XXX", "No Symbol", "", false, 2, true)

    @Test
    fun format_usesTwoDecimalsAndPrefixSymbol() {
        assertEquals("$1234.50", CurrencyFormatter.format(1234.5, usd))
        assertEquals("$0.00", CurrencyFormatter.format(0.0, usd))
        assertEquals("-$5.23", CurrencyFormatter.format(-5.23, usd))
        assertEquals("€12.00", CurrencyFormatter.format(12.0, eur))
    }

    @Test
    fun format_respectsZeroDecimalCurrencies() {
        assertEquals("¥1235", CurrencyFormatter.format(1234.56, jpy))
        assertEquals("¥0", CurrencyFormatter.format(0.0, jpy))
        assertEquals("-¥250", CurrencyFormatter.format(-250.0, jpy))
    }

    @Test
    fun format_placesSymbolAfterWhenConfigured() {
        assertEquals("1234.50 zł", CurrencyFormatter.format(1234.5, pln))
        assertEquals("-1234.50 zł", CurrencyFormatter.format(-1234.5, pln))
        assertEquals("+99.99 zł", CurrencyFormatter.formatSigned(99.99, pln))
    }

    @Test
    fun format_addsSpaceForMultiCharacterAlphabeticSymbols() {
        assertEquals("CHF 12.50", CurrencyFormatter.format(12.5, chf))
        assertEquals("-CHF 12.50", CurrencyFormatter.format(-12.5, chf))
    }

    @Test
    fun format_omitsSymbolWhenMetadataIsEmpty() {
        assertEquals("12.50", CurrencyFormatter.format(12.5, unknown))
        assertEquals("-12.50", CurrencyFormatter.format(-12.5, unknown))
    }

    @Test
    fun formatSigned_marksBothDirections() {
        assertEquals("+$5.23", CurrencyFormatter.formatSigned(5.23, usd))
        assertEquals("-$5.23", CurrencyFormatter.formatSigned(-5.23, usd))
        assertEquals("$0.00", CurrencyFormatter.formatSigned(0.0, usd))
        assertEquals("+¥500", CurrencyFormatter.formatSigned(500.0, jpy))
    }

    @Test
    fun formatAbsolute_dropsSign() {
        assertEquals("$5.23", CurrencyFormatter.formatAbsolute(-5.23, usd))
        assertEquals("¥500", CurrencyFormatter.formatAbsolute(-500.0, jpy))
    }

    @Test
    fun parse_handlesSymbolsGroupingAndSigns() {
        assertEquals(1234.5, CurrencyFormatter.parse("$1,234.50", usd)!!, 0.0001)
        assertEquals(-5.23, CurrencyFormatter.parse("-$5.23", usd)!!, 0.0001)
        assertEquals(1234.0, CurrencyFormatter.parse("$1,234", usd)!!, 0.0001)
        assertEquals(12.5, CurrencyFormatter.parse("12,50 €", eur)!!, 0.0001)
        assertEquals(1000.0, CurrencyFormatter.parse("¥1,000", jpy)!!, 0.0001)
        assertEquals(1234.5, CurrencyFormatter.parse("1234.50 zł", pln)!!, 0.0001)
        assertEquals(12.5, CurrencyFormatter.parse("CHF 12.50", chf)!!, 0.0001)
    }

    @Test
    fun parse_rejectsUnusableInput() {
        assertNull(CurrencyFormatter.parse("", usd))
        assertNull(CurrencyFormatter.parse("   ", usd))
        assertNull(CurrencyFormatter.parse("abc", usd))
        assertNull(CurrencyFormatter.parse("-", usd))
    }
}
