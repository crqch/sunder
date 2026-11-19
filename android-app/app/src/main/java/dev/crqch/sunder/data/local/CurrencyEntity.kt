package dev.crqch.sunder.data.local

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.PrimaryKey

/**
 * A server-allowed currency. Metadata is always fetched from the server and cached
 * locally; the client never ships a static catalog of its own.
 */
@Entity(tableName = "currencies")
data class CurrencyEntity(
    @PrimaryKey val code: String,
    @ColumnInfo(name = "name") val name: String,
    @ColumnInfo(name = "symbol") val symbol: String,
    @ColumnInfo(name = "symbol_after") val symbolAfter: Boolean = false,
    @ColumnInfo(name = "decimal_digits") val decimalDigits: Int = 2,
    @ColumnInfo(name = "is_active") val isActive: Boolean = true
)
