package dev.crqch.sunder.notifications

import dev.crqch.sunder.utils.normalizeCurrencyCode
import java.util.Locale

/**
 * Pure-Kotlin heuristic parser that inspects Android notification text and
 * metadata to detect completed payments and extract structured entry fields.
 */
object PaymentNotificationParser {

    private const val MAX_REASONABLE_AMOUNT = 100_000_000.0
    private const val MAX_DESCRIPTION_LENGTH = 220
    private const val MAX_TITLE_LENGTH = 80

    private val isoCurrencyCodes = setOf(
        "USD", "EUR", "GBP", "PLN", "CHF", "CAD", "AUD", "NZD", "JPY", "CNY",
        "INR", "KRW", "SEK", "NOK", "DKK", "CZK", "HUF", "RON", "BGN", "TRY",
        "UAH", "BRL", "MXN", "SGD", "HKD", "ZAR", "AED", "SAR", "ILS", "THB",
        "PHP", "IDR", "MYR", "VND", "TWD", "ISK"
    )

    private val symbolToCurrency: Map<String, String> = mapOf(
        "us$" to "USD",
        "ca$" to "CAD",
        "c$" to "CAD",
        "au$" to "AUD",
        "a$" to "AUD",
        "nz$" to "NZD",
        "hk$" to "HKD",
        "sg$" to "SGD",
        "s$" to "SGD",
        "mx$" to "MXN",
        "r$" to "BRL",
        "cn¥" to "CNY",
        "rmb" to "CNY",
        "$" to "USD",
        "€" to "EUR",
        "£" to "GBP",
        "zł" to "PLN",
        "zl" to "PLN",
        "¥" to "JPY",
        "₹" to "INR",
        "₩" to "KRW",
        "₴" to "UAH",
        "kč" to "CZK",
        "kc" to "CZK",
        "₺" to "TRY",
        "₪" to "ILS",
        "฿" to "THB",
        "₱" to "PHP",
        "₫" to "VND",
        "ft" to "HUF",
        "fr." to "CHF",
    )

    private const val AMOUNT_NUMBER_PATTERN =
        """(?:\d{1,3}(?:[,\u00A0 ]\d{3})+[.,]\d{1,2}|\d{1,3}(?:\.\d{3})+,\d{1,2}|\d+[.,]\d{1,2}|\d{1,3}(?:,\d{3})+|\d+)"""

    private val isoAlternation = isoCurrencyCodes.joinToString("|")

    private const val SYMBOL_PREFIX_ALTERNATION =
        """US\$|CA\$|AU\$|NZ\$|HK\$|SG\$|MX\$|CN¥|RMB|C\$|A\$|S\$|R\$|Fr\.|[\$€£¥₹₩₴₺₪฿₱₫]"""

    private const val SYMBOL_SUFFIX_ALTERNATION =
        """zł|zl|Kč|Kc|Ft|€|\$|£|¥|₹|₩|₴|₺|₪|฿|₱|₫"""

    // Prefix symbol: "$12.50", "-€19,99", "US$ 40.00", "$-12.50"
    private val prefixSymbolRegex = Regex(
        """(?<![\p{L}\d])([+-])?\s*($SYMBOL_PREFIX_ALTERNATION)\s*([+-])?\s*($AMOUNT_NUMBER_PATTERN)(?![\d.,])""",
        RegexOption.IGNORE_CASE
    )

    // Prefix ISO code: "USD 12.50", "-PLN 45,50", "CHF 15.00"
    private val prefixIsoRegex = Regex(
        """(?<![\p{L}\d])([+-])?\s*\b($isoAlternation)\b\s*([+-])?\s*($AMOUNT_NUMBER_PATTERN)(?![\d.,])""",
        RegexOption.IGNORE_CASE
    )

    // Suffix ISO code or currency symbol: "45,50 PLN", "-12.99 EUR", "24,99 zł", "150 Kč"
    private val suffixCurrencyRegex = Regex(
        """(?<![\p{L}\d])([+-])?\s*($AMOUNT_NUMBER_PATTERN)\s*($isoAlternation|$SYMBOL_SUFFIX_ALTERNATION)(?![\p{L}\d])""",
        RegexOption.IGNORE_CASE
    )

    private const val WB_START = """(?<![\p{L}\d])"""
    private const val WB_END = """(?![\p{L}\d])"""

