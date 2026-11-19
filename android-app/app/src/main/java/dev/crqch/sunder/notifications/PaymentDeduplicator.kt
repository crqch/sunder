package dev.crqch.sunder.notifications

import javax.inject.Inject
import javax.inject.Singleton
import kotlin.math.roundToLong

/**
 * Prevents duplicate payment prompts caused by apps updating the same notification
 * or by both a wallet (e.g. Google Wallet) and a banking app notifying about the
 * same contactless payment within seconds of each other.
 */
@Singleton
class PaymentDeduplicator internal constructor(
    private val clock: () -> Long
) {
    @Inject
    constructor() : this(System::currentTimeMillis)

    private data class SeenPayment(
        val notificationKey: String,
        val sourcePackage: String,
        val amountMinorUnits: Long,
        val currencyCode: String,
        val isExpense: Boolean,
        val seenAtMillis: Long
    )

    private val recent = mutableListOf<SeenPayment>()

    @Synchronized
    fun shouldSuppress(notificationKey: String, payment: DetectedPayment): Boolean {
        val now = clock()
        pruneExpired(now)

        val amountMinor = (payment.amount * 100.0).roundToLong()
        val currency = payment.currencyCode.orEmpty().uppercase()

        val duplicate = recent.any { entry ->
            if (entry.amountMinorUnits != amountMinor ||
                entry.currencyCode != currency ||
                entry.isExpense != payment.isExpense
            ) {
                return@any false
            }

            val age = now - entry.seenAtMillis
            when {
                notificationKey.isNotBlank() &&
                    entry.notificationKey == notificationKey &&
                    age <= SAME_KEY_WINDOW_MS -> true

                entry.sourcePackage == payment.sourcePackage &&
                    age <= SAME_PACKAGE_WINDOW_MS -> true

                age <= CROSS_APP_WINDOW_MS -> true

                else -> false
            }
        }

        if (!duplicate) {
            recent += SeenPayment(
                notificationKey = notificationKey,
                sourcePackage = payment.sourcePackage,
                amountMinorUnits = amountMinor,
                currencyCode = currency,
                isExpense = payment.isExpense,
                seenAtMillis = now
            )
        }

        return duplicate
    }

    @Synchronized
    fun clear() {
        recent.clear()
    }

    private fun pruneExpired(now: Long) {
        recent.removeAll { now - it.seenAtMillis > SAME_KEY_WINDOW_MS }
    }

    companion object {
        internal const val SAME_KEY_WINDOW_MS = 5 * 60 * 1000L
        internal const val SAME_PACKAGE_WINDOW_MS = 60 * 1000L
        internal const val CROSS_APP_WINDOW_MS = 20 * 1000L
    }
}
