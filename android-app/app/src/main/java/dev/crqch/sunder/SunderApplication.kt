package dev.crqch.sunder

import android.app.Application
import dagger.hilt.android.HiltAndroidApp
import dev.crqch.sunder.data.local.SunderDatabase
import dev.crqch.sunder.data.repositories.AuthRepository
import dev.crqch.sunder.data.repositories.EntryRepository
import dev.crqch.sunder.data.repositories.UserRepository

@HiltAndroidApp
class SunderApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        instance = this
    }

    val database by lazy { SunderDatabase.getDatabase(this) }

    companion object {
        lateinit var instance: SunderApplication
            private set
    }
}
