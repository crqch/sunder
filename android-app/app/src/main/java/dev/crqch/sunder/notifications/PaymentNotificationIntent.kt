package dev.crqch.sunder.notifications

import android.content.Context
import android.content.Intent
import dev.crqch.sunder.MainActivity
import dev.crqch.sunder.utils.Cuid2

/**
 * Serialization helpers for passing a detected payment from a notification
 * PendingIntent into [MainActivity] and handling dismiss actions.
 */
object PaymentNotificationIntent {

    const val ACTION_ADD_PAYMENT_ENTRY = "dev.crqch.sunder.action.ADD_PAYMENT_ENTRY"
    const val ACTION_DISMISS_PAYMENT_PROMPT = "dev.crqch.sunder.action.DISMISS_PAYMENT_PROMPT"

    const val EXTRA_DRAFT_ID = "extra_payment_draft_id"
    const val EXTRA_TITLE = "extra_payment_title"
    const val EXTRA_AMOUNT = "extra_payment_amount"
    const val EXTRA_CURRENCY_CODE = "extra_payment_currency_code"
    const val EXTRA_IS_EXPENSE = "extra_payment_is_expense"
    const val EXTRA_DATE = "extra_payment_date"
    const val EXTRA_DESCRIPTION = "extra_payment_description"
    const val EXTRA_NOTIFICATION_ID = "extra_payment_notification_id"

    fun createAddEntryIntent(
        context: Context,
        payment: DetectedPayment,
        notificationId: Int
    ): Intent = Intent(context, MainActivity::class.java).apply {
        action = ACTION_ADD_PAYMENT_ENTRY
        flags = Intent.FLAG_ACTIVITY_NEW_TASK or
            Intent.FLAG_ACTIVITY_SINGLE_TOP or
            Intent.FLAG_ACTIVITY_CLEAR_TOP
        putExtra(EXTRA_DRAFT_ID, Cuid2.generate())
        putExtra(EXTRA_TITLE, payment.title)
        putExtra(EXTRA_AMOUNT, payment.amount.toFloat())
        putExtra(EXTRA_CURRENCY_CODE, payment.currencyCode)
        putExtra(EXTRA_IS_EXPENSE, payment.isExpense)
        putExtra(EXTRA_DATE, payment.timestamp)
        putExtra(EXTRA_DESCRIPTION, payment.description)
        putExtra(EXTRA_NOTIFICATION_ID, notificationId)
    }

    fun createDismissIntent(
        context: Context,
        notificationId: Int
    ): Intent = Intent(context, PaymentPromptActionReceiver::class.java).apply {
        action = ACTION_DISMISS_PAYMENT_PROMPT
        putExtra(EXTRA_NOTIFICATION_ID, notificationId)
    }

    fun extractDraft(intent: Intent?): PaymentEntryDraft? {
        if (intent == null || intent.action != ACTION_ADD_PAYMENT_ENTRY) return null
        val amount = intent.getFloatExtra(EXTRA_AMOUNT, 0f)
        if (amount <= 0f) return null

        val draftId = intent.getStringExtra(EXTRA_DRAFT_ID)
            ?.takeIf { it.isNotBlank() }
            ?: Cuid2.generate()
        val title = intent.getStringExtra(EXTRA_TITLE).orEmpty()
        val currencyCode = intent.getStringExtra(EXTRA_CURRENCY_CODE)?.takeIf { it.isNotBlank() }
        val isExpense = intent.getBooleanExtra(EXTRA_IS_EXPENSE, true)
        val date = intent.getLongExtra(EXTRA_DATE, System.currentTimeMillis())
        val description = intent.getStringExtra(EXTRA_DESCRIPTION).orEmpty()
        val notificationId = intent.getIntExtra(EXTRA_NOTIFICATION_ID, -1).takeIf { it >= 0 }

        return PaymentEntryDraft(
            draftId = draftId,
            title = title,
            amount = amount,
            currencyCode = currencyCode,
            isExpense = isExpense,
            date = date,
            description = description,
            notificationId = notificationId
        )
    }

    fun clearDraftExtras(intent: Intent?) {
        if (intent == null) return
        if (intent.action == ACTION_ADD_PAYMENT_ENTRY) {
            intent.action = null
        }
        intent.removeExtra(EXTRA_DRAFT_ID)
        intent.removeExtra(EXTRA_TITLE)
        intent.removeExtra(EXTRA_AMOUNT)
        intent.removeExtra(EXTRA_CURRENCY_CODE)
        intent.removeExtra(EXTRA_IS_EXPENSE)
        intent.removeExtra(EXTRA_DATE)
        intent.removeExtra(EXTRA_DESCRIPTION)
        intent.removeExtra(EXTRA_NOTIFICATION_ID)
    }
}
