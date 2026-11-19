package dev.crqch.sunder.di

import android.content.Context
import androidx.room.Room
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import dev.crqch.sunder.data.local.AccountDao
import dev.crqch.sunder.data.local.CategoryDao
import dev.crqch.sunder.data.local.CurrencyDao
import dev.crqch.sunder.data.local.EntryDao
import dev.crqch.sunder.data.local.ExchangeRateDao
import dev.crqch.sunder.data.local.SunderDatabase
import dev.crqch.sunder.data.local.SyncMetadataDao
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
object DatabaseModule {

    @Provides
    @Singleton
    fun provideDatabase(@ApplicationContext context: Context): SunderDatabase {
        return Room.databaseBuilder(
            context.applicationContext,
            SunderDatabase::class.java,
            SunderDatabase.DATABASE_NAME
        )
            .addMigrations(SunderDatabase.MIGRATION_3_4)
            .fallbackToDestructiveMigrationFrom(1, 2)
            .build()
    }

    @Provides
    fun provideAccountDao(database: SunderDatabase): AccountDao {
        return database.accountDao()
    }

    @Provides
    fun provideEntryDao(database: SunderDatabase): EntryDao {
        return database.entryDao()
    }

    @Provides
    fun provideCategoryDao(database: SunderDatabase): CategoryDao {
        return database.categoryDao()
    }

    @Provides
    fun provideSyncMetadataDao(database: SunderDatabase): SyncMetadataDao {
        return database.syncMetadataDao()
    }

    @Provides
    fun provideCurrencyDao(database: SunderDatabase): CurrencyDao {
        return database.currencyDao()
    }

    @Provides
    fun provideExchangeRateDao(database: SunderDatabase): ExchangeRateDao {
        return database.exchangeRateDao()
    }
}
