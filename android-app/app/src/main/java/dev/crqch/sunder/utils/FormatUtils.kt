package dev.crqch.sunder.utils

import androidx.compose.runtime.Composable
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.ui.graphics.Color
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle
import java.util.Locale
import kotlin.math.abs

data class CurrencyFormatConfig(
    val symbol: String = "",
    val after: Boolean = false
)

val LocalCurrencyConfig = compositionLocalOf { CurrencyFormatConfig() }

val PRESET_CATEGORY_COLORS = listOf(
    "#ef4444",
    "#f97316",
    "#f59e0b",
    "#84cc16",
    "#22c55e",
    "#10b981",
    "#06b6d4",
    "#3b82f6",
    "#6366f1",
    "#8b5cf6",
    "#d946ef",
    "#f43f5e",
    "#0f172a",
    "#52525b",
    "#78716c"
)

fun formatAmount(
    amount: Float,
    currencySymbol: String = "",
    currencyAfter: Boolean = false,
    showPlusSign: Boolean = false,
    showMinusSign: Boolean = true
): String {
    val formattedNumber = String.format(Locale.getDefault(), "%.2f", abs(amount))
    val sign = when {
        amount < 0f && showMinusSign -> "-"
        amount > 0f && showPlusSign -> "+"
        else -> ""
    }
    val trimmedSymbol = currencySymbol.trim()
    return when {
        trimmedSymbol.isEmpty() -> "$sign$formattedNumber"
        currencyAfter -> "$sign$formattedNumber $trimmedSymbol"
        else -> "$sign$trimmedSymbol$formattedNumber"
    }
}

@Composable
fun formatCurrencyAmount(
    amount: Float,
    showPlusSign: Boolean = false,
    showMinusSign: Boolean = true
): String {
    val config = LocalCurrencyConfig.current
    return formatAmount(
        amount = amount,
        currencySymbol = config.symbol,
        currencyAfter = config.after,
        showPlusSign = showPlusSign,
        showMinusSign = showMinusSign
    )
}

fun parseHexColor(hex: String?, fallback: Color = Color(0xFF6366F1)): Color {
    if (hex.isNullOrBlank()) return fallback
    return try {
        val cleaned = hex.trim().removePrefix("#")
        val parsed = when (cleaned.length) {
            6 -> android.graphics.Color.parseColor("#$cleaned")
            8 -> android.graphics.Color.parseColor("#$cleaned")
            3 -> {
                val expanded = cleaned.map { "$it$it" }.joinToString("")
                android.graphics.Color.parseColor("#$expanded")
            }
            else -> return fallback
        }
        Color(parsed)
    } catch (_: Exception) {
        fallback
    }
}

fun formatDateShort(millis: Long): String {
    val formatter = DateTimeFormatter.ofLocalizedDate(FormatStyle.MEDIUM)
    return Instant.ofEpochMilli(millis)
        .atZone(ZoneId.systemDefault())
        .toLocalDate()
        .format(formatter)
}

fun formatRelativeDay(date: LocalDate): String {
    val today = LocalDate.now()
    return when (date) {
        today -> "Today"
        today.minusDays(1) -> "Yesterday"
        else -> {
            val pattern = if (date.year == today.year) "EEE, d MMM" else "d MMM yyyy"
            date.format(DateTimeFormatter.ofPattern(pattern, Locale.getDefault()))
        }
    }
}

fun formatSyncTimestamp(millis: Long?): String {
    if (millis == null || millis <= 0L) return "Never"
    val formatter = DateTimeFormatter.ofLocalizedDateTime(FormatStyle.MEDIUM, FormatStyle.SHORT)
    return Instant.ofEpochMilli(millis)
        .atZone(ZoneId.systemDefault())
        .format(formatter)
}
