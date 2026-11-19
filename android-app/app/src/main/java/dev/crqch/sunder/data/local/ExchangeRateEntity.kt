package dev.crqch.sunder.data.local

import androidx.room.ColumnInfo
import androidx.room.Entity

/**
 * One currency's rate against the canonical USD base on a given day.
 *
 * Rates are stored exactly as returned by the server: 1 USD == [rate] units of
 * [currencyCode]. The row for [currencyCode] == "USD" is always 1.0.
 */
@Entity(
    tableName = "exchange_rates",
    primaryKeys = ["rate_date", "currency_code"]
)
data class ExchangeRateEntity(
    @ColumnInfo(name = "rate_date") val rateDate: String,
    @ColumnInfo(name = "currency_code") val currencyCode: String,
    @ColumnInfo(name = "rate") val rate: Double
)
