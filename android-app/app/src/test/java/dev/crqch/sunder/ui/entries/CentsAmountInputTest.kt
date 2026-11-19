package dev.crqch.sunder.ui.entries

import androidx.compose.ui.text.AnnotatedString
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class CentsAmountInputTest {

    @Test
    fun parseCentsAmountInput_formatsCommonPrices() {
        val fiveCents = parseCentsAmountInput("5", currentIsExpense = true)
        assertEquals("5", fiveCents.digits)
        assertEquals(0.05f, fiveCents.amount, 0.0001f)
        assertTrue(fiveCents.isExpense)

        val fiftyTwoCents = parseCentsAmountInput("52", currentIsExpense = true)
        assertEquals("52", fiftyTwoCents.digits)
        assertEquals(0.52f, fiftyTwoCents.amount, 0.0001f)

        val fiveTwentyThree = parseCentsAmountInput("523", currentIsExpense = true)
        assertEquals("523", fiveTwentyThree.digits)
        assertEquals(5.23f, fiveTwentyThree.amount, 0.0001f)

        val nineDollars = parseCentsAmountInput("900", currentIsExpense = true)
        assertEquals("900", nineDollars.digits)
        assertEquals(9.00f, nineDollars.amount, 0.0001f)
    }

    @Test
    fun parseCentsAmountInput_stripsLeadingZerosAndTogglesSign() {        val zeros = parseCentsAmountInput("000", currentIsExpense = true)
        assertEquals("", zeros.digits)
        assertEquals(0f, zeros.amount, 0.0001f)

        val income = parseCentsAmountInput("+523", currentIsExpense = true)
        assertEquals("523", income.digits)
        assertEquals(5.23f, income.amount, 0.0001f)
        assertFalse(income.isExpense)

        val expense = parseCentsAmountInput("-900", currentIsExpense = false)
        assertEquals("900", expense.digits)
        assertEquals(9.00f, expense.amount, 0.0001f)
        assertTrue(expense.isExpense)
    }

    @Test
    fun amountToCentsDigits_roundTripsAmounts() {
        assertEquals("", amountToCentsDigits(0f))
        assertEquals("5", amountToCentsDigits(0.05f))
        assertEquals("52", amountToCentsDigits(-0.52f))
        assertEquals("523", amountToCentsDigits(-5.23f))
        assertEquals("900", amountToCentsDigits(9.00f))
    }

    @Test
    fun visualTransformation_formatsAndMapsOffsetsWithinBounds() {
        val cases = listOf(
            "" to "",
            "5" to "0.05",
            "52" to "0.52",
            "523" to "5.23",
            "900" to "9.00",
            "12345" to "123.45",
            "123456789" to "1234567.89"
        )

        for ((raw, expected) in cases) {
            val transformed = CentsAmountVisualTransformation.filter(AnnotatedString(raw))
            assertEquals("Formatted text for '$raw'", expected, transformed.text.text)

            val mapping = transformed.offsetMapping
            for (origOffset in 0..raw.length) {
                val mapped = mapping.originalToTransformed(origOffset)
                assertTrue(
                    "originalToTransformed($origOffset) out of bounds for '$raw': $mapped",
                    mapped in 0..expected.length
                )
            }
            for (transOffset in 0..expected.length) {
                val mapped = mapping.transformedToOriginal(transOffset)
                assertTrue(
                    "transformedToOriginal($transOffset) out of bounds for '$raw': $mapped",
                    mapped in 0..raw.length
                )
            }
        }
    }

    @Test
    fun currencyDigits_respectZeroAndThreeDecimalCurrencies() {
        assertEquals("5", amountToCurrencyDigits(5.23f, 0))
        assertEquals("1234", amountToCurrencyDigits(1.234f, 3))
        assertEquals("", amountToCurrencyDigits(0.004f, 2))

        val yen = parseCurrencyAmountInput("1235", currentIsExpense = true, decimalDigits = 0)
        assertEquals("1235", yen.digits)
        assertEquals(1235f, yen.amount, 0.0001f)

        val dinar = parseCurrencyAmountInput("1234", currentIsExpense = false, decimalDigits = 3)
        assertEquals("1234", dinar.digits)
        assertEquals(1.234f, dinar.amount, 0.0001f)
        assertFalse(dinar.isExpense)
    }

    @Test
    fun visualTransformation_handlesZeroDecimalCurrencies() {
        val transformed = AmountVisualTransformation(0).filter(AnnotatedString("1235"))
        assertEquals("1235", transformed.text.text)
        assertEquals(4, transformed.offsetMapping.originalToTransformed(4))
        assertEquals(4, transformed.offsetMapping.transformedToOriginal(4))

        val threeDigits = AmountVisualTransformation(3).filter(AnnotatedString("1234"))
        assertEquals("1.234", threeDigits.text.text)
    }
}
