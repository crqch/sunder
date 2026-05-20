package dev.crqch.sunder.di

import android.content.Context
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import dev.crqch.sunder.data.local.AccountDao
import dev.crqch.sunder.data.local.CategoryDao
import dev.crqch.sunder.data.local.EntryDao
import dev.crqch.sunder.data.local.SunderDatabase
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
object DatabaseModule {

    @Provides
    @Singleton
    fun provideDatabase(@ApplicationContext context: Context): SunderDatabase {
        return SunderDatabase.getDatabase(context)
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
}