    // Reject OTP/2FA/3DS authorization, declines, promos, reminders, and payment requests
    private val negativePatterns = listOf(
        // OTP / 2FA / pending 3DS approval
        Regex(
            """$WB_START(otp|2fa|verification code|security code|one[- ]time (?:password|code|pin)|passcode|approve (?:this )?payment|confirm (?:this )?payment|authorize (?:this )?(?:payment|transaction)|kod (?:jednorazowy|autoryzacyjny|weryfikacyjny|sms)|potwierd[zź] (?:transakcj[eę]|p[łl]atno[śs][ćc])|autoryzuj)$WB_END""",
            RegexOption.IGNORE_CASE
        ),
        // Declined / failed / cancelled
        Regex(
            """$WB_START(declined|failed|unsuccessful|rejected|insufficient funds|canceled|cancelled|could not be processed|wasn't completed|not completed|odrzucon[aoe]|odrzucenie|nieudan[ae]|brak [śs]rodk[oó]w|anulowan[aoe]|nie powiod[łl]a)$WB_END""",
            RegexOption.IGNORE_CASE
        ),
        // Promotions / offers / ads
        Regex(
            """$WB_START(save up to|get up to|off your next|discount|coupon|promo(?:tion)?|special offer|limited[- ]time|win up to|cashback up to|earn up to|subscribe now|upgrade to|free trial|rabat|promocj[aei]|zni[żz]k[aei]|do wygrania|oferta specjalna)$WB_END""",
            RegexOption.IGNORE_CASE
        ),
        // Requests / reminders / upcoming bills (not yet paid)
        Regex(
            """$WB_START(requested|is requesting|requests?$WB_END.*${WB_START}from you|payment request|money request|bill due|payment due|upcoming payment|scheduled (?:payment|transfer)|due on|due in|reminder|prosi o|pro[śs]ba o|przypomnienie|zbli[żz]a si[ęe] termin|zaplanowan[ya])$WB_END""",
            RegexOption.IGNORE_CASE
        ),
    )

    // Balance context immediately preceding an amount
    private val balancePrefixRegex = Regex(
        """$WB_START(balance|available|remaining|avail\.?|new bal|saldo|dost[ęe]pne|wolne [śs]rodki|pozosta[łl]o|stan konta|limit)$WB_END[^.\n]{0,18}$""",
        RegexOption.IGNORE_CASE
    )

    // Incoming payment / refund signals (evaluated BEFORE outgoing so "John sent you $20" is income)
    private val incomingKeywordsRegex = Regex(
        """$WB_START(sent you|paid you|transferred you|you received|received(?:\s+a)?\s+(?:payment|transfer|money|refund)?|refund(?:ed)?|credited|deposit(?:ed)?|incoming transfer|cashback received|money received|otrzyma[łl](?:e[śs]|a[śs]|o)?|otrzymano|wp[łl]yw|uznanie|zwrot|przelew przychodz[ąa]cy|dosta[łl](?:e[śs]|a[śs]))$WB_END""",
        RegexOption.IGNORE_CASE
    )

    // Strong outgoing payment keywords
    private val outgoingKeywordsRegex = Regex(
        """$WB_START(you paid|you sent|you spent|paid|payment|spent|charged|purchase[d]?|debited|debit|withdrew|withdrawal|bought|transaction|card payment|contactless|tap to pay|autopay|auto-pay|direct debit|transferred|transfer to|sent(?:\s+to)?|order total|zap[łl]acono|zap[łl]aci[łl](?:e[śs]|a[śs])|p[łl]atno[śs][ćc]|wyda[łl](?:e[śs]|a[śs])|obci[ąa][żz]ono|obci[ąa][żz]enie|zakup|transakcj[aei]|przelew wychodz[ąa]cy|wys[łl]ano|wyp[łl]at[ay]|blik|kart[ąa]|bezahlt|zahlung|paiement)$WB_END""",
        RegexOption.IGNORE_CASE
    )

    // Payment instrument / wallet cues (for concise notifications like Google Wallet: "Starbucks / $5.75 with Visa •••• 1234")
    private val walletCardCueRegex = Regex(
        """(visa|mastercard|maestro|amex|american express|discover|debit|credit|card|kart[ąay]|••••|\*{3,4}\s*\d{4}|ending in \d{4}|done$WB_END|tap$WB_END|nfc|google pay|google wallet|apple pay|samsung pay|blik)""",
        RegexOption.IGNORE_CASE
    )

