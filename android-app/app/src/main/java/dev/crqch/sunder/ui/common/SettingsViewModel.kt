package dev.crqch.sunder.ui.common

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.crqch.sunder.data.local.SunderDatabase
import dev.crqch.sunder.data.repositories.AppSettings
import dev.crqch.sunder.data.repositories.PreferencesRepository
import dev.crqch.sunder.data.repositories.SyncRepository
import dev.crqch.sunder.data.repositories.ThemeMode
import dev.crqch.sunder.data.repositories.UserRepository
import dev.crqch.sunder.data.sync.SyncManager
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import javax.inject.Inject

@HiltViewModel
class SettingsViewModel @Inject constructor(
    private val preferencesRepository: PreferencesRepository,
    private val syncRepository: SyncRepository,
    private val syncManager: SyncManager,
    private val userRepository: UserRepository,
    private val database: SunderDatabase
) : ViewModel() {

    val settings: StateFlow<AppSettings> = preferencesRepository.settings
    val currentUser = userRepository.currentUser
    val isSyncing = syncRepository.isSyncing

    val lastSyncedAt: StateFlow<Long?> = syncRepository.lastSyncedAt.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = null
    )

    private val _unsyncedCount = MutableStateFlow(0)
    val unsyncedCount: StateFlow<Int> = _unsyncedCount.asStateFlow()

    private val _snackbarMessage = MutableStateFlow<String?>(null)
    val snackbarMessage: StateFlow<String?> = _snackbarMessage.asStateFlow()

    private val _showUnsyncedLogoutDialog = MutableStateFlow(false)
    val showUnsyncedLogoutDialog: StateFlow<Boolean> = _showUnsyncedLogoutDialog.asStateFlow()

    init {
        refreshUnsyncedCount()
    }

    fun refreshUnsyncedCount() {
        viewModelScope.launch {
            _unsyncedCount.value = withContext(Dispatchers.IO) {
                syncRepository.getUnsyncedCount()
            }
        }
    }

    fun setThemeMode(mode: ThemeMode) {
        viewModelScope.launch {
            preferencesRepository.setThemeMode(mode)
        }
    }

    fun setDynamicColor(enabled: Boolean) {
        viewModelScope.launch {
            preferencesRepository.setDynamicColor(enabled)
        }
    }

    fun setCurrencySymbol(symbol: String) {
        viewModelScope.launch {
            preferencesRepository.setCurrencySymbol(symbol)
        }
    }

    fun setCurrencyAfter(after: Boolean) {
        viewModelScope.launch {
            preferencesRepository.setCurrencyAfter(after)
        }
    }

    fun setAutoSyncEnabled(enabled: Boolean) {
        viewModelScope.launch {
            preferencesRepository.setAutoSyncEnabled(enabled)
            if (enabled) {
                syncManager.schedulePeriodicSync(
                    intervalMinutes = settings.value.autoSyncIntervalMinutes,
                    replaceExisting = true
                )
            } else {
                syncManager.cancelPeriodicSync()
            }
        }
    }

    fun setAutoSyncInterval(minutes: Long) {
        viewModelScope.launch {
            preferencesRepository.setAutoSyncInterval(minutes)
            if (settings.value.autoSyncEnabled) {
                syncManager.schedulePeriodicSync(
                    intervalMinutes = minutes,
                    replaceExisting = true
                )
            }
        }
    }

    fun syncNow(successMsg: String, failMsg: String) {
        if (isSyncing.value) return
        viewModelScope.launch {
            val result = withContext(Dispatchers.IO) {
                syncRepository.syncNow()
            }
            refreshUnsyncedCount()
            _snackbarMessage.value = if (result.isSuccess) successMsg else failMsg
        }
    }

    fun beginLogout(onComplete: () -> Unit) {
        viewModelScope.launch {
            val count = withContext(Dispatchers.IO) {
                syncRepository.getUnsyncedCount()
            }
            _unsyncedCount.value = count
            if (count > 0) {
                _showUnsyncedLogoutDialog.value = true
            } else {
                executeLogout(onComplete)
            }
        }
    }

    fun dismissLogoutDialog() {
        _showUnsyncedLogoutDialog.value = false
    }

    fun syncAndLogout(failMsg: String, onComplete: () -> Unit) {
        viewModelScope.launch {
            val result = withContext(Dispatchers.IO) {
                syncRepository.syncNow()
            }
            if (result.isSuccess) {
                _showUnsyncedLogoutDialog.value = false
                executeLogout(onComplete)
            } else {
                _snackbarMessage.value = failMsg
            }
        }
    }

    fun discardAndLogout(onComplete: () -> Unit) {
        _showUnsyncedLogoutDialog.value = false
        viewModelScope.launch {
            executeLogout(onComplete)
        }
    }

    private suspend fun executeLogout(onComplete: () -> Unit) {
        withContext(Dispatchers.IO) {
            syncManager.cancelPeriodicSync()
            database.clearAllTables()
            userRepository.logout()
        }
        onComplete()
    }

    fun clearSnackbar() {
        _snackbarMessage.value = null
    }
}
