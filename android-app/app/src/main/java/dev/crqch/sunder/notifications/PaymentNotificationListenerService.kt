package dev.crqch.sunder.notifications

import android.app.Notification
import android.service.notification.NotificationListenerService
import android.service.notification.StatusBarNotification
import dagger.hilt.android.AndroidEntryPoint
import dev.crqch.sunder.data.repositories.PreferencesRepository
import dev.crqch.sunder.data.repositories.UserRepository
import javax.inject.Inject

/**
 * Listens for incoming system notifications (from PayPal, Google Wallet, banking
 * apps, etc.), identifies completed payment notifications, and posts a Sunder
 * prompt notification to add the transaction as an entry.
 */
@AndroidEntryPoint
class PaymentNotificationListenerService : NotificationListenerService() {

    @Inject
    lateinit var preferencesRepository: PreferencesRepository

    @Inject
    lateinit var userRepository: UserRepository

    @Inject
    lateinit var deduplicator: PaymentDeduplicator

    @Inject
    lateinit var promptNotifier: PaymentPromptNotifier

    override fun onNotificationPosted(sbn: StatusBarNotification?) {
        if (sbn == null) return
        if (sbn.packageName == packageName) return

        val settings = preferencesRepository.settings.value
        if (!settings.paymentDetectionEnabled) return
        if (userRepository.currentUser.value == null) return

        runCatching {
            val payload = extractPayload(sbn)
            val detected = PaymentNotificationParser.parse(
                payload = payload,
                includeIncoming = settings.paymentDetectionIncludeIncoming,
                fallbackCurrencyCode = settings.defaultCurrencyCode
            ) ?: return

            if (deduplicator.shouldSuppress(sbn.key, detected)) {
                return
            }

            promptNotifier.showPaymentPrompt(detected)
        }
    }

    private fun extractPayload(sbn: StatusBarNotification): NotificationPayload {
        val notification = sbn.notification
        val extras = notification.extras
        val flags = notification.flags

        val title = extras?.getCharSequence(Notification.EXTRA_TITLE)?.toString()
        val text = extras?.getCharSequence(Notification.EXTRA_TEXT)?.toString()
        val bigText = extras?.getCharSequence(Notification.EXTRA_BIG_TEXT)?.toString()
        val subText = extras?.getCharSequence(Notification.EXTRA_SUB_TEXT)?.toString()
        val textLines = extras?.getCharSequenceArray(Notification.EXTRA_TEXT_LINES)
            ?.mapNotNull { it?.toString() }
            .orEmpty()

        val appName = resolveAppLabel(sbn.packageName)

        return NotificationPayload(
            packageName = sbn.packageName.orEmpty(),
            appName = appName,
            title = title,
            text = text,
            bigText = bigText,
            subText = subText,
            textLines = textLines,
            postTime = sbn.postTime.takeIf { it > 0L } ?: System.currentTimeMillis(),
            notificationKey = sbn.key.orEmpty(),
            isOngoing = (flags and (Notification.FLAG_ONGOING_EVENT or Notification.FLAG_FOREGROUND_SERVICE)) != 0,
            isGroupSummary = (flags and Notification.FLAG_GROUP_SUMMARY) != 0
        )
    }

    private fun resolveAppLabel(pkg: String): String? = runCatching {
        val info = packageManager.getApplicationInfo(pkg, 0)
        packageManager.getApplicationLabel(info)?.toString()?.takeIf { it.isNotBlank() }
    }.getOrNull()
}
