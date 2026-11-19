package dev.crqch.sunder.data.local

import android.content.Context
import androidx.room.Database
import androidx.room.Room
import androidx.room.RoomDatabase
import androidx.room.migration.Migration
import androidx.sqlite.db.SupportSQLiteDatabase
import kotlin.jvm.java

@Database(
    entities = [
        EntryEntity::class,
        CategoryEntity::class,
        AccountEntity::class,
        SyncMetadataEntity::class,
        CurrencyEntity::class,
        ExchangeRateEntity::class
    ],
    version = 4
)
abstract class SunderDatabase : RoomDatabase() {
    abstract fun entryDao(): EntryDao
    abstract fun categoryDao(): CategoryDao

    abstract fun accountDao(): AccountDao
    abstract fun syncMetadataDao(): SyncMetadataDao

    abstract fun currencyDao(): CurrencyDao
    abstract fun exchangeRateDao(): ExchangeRateDao

    companion object {
        const val DATABASE_NAME = "sunder_database"

        /**
         * Adds multi-currency storage while preserving every existing local row.
         * Entries default to USD, accounts stay unset until a currency is chosen.
         */
        val MIGRATION_3_4 = object : Migration(3, 4) {
            override fun migrate(db: SupportSQLiteDatabase) {
                db.execSQL(
                    "ALTER TABLE entries ADD COLUMN currency_code TEXT NOT NULL DEFAULT 'USD'"
                )
                db.execSQL("ALTER TABLE accounts ADD COLUMN currency_code TEXT")
                db.execSQL(
                    """
                    CREATE TABLE IF NOT EXISTS currencies (
                        code TEXT NOT NULL,
                        name TEXT NOT NULL,
                        symbol TEXT NOT NULL,
                        symbol_after INTEGER NOT NULL,
                        decimal_digits INTEGER NOT NULL,
                        is_active INTEGER NOT NULL,
                        PRIMARY KEY(code)
                    )
                    """.trimIndent()
                )
                db.execSQL(
                    """
                    CREATE TABLE IF NOT EXISTS exchange_rates (
                        rate_date TEXT NOT NULL,
                        currency_code TEXT NOT NULL,
                        rate REAL NOT NULL,
                        PRIMARY KEY(rate_date, currency_code)
                    )
                    """.trimIndent()
                )
            }
        }

        @Volatile
        private var INSTANCE: SunderDatabase? = null

        fun getDatabase(context: Context): SunderDatabase {
            return INSTANCE ?: synchronized(this) {
                val instance = Room.databaseBuilder(
                    context.applicationContext,
                    SunderDatabase::class.java,
                    DATABASE_NAME
                )
                    .addMigrations(MIGRATION_3_4)
                    .build()
                INSTANCE = instance
                instance
            }
        }
    }
}