    // Generic titles that should NOT be used as the merchant name
    private val genericTitles = setOf(
        "payment", "payment sent", "payment received", "payment successful", "payment complete",
        "payment completed", "payment alert", "payment confirmation", "successful payment",
        "transaction", "transaction alert", "card payment", "card purchase",
        "debit card purchase", "credit card purchase", "contactless payment", "online payment",
        "purchase", "purchase alert", "new transaction", "money sent", "money received",
        "transfer", "bank transfer", "debit alert", "credit alert", "refund", "receipt",
        "notification", "alert", "update", "google pay", "google wallet", "paypal", "revolut",
        "wise", "venmo", "cash app", "samsung pay", "samsung wallet", "chase", "bank of america",
        "wells fargo", "citi", "capital one", "american express", "amex", "monzo", "starling",
        "n26", "bunq", "ing", "moje ing", "mbank", "iko", "pko bp", "santander", "peopay",
        "pekao", "millennium", "alior", "alior bank", "bnp paribas", "blik",
        "płatność", "platnosc", "płatność kartą", "płatność blik", "nowa płatność",
        "potwierdzenie płatności", "transakcja", "transakcja kartą", "transakcja blik",
        "nowa transakcja", "zakup", "obciążenie", "przelew", "wpływ", "wpływ na konto",
        "zwrot", "powiadomienie", "operacja", "operacja kartą", "zapłacono"
    )

    // Structured merchant extraction patterns
    private val merchantPatterns = listOf(
        // "John Doe sent you $25.00" / "Jane paid you €10"
        Regex("""^([^\n,:]{2,50}?)\s+(?:sent you|paid you|transferred you)$WB_END""", RegexOption.IGNORE_CASE),
        // "You paid $12.50 to Starbucks" / "Paid €14.99 at Lidl"
        Regex(
            """$WB_START(?:paid|sent|spent|charged|purchase[d]?|payment|transaction|debited|bought|transfer(?:red)?)$WB_END[^\n]{0,60}?$WB_START(?:to|at)\s+([^\n.;|]{2,55})""",
            RegexOption.IGNORE_CASE
        ),
        // "You received $50.00 from Jane Doe" / "Refund of $15.00 from Amazon" / "Purchase from Amazon"
        Regex(
            """$WB_START(?:received|refund(?:ed)?|credited|deposit(?:ed)?|purchase[d]?|charged|bought)$WB_END[^\n]{0,60}?$WB_START(?:from|by)\s+([^\n.;|]{2,55})""",
            RegexOption.IGNORE_CASE
        ),
        // Polish: "Zapłacono 24,99 zł w Żabka" / "Płatność kartą 30,00 PLN w Rossmann" / "Przelew do Jan Kowalski"
        Regex(
            """$WB_START(?:zap[łl]acono|zap[łl]aci[łl](?:e[śs]|a[śs])|p[łl]atno[śs][ćc](?:\s+kart[ąa]|\s+blik)?|transakcj[aei](?:\s+kart[ąa]|\s+blik)?|zakup|obci[ąa][żz]enie|przelew|otrzymano|otrzyma[łl](?:e[śs]|a[śs])|zwrot)$WB_END[^\n]{0,60}?$WB_START(?:w|u|dla|do|od|na rzecz)\s+([^\n.;|]{2,55})""",
            RegexOption.IGNORE_CASE
        ),
        // Dash/colon merchant suffix after amount: "Transakcja BLIK 49,00 PLN - Allegro" or "Card payment $12.50: Starbucks"
        Regex(
            """(?:$AMOUNT_NUMBER_PATTERN)\s*(?:$isoAlternation|$SYMBOL_SUFFIX_ALTERNATION)?\s*(?:[-–—:]\s*)([A-Za-zÀ-ÖØ-öø-ÿĄĆĘŁŃÓŚŹŻąćęłńóśźż0-9][^\n.;|]{1,50})""",
            RegexOption.IGNORE_CASE
        ),
    )

    private val merchantTrailingNoiseRegex = Regex(
        """(?:\s+(?:(?:was|were|has been|is)\s+(?:charged|debited|paid|processed|completed|authorized|made|sent|approved)|with|using|via|on|for|at\s+\d|card|visa|mastercard|maestro|amex|ending|••••|\*{3,4}|kart[ąa]|nr\s+karty|w\s+dniu|dnia|saldo|balance|available|ref\.?|id[:#]?)$WB_END.*|\s*[•·|].*)$""",
        RegexOption.IGNORE_CASE
    )

