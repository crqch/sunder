package dev.crqch.sunder.data.repositories

import android.content.Context
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.longPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import dev.crqch.sunder.utils.normalizeCurrencyCode
import javax.inject.Inject
import javax.inject.Singleton

private val Context.settingsDataStore by preferencesDataStore(name = "sunder_settings")

enum class ThemeMode {
    SYSTEM,
    LIGHT,
    DARK
}

data class AppSettings(
    val themeMode: ThemeMode = ThemeMode.SYSTEM,
    val dynamicColor: Boolean = false,
    val currencySymbol: String = "",
    val currencyAfter: Boolean = false,
    val defaultCurrencyCode: String = "USD",
    val autoSyncEnabled: Boolean = true,
    val autoSyncIntervalMinutes: Long = 60L
)

@Singleton
class PreferencesRepository @Inject constructor(
    @ApplicationContext private val context: Context
) {
    private val scope = CoroutineScope(Dispatchers.IO)

    companion object {
        private val THEME_MODE = stringPreferencesKey("theme_mode")
        private val DYNAMIC_COLOR = booleanPreferencesKey("dynamic_color")
        private val CURRENCY_SYMBOL = stringPreferencesKey("currency_symbol")
        private val CURRENCY_AFTER = booleanPreferencesKey("currency_after")
        private val DEFAULT_CURRENCY_CODE = stringPreferencesKey("default_currency_code")
        private val AUTO_SYNC_ENABLED = booleanPreferencesKey("auto_sync_enabled")
        private val AUTO_SYNC_INTERVAL = longPreferencesKey("auto_sync_interval")
    }

    val defaultCurrencyCode: Flow<String> = context.settingsDataStore.data
        .map { prefs -> normalizeCurrencyCode(prefs[DEFAULT_CURRENCY_CODE]) }

    val settings: StateFlow<AppSettings> = context.settingsDataStore.data
        .map { prefs ->
            val mode = prefs[THEME_MODE]?.let { name ->
                ThemeMode.entries.find { it.name == name }
            } ?: ThemeMode.SYSTEM

            AppSettings(
                themeMode = mode,
                dynamicColor = prefs[DYNAMIC_COLOR] ?: false,
                currencySymbol = prefs[CURRENCY_SYMBOL] ?: "",
                currencyAfter = prefs[CURRENCY_AFTER] ?: false,
                defaultCurrencyCode = normalizeCurrencyCode(prefs[DEFAULT_CURRENCY_CODE]),
                autoSyncEnabled = prefs[AUTO_SYNC_ENABLED] ?: true,
                autoSyncIntervalMinutes = prefs[AUTO_SYNC_INTERVAL] ?: 60L
            )
        }
        .stateIn(
            scope = scope,
            started = SharingStarted.Eagerly,
            initialValue = AppSettings()
        )

    suspend fun setThemeMode(mode: ThemeMode) {
        context.settingsDataStore.edit { it[THEME_MODE] = mode.name }
    }

    suspend fun setDynamicColor(enabled: Boolean) {
        context.settingsDataStore.edit { it[DYNAMIC_COLOR] = enabled }
    }

    suspend fun setCurrencySymbol(symbol: String) {
        context.settingsDataStore.edit { it[CURRENCY_SYMBOL] = symbol.take(5) }
    }

    suspend fun setCurrencyAfter(after: Boolean) {
        context.settingsDataStore.edit { it[CURRENCY_AFTER] = after }
    }

    /** Updates the legacy symbol/position settings so existing screens keep working. */
    suspend fun setCurrencyMetadata(symbol: String, symbolAfter: Boolean) {
        context.settingsDataStore.edit { prefs ->
            prefs[CURRENCY_SYMBOL] = symbol.take(5)
            prefs[CURRENCY_AFTER] = symbolAfter
        }
    }

    suspend fun setDefaultCurrencyCode(code: String) {
        context.settingsDataStore.edit { it[DEFAULT_CURRENCY_CODE] = normalizeCurrencyCode(code) }
    }

    suspend fun setAutoSyncEnabled(enabled: Boolean) {
        context.settingsDataStore.edit { it[AUTO_SYNC_ENABLED] = enabled }
    }

    suspend fun setAutoSyncInterval(minutes: Long) {
        context.settingsDataStore.edit { it[AUTO_SYNC_INTERVAL] = minutes }
    }
}
