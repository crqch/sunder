package dev.crqch.sunder.notifications

/**
 * Registry of well-known payment wallets, fintechs, and banking apps along with
 * packages that should always be ignored to avoid noise and false positives.
 */
object KnownPaymentApps {

    /** Known payment / wallet / banking packages mapped to a clean fallback display name. */
    val knownApps: Map<String, String> = mapOf(
        // Wallets & P2P
        "com.google.android.apps.walletnfcrel" to "Google Wallet",
        "com.google.android.apps.nbu.paisa.user" to "Google Pay",
        "com.paypal.android.p2pmobile" to "PayPal",
        "com.venmo" to "Venmo",
        "com.squareup.cash" to "Cash App",
        "com.samsung.android.spay" to "Samsung Wallet",
        "com.revolut.revolut" to "Revolut",
        "com.transferwise.android" to "Wise",
        "com.imaginecurve.curve.prd" to "Curve",
        "com.myklarnamobile" to "Klarna",
        "com.zellepay.zelle" to "Zelle",
        "com.payoneer.android" to "Payoneer",
        "com.skrill.skrillpayments" to "Skrill",
        "com.zen.android" to "ZEN",
        "com.eg.android.AlipayGphone" to "Alipay",
        "com.phonepe.app" to "PhonePe",
        "net.one97.paytm" to "Paytm",
        "com.stripe.android.dashboard" to "Stripe",

        // US Banks
        "com.chase.sig.android" to "Chase",
        "com.infonow.bofa" to "Bank of America",
        "com.wf.wellsfargomobile" to "Wells Fargo",
        "com.citi.citimobile" to "Citi",
        "com.konylabs.capitalone" to "Capital One",
        "com.americanexpress.android.acctsvcs.us" to "American Express",
        "com.discoverfinancial.mobile" to "Discover",
        "com.usbank.mobilebanking" to "U.S. Bank",
        "com.onedebit.chime" to "Chime",
        "com.sofi.mobile" to "SoFi",
        "com.ally.MobileBanking" to "Ally",
        "com.pnc.ecommerce.mobile" to "PNC",
        "com.tdbank" to "TD Bank",

        // UK & EU Banks
        "co.uk.getmondo" to "Monzo",
        "com.starlingbank.android" to "Starling Bank",
        "de.number26.android" to "N26",
        "com.bunq.android" to "bunq",
        "com.barclays.android.barclaysmobilebanking" to "Barclays",
        "uk.co.hsbc.hsbcukmobilebanking" to "HSBC",
        "com.grppl.android.shell.CMBlloydsTSB73" to "Lloyds",
        "com.rbs.mobile.android.natwest" to "NatWest",
        "com.db.pwcc.dbmobile" to "Deutsche Bank",
        "com.starfinanz.smob.android.sfinanzstatus" to "Sparkasse",
        "de.commerzbanking.mobil" to "Commerzbank",
        "com.ing.mobile" to "ING",
        "com.ing.diba.mbbr2" to "ING",
        "de.traderepublic.app" to "Trade Republic",

        // Polish Banks & Payments
        "pl.mbank" to "mBank",
        "pl.pkobp.iko" to "IKO",
        "pl.ing.mojeing" to "Moje ING",
        "pl.bzwbk.bzwbk24" to "Santander",
        "eu.eleader.mobilebanking.pekao" to "PeoPay",
        "wit.android.bcpBankingApp.millenniumPL" to "Bank Millennium",
        "pl.aliorbank.aib" to "Alior Bank",
        "pl.bnpparibas.gomobile" to "BNP Paribas",
        "com.finanteq.finance.ca" to "Credit Agricole",
        "pl.creditagricole.ca24" to "Credit Agricole",
        "pl.velobank.mobile" to "VeloBank",
        "pl.nestbank.mobile" to "Nest Bank",
        "pl.cinkciarz.mobile" to "Cinkciarz",
    )

    /**
     * Packages whose notifications are never payment confirmations
     * (Sunder itself, OS UI, chat/social/media apps).
     */
    private val ignoredPackages: Set<String> = setOf(
        "dev.crqch.sunder",
        "dev.crqch.sunder.debug",
        "android",
        "com.android.systemui",
        "com.android.settings",
        "com.android.dialer",
        "com.android.deskclock",
        "com.android.calendar",
        "com.google.android.gms",
        "com.google.android.gsf",
        "com.google.android.deskclock",
        "com.google.android.calendar",
        "com.google.android.youtube",
        "com.google.android.apps.youtube.music",
        "com.spotify.music",
        // Social & messaging apps (chat texts like "I paid $20" are not payment alerts)
        "com.whatsapp",
        "com.whatsapp.w4b",
        "org.telegram.messenger",
        "org.telegram.messenger.web",
        "com.discord",
        "com.instagram.android",
        "com.facebook.katana",
        "com.facebook.orca",
        "com.snapchat.android",
        "com.twitter.android",
        "com.reddit.frontpage",
        "com.Slack",
        "com.microsoft.teams",
        "org.thoughtcrime.securesms",
        "com.viber.voip",
        "com.zhiliaoapp.musically",
    )

    private val ignoredPackagePrefixes: List<String> = listOf(
        "dev.crqch.sunder",
        "com.android.providers.",
    )

    private val bankingPackageTokenRegex = Regex(
        "(?:^|[._-])(bank|banking|wallet|pay|payment|payments|fintech|revolut|paypal|venmo|blik|monzo|santander|pekao|mbank|alior|millennium|sparkasse|barclays|hsbc|chase|citi|amex|klarna|curve|wise)(?:[._-]|$)",
        RegexOption.IGNORE_CASE
    )

    fun isIgnoredPackage(packageName: String): Boolean {
        if (packageName.isBlank()) return true
        if (packageName in ignoredPackages) return true
        return ignoredPackagePrefixes.any { packageName.startsWith(it) }
    }

    fun isKnownPaymentApp(packageName: String): Boolean =
        packageName in knownApps || looksLikeBankingPackage(packageName)

    fun isCuratedPaymentApp(packageName: String): Boolean =
        packageName in knownApps

    fun looksLikeBankingPackage(packageName: String): Boolean {
        if (isIgnoredPackage(packageName)) return false
        return bankingPackageTokenRegex.containsMatchIn(packageName)
    }

    fun displayNameFor(packageName: String, fallbackAppName: String? = null): String? {
        return knownApps[packageName] ?: fallbackAppName?.takeIf { it.isNotBlank() }
    }
}
