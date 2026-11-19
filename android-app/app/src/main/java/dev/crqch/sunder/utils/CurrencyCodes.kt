package dev.crqch.sunder.utils

/** The canonical base every cached exchange rate is expressed against. */
const val CANONICAL_BASE_CURRENCY = "USD"

/** Cleans user/server supplied codes into a comparable ISO-style code. */
fun normalizeCurrencyCode(code: String?): String {
    val cleaned = code?.trim()?.uppercase().orEmpty()
    return cleaned.ifEmpty { CANONICAL_BASE_CURRENCY }
}
