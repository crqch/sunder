package dev.crqch.sunder.data.repositories

import android.content.Context
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import dagger.hilt.android.qualifiers.ApplicationContext
import dev.crqch.sunder.BuildConfig
import dev.crqch.sunder.api.DefaultApi
import dev.crqch.sunder.models.AuthRefreshTokenTHN4OMARequest
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import org.json.JSONObject
import javax.inject.Inject
import javax.inject.Singleton

private val Context.authDataStore by preferencesDataStore(name = "auth_prefs")

@Singleton
class AuthRepository @Inject constructor(
    private val api: DefaultApi,
    @ApplicationContext private val context: Context
) {
    private val scope = CoroutineScope(Dispatchers.IO)

    companion object {
        private val REFRESH_TOKEN = stringPreferencesKey("refresh_token")
        private val ACCESS_TOKEN = stringPreferencesKey("access_token")
    }

    val accessTokenFlow: Flow<String?> = context.authDataStore.data.map { it[ACCESS_TOKEN] }
    val refreshTokenFlow: Flow<String?> = context.authDataStore.data.map { it[REFRESH_TOKEN] }

    val accessToken: StateFlow<String?> = accessTokenFlow.stateIn(
        scope = scope,
        started = SharingStarted.Eagerly,
        initialValue = null
    )

    val refreshToken: StateFlow<String?> = refreshTokenFlow.stateIn(
        scope = scope,
        started = SharingStarted.Eagerly,
        initialValue = null
    )

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

    suspend fun refreshTokens(): String? {
        val refresh = refreshTokenFlow.first() ?: return null

        return try {
            val response = api.authRefreshTokenTHN4OMA(
                AuthRefreshTokenTHN4OMARequest(token = refresh)
            )
            if (response.isSuccessful) {
                val body = response.body()
                val newAccess = body?.accessToken ?: return null
                saveTokens(newAccess, refresh)
                newAccess
            } else {
                null
            }
        } catch (e: Exception) {
            null
        }
    }
}
