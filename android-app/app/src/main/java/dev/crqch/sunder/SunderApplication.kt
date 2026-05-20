package dev.crqch.sunder

import android.app.Application
import dev.crqch.sunder.data.local.SunderDatabase
import dev.crqch.sunder.data.repositories.AuthRepository
import dev.crqch.sunder.data.repositories.EntryRepository
import dev.crqch.sunder.data.repositories.UserRepository

class SunderApplication : Application() {
    val authRepository by lazy { AuthRepository(this) }
    val userRepository by lazy { UserRepository(authRepository) }

    val database by lazy { SunderDatabase.getDatabase(this) }
    val entryRepository by lazy { EntryRepository(database.entryDao()) }
}