    private val accountOrCardPhraseRegex = Regex(
        """^(?:your|my|the|tw[oó]j[ae]?|moje?)\s+.*(?:card|account|wallet|balance|bank|visa|mastercard|amex|konto|kart[ęąa]|rachunek)$WB_END""",
        RegexOption.IGNORE_CASE
    )

    private val whitespaceRegex = Regex("""\s+""")

    private data class AmountCandidate(
        val amount: Double,
        val currencyCode: String,
        val sign: Char?,
        val startIndex: Int,
        val isBalance: Boolean
    )

    fun parse(
        payload: NotificationPayload,
        includeIncoming: Boolean = true,
        fallbackCurrencyCode: String? = null
    ): DetectedPayment? {
        if (payload.isOngoing || payload.isGroupSummary) return null
        if (KnownPaymentApps.isIgnoredPackage(payload.packageName)) return null

        val cleanTitle = payload.title?.trim().orEmpty()
        val bodyParts = buildList {
            val primaryBody = payload.bigText?.trim().takeIf { !it.isNullOrBlank() }
                ?: payload.text?.trim().takeIf { !it.isNullOrBlank() }
            if (primaryBody != null) add(primaryBody)
            payload.textLines.map { it.trim() }.filter { it.isNotEmpty() }.forEach { line ->
                if (none { it.contains(line, ignoreCase = true) }) {
                    add(line)
                }
            }
            payload.subText?.trim()?.takeIf { it.isNotBlank() }?.let { sub ->
                if (none { it.contains(sub, ignoreCase = true) } &&
                    !cleanTitle.equals(sub, ignoreCase = true)
                ) {
                    add(sub)
                }
            }
        }
        val bodyText = bodyParts.joinToString("\n").trim()
        val combinedText = listOf(cleanTitle, bodyText)
            .filter { it.isNotBlank() }
            .joinToString("\n")

        if (combinedText.isBlank()) return null

        // Reject OTPs, declines, promos, reminders, requests
        if (negativePatterns.any { it.containsMatchIn(combinedText) }) {
            return null
        }

        // Extract monetary amounts
        val candidates = extractAmountCandidates(combinedText)
        if (candidates.isEmpty()) return null

        val primaryCandidate = candidates.firstOrNull { !it.isBalance } ?: return null
        if (primaryCandidate.amount <= 0.0 || primaryCandidate.amount > MAX_REASONABLE_AMOUNT) {
            return null
        }

        val isKnownPaymentApp = KnownPaymentApps.isKnownPaymentApp(payload.packageName)
        val isCuratedApp = KnownPaymentApps.isCuratedPaymentApp(payload.packageName)
        val hasIncomingKeyword = incomingKeywordsRegex.containsMatchIn(combinedText)
        val hasOutgoingKeyword = outgoingKeywordsRegex.containsMatchIn(combinedText)
        val hasWalletCardCue = walletCardCueRegex.containsMatchIn(combinedText)
        val hasNegativeSign = primaryCandidate.sign == '-'
        val hasPositiveSign = primaryCandidate.sign == '+'

        // Determine direction (expense vs income)
        val isExpense = when {
            hasIncomingKeyword -> false
            hasPositiveSign && !hasOutgoingKeyword -> false
            hasOutgoingKeyword -> true
            hasNegativeSign -> true
            else -> true
        }

        if (!isExpense && !includeIncoming) {
            return null
        }

        // Decide whether this notification really represents a payment transaction
        val confidence = when {
            hasIncomingKeyword || hasOutgoingKeyword -> PaymentConfidence.HIGH
            isCuratedApp && (hasWalletCardCue || hasNegativeSign || hasPositiveSign || isLikelyMerchantTitle(cleanTitle, payload)) ->
                PaymentConfidence.HIGH
            isKnownPaymentApp && (hasWalletCardCue || hasNegativeSign || hasPositiveSign) ->
                PaymentConfidence.MEDIUM
            else -> return null
        }

        val resolvedAppName = KnownPaymentApps.displayNameFor(payload.packageName, payload.appName)
        val resolvedCurrency = primaryCandidate.currencyCode.ifBlank {
            fallbackCurrencyCode?.takeIf { it.isNotBlank() }?.let { normalizeCurrencyCode(it) }
        }?.takeIf { it.isNotBlank() }

        val title = extractTitle(
            cleanTitle = cleanTitle,
            bodyText = bodyText,
            combinedText = combinedText,
            appName = resolvedAppName,
            isExpense = isExpense,
            isCuratedApp = isCuratedApp
        )

        val description = buildDescription(
            appName = resolvedAppName,
            cleanTitle = cleanTitle,
            bodyText = bodyText
        )

        return DetectedPayment(
            amount = primaryCandidate.amount,
            currencyCode = resolvedCurrency,
            isExpense = isExpense,
            title = title,
            description = description,
            timestamp = payload.postTime,
            sourcePackage = payload.packageName,
            sourceAppName = resolvedAppName,
            confidence = confidence
        )
    }

