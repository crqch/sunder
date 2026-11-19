package dev.crqch.sunder.notifications

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import androidx.core.app.NotificationManagerCompat

/**
 * Handles non-UI notification actions such as dismissing a payment prompt
 * without opening Sunder.
 */
class PaymentPromptActionReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent?) {
        if (intent?.action != PaymentNotificationIntent.ACTION_DISMISS_PAYMENT_PROMPT) return
        val notificationId = intent.getIntExtra(
            PaymentNotificationIntent.EXTRA_NOTIFICATION_ID,
            -1
        )
        if (notificationId >= 0) {
            runCatching {
                NotificationManagerCompat.from(context).cancel(notificationId)
            }
        }
    }
}
