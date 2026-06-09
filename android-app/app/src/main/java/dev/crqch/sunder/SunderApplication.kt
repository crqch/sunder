package dev.crqch.sunder

import android.app.Application
import androidx.hilt.work.HiltWorkerFactory
import androidx.work.Configuration
import dagger.hilt.android.HiltAndroidApp
import dev.crqch.sunder.data.local.SunderDatabase
import dev.crqch.sunder.data.repositories.AuthRepository
import dev.crqch.sunder.data.repositories.EntryRepository
import dev.crqch.sunder.data.repositories.UserRepository
import javax.inject.Inject

@HiltAndroidApp
class SunderApplication : Application(), Configuration.Provider {
    @Inject
    lateinit var workerFactory: HiltWorkerFactory

    override fun onCreate() {
        super.onCreate()
        instance = this
    }

    override val workManagerConfiguration: Configuration
        get() = Configuration.Builder()
            .setWorkerFactory(workerFactory)
            .build()

    val database by lazy { SunderDatabase.getDatabase(this) }

    companion object {
        lateinit var instance: SunderApplication
            private set
    }
}
