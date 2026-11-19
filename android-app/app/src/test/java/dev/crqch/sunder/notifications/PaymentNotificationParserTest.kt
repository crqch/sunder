package dev.crqch.sunder.notifications

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class PaymentNotificationParserTest {

    @Test
    fun `detects Google Wallet contactless payment with merchant in title`() {
        val payload = NotificationPayload(
            packageName = "com.google.android.apps.walletnfcrel",
            appName = "Wallet",
            title = "Starbucks",
            text = "$5.75 with Visa •••• 1234",
            postTime = 1_700_000_000_000L
        )

        val result = PaymentNotificationParser.parse(payload)
        assertNotNull(result)
        assertEquals(5.75, result!!.amount, 0.001)
        assertEquals("USD", result.currencyCode)
        assertTrue(result.isExpense)
        assertEquals("Starbucks", result.title)
        assertEquals("Google Wallet", result.sourceAppName)
        assertEquals(1_700_000_000_000L, result.timestamp)
    }

    @Test
    fun `detects Google Wallet Polish zloty payment`() {
        val payload = NotificationPayload(
            packageName = "com.google.android.apps.walletnfcrel",
            title = "Żabka",
            text = "12,49 zł · Mastercard •••• 4321"
        )

        val result = PaymentNotificationParser.parse(payload)
        assertNotNull(result)
        assertEquals(12.49, result!!.amount, 0.001)
        assertEquals("PLN", result.currencyCode)
        assertTrue(result.isExpense)
        assertEquals("Żabka", result.title)
    }

    @Test
    fun `detects PayPal outgoing payment and extracts merchant`() {
        val payload = NotificationPayload(
            packageName = "com.paypal.android.p2pmobile",
            title = "Payment sent",
            text = "You paid $24.99 USD to Spotify"
        )

        val result = PaymentNotificationParser.parse(payload)
        assertNotNull(result)
        assertEquals(24.99, result!!.amount, 0.001)
        assertEquals("USD", result.currencyCode)
        assertTrue(result.isExpense)
        assertEquals("Spotify", result.title)
        assertEquals("PayPal", result.sourceAppName)
    }

    @Test
    fun `detects PayPal incoming payment as income`() {
        val payload = NotificationPayload(
            packageName = "com.paypal.android.p2pmobile",
            title = "PayPal",
            text = "John Doe sent you €40.00"
        )

        val result = PaymentNotificationParser.parse(payload, includeIncoming = true)
        assertNotNull(result)
        assertEquals(40.0, result!!.amount, 0.001)
        assertEquals("EUR", result.currencyCode)
        assertFalse(result.isExpense)
        assertEquals("John Doe", result.title)
    }

    @Test
    fun `ignores incoming payment when includeIncoming is false`() {
        val payload = NotificationPayload(
            packageName = "com.paypal.android.p2pmobile",
            title = "Money received",
            text = "You received $50.00 from Jane Doe"
        )

        val result = PaymentNotificationParser.parse(payload, includeIncoming = false)
        assertNull(result)
    }

    @Test
    fun `detects Revolut payment with euro comma decimal`() {
        val payload = NotificationPayload(
            packageName = "com.revolut.revolut",
            title = "Revolut",
            text = "Spent €18,50 at Pret A Manger"
        )

        val result = PaymentNotificationParser.parse(payload)
        assertNotNull(result)
        assertEquals(18.50, result!!.amount, 0.001)
        assertEquals("EUR", result.currencyCode)
        assertTrue(result.isExpense)
        assertEquals("Pret A Manger", result.title)
    }

    @Test
    fun `detects Polish bank card payment and extracts merchant`() {
        val payload = NotificationPayload(
            packageName = "pl.mbank",
            title = "mBank",
            text = "Płatność kartą 45,50 PLN w Biedronka. Dostępne środki: 1 234,56 PLN"
        )

        val result = PaymentNotificationParser.parse(payload)
        assertNotNull(result)
        assertEquals(45.50, result!!.amount, 0.001)
        assertEquals("PLN", result.currencyCode)
        assertTrue(result.isExpense)
        assertEquals("Biedronka", result.title)
    }

    @Test
    fun `detects Polish BLIK transaction with dash merchant`() {
        val payload = NotificationPayload(
            packageName = "pl.pkobp.iko",
            title = "IKO",
            text = "Transakcja BLIK 129,99 PLN - Allegro"
        )

        val result = PaymentNotificationParser.parse(payload)
        assertNotNull(result)
        assertEquals(129.99, result!!.amount, 0.001)
        assertEquals("PLN", result.currencyCode)
        assertTrue(result.isExpense)
        assertEquals("Allegro", result.title)
    }

    @Test
    fun `picks transaction amount instead of remaining balance`() {
        val payload = NotificationPayload(
            packageName = "com.chase.sig.android",
            title = "Chase",
            text = "Available balance: $1,420.50. You paid $19.95 at Trader Joe's"
        )

        val result = PaymentNotificationParser.parse(payload)
        assertNotNull(result)
        assertEquals(19.95, result!!.amount, 0.001)
        assertEquals("USD", result.currencyCode)
        assertEquals("Trader Joe's", result.title)
    }

    @Test
    fun `ignores balance-only notification`() {
        val payload = NotificationPayload(
            packageName = "com.chase.sig.android",
            title = "Account update",
            text = "Your available balance is $1,420.50"
        )

        assertNull(PaymentNotificationParser.parse(payload))
    }

    @Test
    fun `detects generic unknown bank app with strong payment wording`() {
        val payload = NotificationPayload(
            packageName = "com.example.localcreditunion",
            appName = "Local Credit Union",
            title = "Debit card purchase",
            text = "A purchase of £14.20 at Tesco Express was charged to your card."
        )

        val result = PaymentNotificationParser.parse(payload)
        assertNotNull(result)
        assertEquals(14.20, result!!.amount, 0.001)
        assertEquals("GBP", result.currencyCode)
        assertTrue(result.isExpense)
        assertEquals("Tesco Express", result.title)
    }

    @Test
    fun `parses thousands separators and international currencies`() {
        val jpy = PaymentNotificationParser.parse(
            NotificationPayload(
                packageName = "com.transferwise.android",
                title = "Wise",
                text = "You spent ¥1,500 at 7-Eleven"
            )
        )
        assertNotNull(jpy)
        assertEquals(1500.0, jpy!!.amount, 0.001)
        assertEquals("JPY", jpy.currencyCode)

        val chf = PaymentNotificationParser.parse(
            NotificationPayload(
                packageName = "com.revolut.revolut",
                title = "Revolut",
                text = "Paid CHF 1 234,50 at SBB"
            )
        )
        assertNotNull(chf)
        assertEquals(1234.50, chf!!.amount, 0.001)
        assertEquals("CHF", chf.currencyCode)

        val cad = PaymentNotificationParser.parse(
            NotificationPayload(
                packageName = "com.paypal.android.p2pmobile",
                title = "PayPal",
                text = "You paid C$89.99 to Indigo"
            )
        )
        assertNotNull(cad)
        assertEquals(89.99, cad!!.amount, 0.001)
        assertEquals("CAD", cad.currencyCode)
    }

    @Test
    fun `rejects OTP and 3DS approval notifications`() {
        val otpEn = NotificationPayload(
            packageName = "com.chase.sig.android",
            title = "Chase",
            text = "Your verification code is 849201 to approve this payment of $59.99"
        )
        val otpPl = NotificationPayload(
            packageName = "pl.mbank",
            title = "mBank",
            text = "Potwierdź płatność 120,00 PLN kodem jednorazowym 441298"
        )

        assertNull(PaymentNotificationParser.parse(otpEn))
        assertNull(PaymentNotificationParser.parse(otpPl))
    }

    @Test
    fun `rejects declined and failed payment notifications`() {
        val declined = NotificationPayload(
            packageName = "com.revolut.revolut",
            title = "Payment declined",
            text = "Your payment of €25.00 at Uber was declined due to insufficient funds"
        )
        val declinedPl = NotificationPayload(
            packageName = "pl.ing.mojeing",
            title = "Moje ING",
            text = "Płatność kartą 89,00 PLN została odrzucona"
        )

        assertNull(PaymentNotificationParser.parse(declined))
        assertNull(PaymentNotificationParser.parse(declinedPl))
    }

    @Test
    fun `rejects promotions payment requests and upcoming bills`() {
        val promo = NotificationPayload(
            packageName = "com.paypal.android.p2pmobile",
            title = "Special offer",
            text = "Save up to $20 on your next purchase with PayPal!"
        )
        val request = NotificationPayload(
            packageName = "com.venmo",
            title = "Venmo",
            text = "Alex requested $15.00 for dinner"
        )
        val billDue = NotificationPayload(
            packageName = "com.chase.sig.android",
            title = "Reminder",
            text = "Upcoming payment due on Nov 25: $145.00"
        )

        assertNull(PaymentNotificationParser.parse(promo))
        assertNull(PaymentNotificationParser.parse(request))
        assertNull(PaymentNotificationParser.parse(billDue))
    }

    @Test
    fun `ignores chat apps Sunder itself ongoing and group summary notifications`() {
        val whatsapp = NotificationPayload(
            packageName = "com.whatsapp",
            title = "Alex",
            text = "I paid $20.00 at Starbucks just now"
        )
        val sunderOwn = NotificationPayload(
            packageName = "dev.crqch.sunder",
            title = "Add $5.75 at Starbucks?",
            text = "Detected from Google Wallet · Tap to add to Sunder"
        )
        val ongoing = NotificationPayload(
            packageName = "com.revolut.revolut",
            title = "Revolut",
            text = "Paid $10.00 at Cafe",
            isOngoing = true
        )
        val groupSummary = NotificationPayload(
            packageName = "com.paypal.android.p2pmobile",
            title = "PayPal",
            text = "You paid $10.00 to Store",
            isGroupSummary = true
        )

        assertNull(PaymentNotificationParser.parse(whatsapp))
        assertNull(PaymentNotificationParser.parse(sunderOwn))
        assertNull(PaymentNotificationParser.parse(ongoing))
        assertNull(PaymentNotificationParser.parse(groupSummary))
    }

    @Test
    fun `uses bigText when short text is truncated`() {
        val payload = NotificationPayload(
            packageName = "pl.ing.mojeing",
            title = "Moje ING",
            text = "Płatność kartą...",
            bigText = "Płatność kartą na kwotę 64,90 zł w Empik"
        )

        val result = PaymentNotificationParser.parse(payload)
        assertNotNull(result)
        assertEquals(64.90, result!!.amount, 0.001)
        assertEquals("PLN", result.currencyCode)
        assertEquals("Empik", result.title)
    }

    @Test
    fun `skips card phrase and extracts actual merchant`() {
        val payload = NotificationPayload(
            packageName = "com.citi.citimobile",
            title = "Citi",
            text = "Payment of $31.40 from your Visa card at Whole Foods Market"
        )

        val result = PaymentNotificationParser.parse(payload)
        assertNotNull(result)
        assertEquals(31.40, result!!.amount, 0.001)
        assertEquals("Whole Foods Market", result.title)
    }
}
