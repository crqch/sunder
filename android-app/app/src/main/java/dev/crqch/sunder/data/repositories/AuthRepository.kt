package dev.crqch.sunder.data.repositories

import android.content.Context
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

private val Context.authDataStore by preferencesDataStore(name = "auth_prefs")

class AuthRepository(private val context: Context) {
    companion object {
        private val REFRESH_TOKEN = stringPreferencesKey("refresh_token")
        private val ACCESS_TOKEN = stringPreferencesKey("access_token")
    }

    val accessToken: Flow<String?> = context.authDataStore.data.map { it[ACCESS_TOKEN] }

    val refreshToken: Flow<String?> = context.authDataStore.data.map { it[REFRESH_TOKEN] }

    suspend fun saveTokens(access: String, refresh: String) {
        context.authDataStore.edit { preferences ->
            preferences[ACCESS_TOKEN] = access
            preferences[REFRESH_TOKEN] = refresh
        }
    }

    suspend fun clearTokens() {
        context.authDataStore.edit {
            it.clear()
        }
    }
}