    private fun extractAmountCandidates(text: String): List<AmountCandidate> {
        val rawMatches = mutableListOf<Triple<Int, Int, AmountCandidate>>()

        fun addMatch(match: MatchResult, rawNumber: String, rawCurrency: String, sign: Char?) {
            val amount = parseNumber(rawNumber) ?: return
            val currency = resolveCurrency(rawCurrency) ?: return
            val start = match.range.first
            val end = match.range.last
            val prefixContext = text.substring((start - 36).coerceAtLeast(0), start)
            val isBalance = balancePrefixRegex.containsMatchIn(prefixContext)
            rawMatches += Triple(
                start,
                end,
                AmountCandidate(
                    amount = amount,
                    currencyCode = currency,
                    sign = sign,
                    startIndex = start,
                    isBalance = isBalance
                )
            )
        }

        prefixSymbolRegex.findAll(text).forEach { match ->
            val sign = (match.groupValues[1].ifEmpty { match.groupValues[3] }).firstOrNull()
            addMatch(match, match.groupValues[4], match.groupValues[2], sign)
        }
        prefixIsoRegex.findAll(text).forEach { match ->
            val sign = (match.groupValues[1].ifEmpty { match.groupValues[3] }).firstOrNull()
            addMatch(match, match.groupValues[4], match.groupValues[2], sign)
        }
        suffixCurrencyRegex.findAll(text).forEach { match ->
            val sign = match.groupValues[1].firstOrNull()
            addMatch(match, match.groupValues[2], match.groupValues[3], sign)
        }

        // Sort by appearance in text and remove overlapping regex matches (keeping earliest/longest)
        val sorted = rawMatches.sortedWith(compareBy<Triple<Int, Int, AmountCandidate>> { it.first }
            .thenByDescending { it.second - it.first })
        val deduped = mutableListOf<AmountCandidate>()
        var lastEnd = -1
        for ((start, end, candidate) in sorted) {
            if (start <= lastEnd) continue
            deduped += candidate
            lastEnd = end
        }
        return deduped
    }

    private fun resolveCurrency(rawToken: String): String? {
        val trimmed = rawToken.trim()
        if (trimmed.isEmpty()) return null
        val upper = trimmed.uppercase(Locale.US)
        if (upper in isoCurrencyCodes) return upper
        return symbolToCurrency[trimmed.lowercase(Locale.US)]
    }

    internal fun parseNumber(raw: String): Double? {
        val cleaned = raw
            .replace("\u00A0", "")
            .replace(" ", "")
            .trim()
        if (cleaned.isEmpty()) return null

        val lastComma = cleaned.lastIndexOf(',')
        val lastDot = cleaned.lastIndexOf('.')

        val normalized = when {
            lastComma >= 0 && lastDot >= 0 -> {
                if (lastComma > lastDot) {
                    // European: 1.234,56
                    cleaned.replace(".", "").replace(',', '.')
                } else {
                    // US/UK: 1,234.56
                    cleaned.replace(",", "")
                }
            }
            lastComma >= 0 -> {
                val fracLen = cleaned.length - lastComma - 1
                if (cleaned.count { it == ',' } == 1 && fracLen in 1..2) {
                    // Decimal comma: 12,50
                    cleaned.replace(',', '.')
                } else {
                    // Thousands comma: 1,500 or 1,000,000
                    cleaned.replace(",", "")
                }
            }
            lastDot >= 0 -> {
                val fracLen = cleaned.length - lastDot - 1
                if (cleaned.count { it == '.' } > 1 || (fracLen == 3 && cleaned.length > 4)) {
                    // Thousands dots: 1.234.567 or 1.500
                    cleaned.replace(".", "")
                } else {
                    cleaned
                }
            }
            else -> cleaned
        }

        return normalized.toDoubleOrNull()
    }

