package dev.crqch.sunder.notifications

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class PaymentDeduplicatorTest {

    private fun samplePayment(
        amount: Double = 12.50,
        currency: String = "USD",
        isExpense: Boolean = true,
        pkg: String = "com.google.android.apps.walletnfcrel"
    ) = DetectedPayment(
        amount = amount,
        currencyCode = currency,
        isExpense = isExpense,
        title = "Starbucks",
        description = "Detected payment",
        timestamp = 1_000L,
        sourcePackage = pkg,
        sourceAppName = "Wallet",
        confidence = PaymentConfidence.HIGH
    )

    @Test
    fun `suppresses repeated notification with same key within window`() {
        var now = 10_000L
        val deduplicator = PaymentDeduplicator { now }
        val payment = samplePayment()

        assertFalse(deduplicator.shouldSuppress("key-1", payment))
        now += 120_000L // 2 minutes later
        assertTrue(deduplicator.shouldSuppress("key-1", payment))
        now += 400_000L // beyond 5-minute window
        assertFalse(deduplicator.shouldSuppress("key-1", payment))
    }

    @Test
    fun `suppresses cross-app duplicate within 20 seconds`() {
        var now = 10_000L
        val deduplicator = PaymentDeduplicator { now }
        val walletPayment = samplePayment(pkg = "com.google.android.apps.walletnfcrel")
        val bankPayment = samplePayment(pkg = "pl.mbank")

        assertFalse(deduplicator.shouldSuppress("wallet-key", walletPayment))
        now += 5_000L
        assertTrue(deduplicator.shouldSuppress("bank-key", bankPayment))
        now += 25_000L
        assertFalse(deduplicator.shouldSuppress("bank-key-2", bankPayment))
    }

    @Test
    fun `does not suppress different amounts or currencies`() {
        val now = 10_000L
        val deduplicator = PaymentDeduplicator { now }

        assertFalse(deduplicator.shouldSuppress("k1", samplePayment(amount = 10.0, currency = "USD")))
        assertFalse(deduplicator.shouldSuppress("k2", samplePayment(amount = 12.0, currency = "USD")))
        assertFalse(deduplicator.shouldSuppress("k3", samplePayment(amount = 10.0, currency = "EUR")))
        assertFalse(deduplicator.shouldSuppress("k4", samplePayment(amount = 10.0, currency = "USD", isExpense = false)))
    }
}
