package dev.crqch.sunder.data.repositories

import android.util.Log
import androidx.room.withTransaction
import dev.crqch.sunder.api.SyncApi
import dev.crqch.sunder.data.local.*
import dev.crqch.sunder.data.sync.*
import dev.crqch.sunder.utils.normalizeCurrencyCode
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import java.time.Instant
import java.time.ZoneOffset
import java.time.format.DateTimeFormatter
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class SyncRepository @Inject constructor(
    private val database: SunderDatabase,
    private val syncApi: SyncApi,
    private val accountDao: AccountDao,
    private val categoryDao: CategoryDao,
    private val entryDao: EntryDao,
    private val syncMetadataDao: SyncMetadataDao,
    private val preferencesRepository: PreferencesRepository,
    private val currencyRepository: CurrencyRepository
) {
    private val formatter = DateTimeFormatter.ISO_INSTANT.withZone(ZoneOffset.UTC)

    val lastSyncedAt: Flow<Long?> = syncMetadataDao.observeLastSyncedAt()

    private val _isSyncing = MutableStateFlow(false)
    val isSyncing: StateFlow<Boolean> = _isSyncing.asStateFlow()

    suspend fun performSync() {
        _isSyncing.value = true
        val lastSyncedAt = syncMetadataDao.getLastSyncedAt() ?: 0L
        val syncStartTime = System.currentTimeMillis()

        val dirtyAccounts = accountDao.getModifiedSince(lastSyncedAt)
        val dirtyCategories = categoryDao.getModifiedSince(lastSyncedAt)
        val dirtyEntries = entryDao.getModifiedSince(lastSyncedAt)

        val request = SyncRequestDto(
            accounts = dirtyAccounts.map { it.toDto() },
            categories = dirtyCategories.map { it.toDto() },
            entries = dirtyEntries.map { it.toDto() }
        )

        val lastTimestamp = if (lastSyncedAt == 0L) "1970-01-01T00:00:00Z" else formatter.format(
            Instant.ofEpochMilli(lastSyncedAt)
        ) ?: "1970-01-01T00:00:00Z"

        try {
            val response = syncApi.sync(lastTimestamp, request)
            if (response.isSuccessful) {
                val data = response.body()
                if (data == null) {
                    Log.e("SyncRepository", "Sync successful but response body is null")
                    return
                }

                database.withTransaction {
                    if (data.accounts.isNotEmpty()) accountDao.upsertAll(data.accounts.map { it.toEntity() })
                    if (data.categories.isNotEmpty()) categoryDao.upsertAll(data.categories.map { it.toEntity() })
                    if (data.entries.isNotEmpty()) entryDao.upsertAll(data.entries.map { it.toEntity() })

                    syncMetadataDao.saveMetadata(
                        SyncMetadataEntity(
                            id = 0,
                            lastSyncedAt = syncStartTime
                        )
                    )
                }

                syncCurrencyData(data.defaultCurrencyCode)
            } else {
                Log.e(
                    "SyncRepository",
                    "Sync failed with code: ${response.code()}, error: ${
                        response.errorBody()?.string()
                    }"
                )
                throw Exception("Sync failed: ${response.code()}")
            }
        } catch (e: Exception) {
            Log.e("SyncRepository", "Exception during sync: ${e.message}", e)
            throw e
        } finally {
            _isSyncing.value = false
        }
    }

    suspend fun syncNow(): Result<Unit> {
        return try {
            performSync()
            Result.success(Unit)
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    /**
     * Refreshes the currency catalog and historical rates after a successful sync.
     * Every step is best-effort so an offline device never fails the data sync.
     */
    private suspend fun syncCurrencyData(defaultCurrencyCode: String?) {
        runCatching { currencyRepository.refreshCurrencies() }

        val hasPendingLocalPreference = runCatching {
            preferencesRepository.hasPendingDefaultCurrencySync()
        }.getOrDefault(false)

        if (hasPendingLocalPreference) {
            runCatching { currencyRepository.pushPendingDefaultCurrency() }
        } else if (defaultCurrencyCode != null) {
            val normalized = normalizeCurrencyCode(defaultCurrencyCode)
            runCatching {
                preferencesRepository.setDefaultCurrencyCode(normalized)
                currencyRepository.getCurrency(normalized)?.let {
                    preferencesRepository.setCurrencyMetadata(it.symbol, it.symbolAfter)
                }
            }
        }

        runCatching { currencyRepository.refreshRates() }
    }

    suspend fun resetSync() {
        syncMetadataDao.saveMetadata(SyncMetadataEntity(lastSyncedAt = null))
    }

    suspend fun getUnsyncedCount(): Int {
        val lastSyncedAt = syncMetadataDao.getLastSyncedAt() ?: 0L
        return accountDao.getModifiedSince(lastSyncedAt).size +
                categoryDao.getModifiedSince(lastSyncedAt).size +
                entryDao.getModifiedSince(lastSyncedAt).size
    }

    suspend fun hasUnsyncedData(): Boolean {
        return getUnsyncedCount() > 0
    }
}