    private fun isLikelyMerchantTitle(cleanTitle: String, payload: NotificationPayload): Boolean {
        if (cleanTitle.length < 2) return false
        val lower = cleanTitle.lowercase(Locale.ROOT)
        if (lower in genericTitles) return false
        val appName = KnownPaymentApps.displayNameFor(payload.packageName, payload.appName)
        if (appName != null && lower == appName.lowercase(Locale.ROOT)) return false
        // Must contain at least one letter so pure amounts/numbers aren't treated as merchants
        return cleanTitle.any { it.isLetter() }
    }

    private fun extractTitle(
        cleanTitle: String,
        bodyText: String,
        combinedText: String,
        appName: String?,
        isExpense: Boolean,
        isCuratedApp: Boolean
    ): String {
        // For concise wallet alerts like Google Wallet ("Starbucks" / "$5.75 with Visa •••• 1234"),
        // the notification title itself is typically the exact merchant name.
        if (isCuratedApp &&
            isLikelyMerchantTitle(cleanTitle, NotificationPayload(packageName = "", appName = appName)) &&
            !containsAmountOrPaymentPhrase(cleanTitle)
        ) {
            return cleanMerchant(cleanTitle)
        }

        // Try structured merchant extraction from body first, then combined text
        for (source in listOf(bodyText, combinedText)) {
            if (source.isBlank()) continue
            for (pattern in merchantPatterns) {
                for (match in pattern.findAll(source)) {
                    val rawMerchant = match.groupValues.getOrNull(1)?.trim().orEmpty()
                    val cleaned = cleanMerchant(rawMerchant)
                    if (isUsableMerchant(cleaned, appName)) {
                        return cleaned
                    }
                }
            }
        }

        // Fallback to notification title if it isn't generic and doesn't just duplicate app name
        if (isLikelyMerchantTitle(cleanTitle, NotificationPayload(packageName = "", appName = appName)) &&
            !containsAmountOrPaymentPhrase(cleanTitle)
        ) {
            return cleanMerchant(cleanTitle)
        }

        return when {
            !appName.isNullOrBlank() && isExpense -> "$appName payment"
            !appName.isNullOrBlank() -> "$appName income"
            isExpense -> "Payment"
            else -> "Received payment"
        }
    }

    private fun containsAmountOrPaymentPhrase(text: String): Boolean {
        return prefixSymbolRegex.containsMatchIn(text) ||
            prefixIsoRegex.containsMatchIn(text) ||
            suffixCurrencyRegex.containsMatchIn(text) ||
            outgoingKeywordsRegex.containsMatchIn(text) ||
            incomingKeywordsRegex.containsMatchIn(text)
    }

    private fun cleanMerchant(raw: String): String {
        val stripped = raw
            .replace(merchantTrailingNoiseRegex, "")
            .trim(' ', '-', '–', '—', ':', ',', '.', '•', '·', '"', '\'')
            .replace(whitespaceRegex, " ")
            .trim()
        return stripped.take(MAX_TITLE_LENGTH)
    }

    private fun isUsableMerchant(candidate: String, appName: String?): Boolean {
        if (candidate.length < 2) return false
        if (!candidate.any { it.isLetter() }) return false
        val lower = candidate.lowercase(Locale.ROOT)
        if (lower in genericTitles) return false
        if (appName != null && lower == appName.lowercase(Locale.ROOT)) return false
        if (accountOrCardPhraseRegex.containsMatchIn(candidate)) return false
        val stopWords = setOf(
            "your", "your card", "your account", "account", "card", "bank", "you",
            "konto", "kartę", "kartą", "rachunek", "twój", "twoje"
        )
        if (lower in stopWords) return false
        return true
    }

    private fun buildDescription(
        appName: String?,
        cleanTitle: String,
        bodyText: String
    ): String {
        val snippet = when {
            bodyText.isNotBlank() -> bodyText.replace('\n', ' ').replace(whitespaceRegex, " ").trim()
            cleanTitle.isNotBlank() -> cleanTitle.replace(whitespaceRegex, " ").trim()
            else -> ""
        }
        val prefix = if (!appName.isNullOrBlank()) "Detected from $appName" else "Detected payment"
        val full = if (snippet.isNotBlank()) "$prefix: $snippet" else prefix
        return if (full.length > MAX_DESCRIPTION_LENGTH) {
            full.take(MAX_DESCRIPTION_LENGTH - 1).trimEnd() + "…"
        } else {
            full
        }
    }
}
