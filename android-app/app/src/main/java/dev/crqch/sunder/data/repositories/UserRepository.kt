package dev.crqch.sunder.data.repositories

import android.content.Context
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import dagger.hilt.android.qualifiers.ApplicationContext
import dev.crqch.sunder.api.DefaultApi
import dev.crqch.sunder.data.local.User
import dev.crqch.sunder.data.sync.SyncManager
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import javax.inject.Inject
import javax.inject.Singleton

private val Context.userDataStore by preferencesDataStore(name = "user_prefs")

@Singleton
class UserRepository @Inject constructor(
    private val authRepository: AuthRepository,
    private val api: DefaultApi,
    private val syncManager: SyncManager,
    private val syncRepository: SyncRepository,
    @ApplicationContext private val context: Context
) {
    private val scope = CoroutineScope(Dispatchers.IO)

    companion object {
        private val USER_DATA = stringPreferencesKey("user_data")
    }

    val currentUser: StateFlow<User?> = context.userDataStore.data
        .map { preferences ->
            preferences[USER_DATA]?.let { json ->
                try {
                    Json.decodeFromString<User>(json)
                } catch (e: Exception) {
                    null
                }
            }
        }
        .stateIn(
            scope = scope,
            started = SharingStarted.Eagerly,
            initialValue = null
        )

    suspend fun fetchCurrentUser() {
        try {
            val res = api.dashboardIndexFHMHL3Q()
            val body = res.body()

            if (res.isSuccessful && body != null) {
                val user = User(
                    username = body.username ?: "",
                    email = body.email ?: "",
                    flags = body.flags ?: emptyList()
                )
                saveUserLocal(user)
                syncManager.triggerSync()
                syncManager.schedulePeriodicSync()
            } else if (res.code() == 401 || res.code() == 403) {
                clearUserLocal()
            }
        } catch (e: Exception) {
            // Keep existing data on network error or other exceptions
            // If we have local user, trigger sync anyway
            if (currentUser.value != null) {
                syncManager.triggerSync()
                syncManager.schedulePeriodicSync()
            }
        }
    }

    private suspend fun saveUserLocal(user: User) {
        context.userDataStore.edit { preferences ->
            preferences[USER_DATA] = Json.encodeToString(user)
        }
    }

    private suspend fun clearUserLocal() {
        context.userDataStore.edit { preferences ->
            preferences.remove(USER_DATA)
        }
    }

    suspend fun logout() {
        authRepository.clearTokens()
        clearUserLocal()
        syncRepository.resetSync()
    }
}
