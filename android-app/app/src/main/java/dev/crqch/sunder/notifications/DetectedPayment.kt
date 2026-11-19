package dev.crqch.sunder.notifications

enum class PaymentConfidence {
    HIGH,
    MEDIUM
}

/**
 * Raw fields extracted from an Android status-bar notification.
 * Decoupled from Android framework classes so parsing is unit-testable.
 */
data class NotificationPayload(
    val packageName: String,
    val appName: String? = null,
    val title: String? = null,
    val text: String? = null,
    val bigText: String? = null,
    val subText: String? = null,
    val textLines: List<String> = emptyList(),
    val postTime: Long = System.currentTimeMillis(),
    val notificationKey: String = "",
    val isOngoing: Boolean = false,
    val isGroupSummary: Boolean = false
)

/**
 * Structured representation of a payment parsed from an incoming notification.
 */
data class DetectedPayment(
    val amount: Double,
    val currencyCode: String?,
    val isExpense: Boolean,
    val title: String,
    val description: String,
    val timestamp: Long,
    val sourcePackage: String,
    val sourceAppName: String?,
    val confidence: PaymentConfidence
)

/**
 * Pre-filled draft passed into the Create Entry screen when a user taps
 * a Sunder payment prompt notification.
 */
data class PaymentEntryDraft(
    val draftId: String,
    val title: String,
    val amount: Float,
    val currencyCode: String?,
    val isExpense: Boolean,
    val date: Long,
    val description: String,
    val notificationId: Int? = null
)
