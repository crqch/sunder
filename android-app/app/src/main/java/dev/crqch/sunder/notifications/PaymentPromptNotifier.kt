package dev.crqch.sunder.notifications

import android.annotation.SuppressLint
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import dagger.hilt.android.qualifiers.ApplicationContext
import dev.crqch.sunder.R
import dev.crqch.sunder.utils.normalizeCurrencyCode
import java.util.Locale
import java.util.concurrent.atomic.AtomicInteger
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Posts actionable Sunder notifications prompting the user to add a detected
 * payment as an entry.
 */
@Singleton
class PaymentPromptNotifier @Inject constructor(
    @ApplicationContext private val context: Context
) {
    private val nextNotificationId = AtomicInteger(NOTIFICATION_ID_BASE)

    fun ensureChannel() {
        val manager = context.getSystemService(NotificationManager::class.java) ?: return
        if (manager.getNotificationChannel(CHANNEL_ID) != null) return

        val channel = NotificationChannel(
            CHANNEL_ID,
            context.getString(R.string.payment_prompt_channel_name),
            NotificationManager.IMPORTANCE_HIGH
        ).apply {
            description = context.getString(R.string.payment_prompt_channel_desc)
            enableVibration(true)
        }
        manager.createNotificationChannel(channel)
    }

    @SuppressLint("MissingPermission")
    fun showPaymentPrompt(payment: DetectedPayment): Boolean {
        if (!PaymentNotificationAccess.canPostNotifications(context)) {
            return false
        }
        ensureChannel()

        val notificationId = allocateNotificationId()
        val addIntent = PaymentNotificationIntent.createAddEntryIntent(
            context = context,
            payment = payment,
            notificationId = notificationId
        )
        val addPendingIntent = PendingIntent.getActivity(
            context,
            notificationId * 2,
            addIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )

        val dismissIntent = PaymentNotificationIntent.createDismissIntent(
            context = context,
            notificationId = notificationId
        )
        val dismissPendingIntent = PendingIntent.getBroadcast(
            context,
            notificationId * 2 + 1,
            dismissIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )

        val formattedAmount = formatAmountForPrompt(payment.amount, payment.currencyCode)
        val title = if (payment.isExpense) {
            context.getString(R.string.payment_prompt_title_expense, formattedAmount, payment.title)
        } else {
            context.getString(R.string.payment_prompt_title_income, formattedAmount, payment.title)
        }

        val sourceLabel = payment.sourceAppName?.takeIf { it.isNotBlank() }
        val contentText = if (sourceLabel != null) {
            context.getString(R.string.payment_prompt_body_with_source, sourceLabel)
        } else {
            context.getString(R.string.payment_prompt_body_generic)
        }

        val expandedText = if (payment.description.isNotBlank()) {
            "$contentText\n${payment.description}"
        } else {
            contentText
        }

        val notification = NotificationCompat.Builder(context, CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_notification_payment)
            .setContentTitle(title)
            .setContentText(contentText)
            .setStyle(NotificationCompat.BigTextStyle().bigText(expandedText))
            .setPriority(NotificationCompat.PRIORITY_HIGH)
            .setCategory(NotificationCompat.CATEGORY_REMINDER)
            .setAutoCancel(true)
            .setContentIntent(addPendingIntent)
            .addAction(
                0,
                context.getString(R.string.payment_prompt_action_add),
                addPendingIntent
            )
            .addAction(
                0,
                context.getString(R.string.payment_prompt_action_dismiss),
                dismissPendingIntent
            )
            .build()

        return runCatching {
            NotificationManagerCompat.from(context).notify(notificationId, notification)
            true
        }.getOrDefault(false)
    }

    fun showTestPrompt(defaultCurrencyCode: String): Boolean {
        val currency = normalizeCurrencyCode(defaultCurrencyCode)
        val samplePayment = DetectedPayment(
            amount = 4.75,
            currencyCode = currency,
            isExpense = true,
            title = context.getString(R.string.payment_test_merchant),
            description = context.getString(R.string.payment_test_description),
            timestamp = System.currentTimeMillis(),
            sourcePackage = context.packageName,
            sourceAppName = context.getString(R.string.app_name),
            confidence = PaymentConfidence.HIGH
        )
        return showPaymentPrompt(samplePayment)
    }

    fun cancelPrompt(notificationId: Int) {
        runCatching {
            NotificationManagerCompat.from(context).cancel(notificationId)
        }
    }

    private fun allocateNotificationId(): Int {
        val current = nextNotificationId.getAndIncrement()
        if (current > NOTIFICATION_ID_MAX) {
            nextNotificationId.set(NOTIFICATION_ID_BASE)
        }
        return current
    }

    companion object {
        const val CHANNEL_ID = "sunder_payment_prompts"
        private const val NOTIFICATION_ID_BASE = 4100
        private const val NOTIFICATION_ID_MAX = 4999

        internal fun formatAmountForPrompt(amount: Double, currencyCode: String?): String {
            val code = currencyCode?.trim()?.uppercase(Locale.US).orEmpty()
            val zeroDecimal = code in setOf("JPY", "KRW", "VND", "HUF", "ISK")
            val formattedNumber = if (zeroDecimal) {
                String.format(Locale.US, "%.0f", amount)
            } else {
                String.format(Locale.US, "%.2f", amount)
            }
            return when (code) {
                "USD" -> "$$formattedNumber"
                "EUR" -> "€$formattedNumber"
                "GBP" -> "£$formattedNumber"
                "JPY" -> "¥$formattedNumber"
                "PLN" -> "$formattedNumber zł"
                "" -> formattedNumber
                else -> "$formattedNumber $code"
            }
        }
    }
}
