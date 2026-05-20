package dev.crqch.sunder.data.local

import android.content.Context
import androidx.room.Database
import androidx.room.Room
import androidx.room.RoomDatabase
import kotlin.jvm.java

@Database(entities = [EntryEntity::class, CategoryEntity::class], version = 1)
abstract class SunderDatabase : RoomDatabase() {
    abstract fun entryDao(): EntryDao
    abstract fun categoryDao(): CategoryDao

    companion object {
        @Volatile
        private var INSTANCE: SunderDatabase? = null

        fun getDatabase(context: Context): SunderDatabase {
            return INSTANCE ?: synchronized(this) {
                val instance = Room.databaseBuilder(
                    context.applicationContext,
                    SunderDatabase::class.java,
                    "sunder_database"
                ).build()
                INSTANCE = instance
                instance
            }
        }
    }
}
