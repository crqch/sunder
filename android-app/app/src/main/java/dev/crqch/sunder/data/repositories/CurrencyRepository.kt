package dev.crqch.sunder.data.repositories

import android.util.Log
import androidx.room.withTransaction
import dev.crqch.sunder.api.AdminApi
import dev.crqch.sunder.api.AdminCurrencyDto
import dev.crqch.sunder.api.CreateCurrencyRequest
import dev.crqch.sunder.api.CurrencyApi
import dev.crqch.sunder.api.CurrencyResponseDto
import dev.crqch.sunder.api.PreferencesApi
import dev.crqch.sunder.api.UpdateCurrencyRequest
import dev.crqch.sunder.api.UpdatePreferencesRequest
import dev.crqch.sunder.api.UpsertRateRequest
import dev.crqch.sunder.data.local.CurrencyDao
import dev.crqch.sunder.data.local.CurrencyEntity
import dev.crqch.sunder.data.local.EntryDao
import dev.crqch.sunder.data.local.ExchangeRateDao
import dev.crqch.sunder.data.local.ExchangeRateEntity
import dev.crqch.sunder.data.local.SunderDatabase
import dev.crqch.sunder.utils.CANONICAL_BASE_CURRENCY
import dev.crqch.sunder.utils.CurrencyConverter
import dev.crqch.sunder.utils.CurrencyUiState
import dev.crqch.sunder.utils.normalizeCurrencyCode
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneOffset
import javax.inject.Inject
import javax.inject.Singleton

data class RateSyncSummary(
    val success: Boolean,
    val date: String? = null,
    val from: String? = null,
    val to: String? = null,
    val source: String? = null,
    val storedCount: Int? = null
)

/**
 * Owns the server-driven currency catalog and the cached daily exchange rates.
 *
 * The catalog is never hardcoded: it is fetched from GET /currencies and stored in
 * Room. The single fallback below only applies before the first successful fetch,
 * so the UI can still render USD amounts while offline.
 */
@Singleton
class CurrencyRepository @Inject constructor(
    private val currencyApi: CurrencyApi,
    private val preferencesApi: PreferencesApi,
    private val adminApi: AdminApi,
    private val currencyDao: CurrencyDao,
    private val exchangeRateDao: ExchangeRateDao,
    private val entryDao: EntryDao,
    private val preferencesRepository: PreferencesRepository,
    private val database: SunderDatabase
) {
    companion object {
        private const val TAG = "CurrencyRepository"
        private const val MAX_HISTORY_DAYS = 365L

        val FALLBACK_CURRENCY = CurrencyEntity(
            code = CANONICAL_BASE_CURRENCY,
            name = "US Dollar",
            symbol = "$",
            symbolAfter = false,
            decimalDigits = 2,
            isActive = true
        )
    }

    val activeCurrencies: Flow<List<CurrencyEntity>> = currencyDao.observeActiveCurrencies()
        .map { currencies -> currencies.ifEmpty { listOf(FALLBACK_CURRENCY) } }

    val allCurrencies: Flow<List<CurrencyEntity>> = currencyDao.observeAllCurrencies()
        .map { currencies -> currencies.ifEmpty { listOf(FALLBACK_CURRENCY) } }

    val exchangeRates: Flow<List<ExchangeRateEntity>> = exchangeRateDao.observeAll()

    val ratesByDate: Flow<Map<String, Map<String, Double>>> = exchangeRateDao.observeAll()
        .map { CurrencyConverter.ratesByDate(it) }

    val defaultCurrencyCode: Flow<String> = preferencesRepository.defaultCurrencyCode

    /** Combined catalog, default currency, and rate cache consumed by every screen. */
    val uiState: Flow<CurrencyUiState> = combine(
        activeCurrencies,
        allCurrencies,
        defaultCurrencyCode,
        ratesByDate
    ) { active, all, defaultCode, rates ->
        CurrencyUiState(
            currencies = active,
            defaultCurrencyCode = defaultCode,
            ratesByDate = rates,
            allCurrencies = all
        )
    }

    suspend fun getCurrency(code: String?): CurrencyEntity? =
        currencyDao.getCurrency(normalizeCurrencyCode(code))

    suspend fun getCurrencyOrDefault(code: String?): CurrencyEntity {
        val normalized = normalizeCurrencyCode(code)
        return getCurrency(normalized)
            ?: if (normalized == CANONICAL_BASE_CURRENCY) {
                FALLBACK_CURRENCY
            } else {
                CurrencyUiState.fallback(normalized)
            }
    }

    /**
     * Fetches the server-allowed active currencies and merges them into the local cache
     * while keeping previously cached currencies marked inactive so historical entries
     * retain their symbol, name, and decimal digits.
     */
    suspend fun refreshCurrencies(): List<CurrencyEntity> {
        return try {
            val response = currencyApi.getCurrencies()
            val body = response.body()
            if (!response.isSuccessful || body == null) {
                Log.w(TAG, "Unable to refresh currencies (${response.code()})")
                return cachedActiveCurrencies()
            }

            val currencies = body.currencies.map { it.toEntity() }
            if (currencies.isNotEmpty()) {
                database.withTransaction {
                    currencyDao.deactivateAll()
                    currencyDao.upsertAll(currencies)
                }
                syncLegacyCurrencyMetadata(currencies)
            }
            currencies
        } catch (e: Exception) {
            Log.w(TAG, "Currency refresh failed: ${e.message}")
            cachedActiveCurrencies()
        }
    }

    /**
     * Refreshes the latest snapshot and backfills history in chunks the server
     * accepts (max 365 days per request). Routine syncs (when [fromDate] and [toDate]
     * are null and [force] is false) only fetch date ranges not already present in the
     * local cache.
     */
    suspend fun refreshRates(
        fromDate: String? = null,
        toDate: String? = null,
        force: Boolean = false
    ) {
        val today = LocalDate.now(ZoneOffset.UTC)
        val end = parseDate(toDate) ?: today

        refreshLatestRates()

        val explicitRange = fromDate != null || toDate != null
        val start = parseDate(fromDate) ?: earliestEntryDate() ?: return
        val clampedStart = if (start > end) end else start

        if (explicitRange || force) {
            fetchHistoryRange(clampedStart, end)
            return
        }

        val cachedEarliest = parseDate(exchangeRateDao.getEarliestRateDate())
        val cachedLatest = parseDate(exchangeRateDao.getLatestRateDate())

        if (cachedEarliest == null || cachedLatest == null) {
            fetchHistoryRange(clampedStart, end)
            return
        }

        if (clampedStart.isBefore(cachedEarliest)) {
            fetchHistoryRange(clampedStart, cachedEarliest.minusDays(1))
        }
        if (cachedLatest.isBefore(end)) {
            fetchHistoryRange(cachedLatest, end)
        }
    }

    suspend fun refreshAll(force: Boolean = false) {
        refreshCurrencies()
        refreshRates(force = force)
    }

    /**
     * Applies the default currency locally right away, then pushes it to the
     * server on a best-effort basis so offline selections survive until the next sync.
     */
    suspend fun setDefaultCurrency(code: String) {
        val normalized = normalizeCurrencyCode(code)
        preferencesRepository.setDefaultCurrencyCode(normalized)
        preferencesRepository.setPendingDefaultCurrencySync(true)
        getCurrency(normalized)?.let {
            preferencesRepository.setCurrencyMetadata(it.symbol, it.symbolAfter)
        }

        pushDefaultCurrencyToServer(normalized)
    }

    /** Pushes a pending offline default currency choice to the server. Returns true if synced. */
    suspend fun pushPendingDefaultCurrency(): Boolean {
        if (!preferencesRepository.hasPendingDefaultCurrencySync()) return false
        val localCode = preferencesRepository.defaultCurrencyCode.first()
        return pushDefaultCurrencyToServer(localCode)
    }

    private suspend fun pushDefaultCurrencyToServer(code: String): Boolean {
        return try {
            val response = preferencesApi.updatePreferences(
                UpdatePreferencesRequest(defaultCurrencyCode = code)
            )
            val body = response.body()
            if (response.isSuccessful && body != null) {
                preferencesRepository.setDefaultCurrencyCode(body.defaultCurrencyCode)
                body.defaultCurrency?.let {
                    preferencesRepository.setCurrencyMetadata(it.symbol, it.symbolAfter)
                }
                preferencesRepository.setPendingDefaultCurrencySync(false)
                true
            } else {
                false
            }
        } catch (e: Exception) {
            Log.w(TAG, "Unable to persist default currency: ${e.message}")
            false
        }
    }

    /** Pulls the authenticated user's preferences from the server, if reachable. */
    suspend fun refreshPreferences() {
        if (preferencesRepository.hasPendingDefaultCurrencySync()) {
            if (!pushPendingDefaultCurrency()) return
        }
        try {
            val body = preferencesApi.getPreferences().body() ?: return
            preferencesRepository.setDefaultCurrencyCode(body.defaultCurrencyCode)
            body.defaultCurrency?.let {
                preferencesRepository.setCurrencyMetadata(it.symbol, it.symbolAfter)
            }
        } catch (e: Exception) {
            Log.w(TAG, "Unable to refresh currency preferences: ${e.message}")
        }
    }

    suspend fun getAdminCurrencies(): Result<List<CurrencyEntity>> {
        return try {
            val response = adminApi.getCurrencies()
            val body = response.body()
            if (!response.isSuccessful || body == null) {
                Result.failure(IllegalStateException("Failed to load currencies (${response.code()})"))
            } else {
                Result.success(body.currencies.map { it.toEntity() })
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun createCurrency(currency: CurrencyEntity): Result<CurrencyEntity> {
        return try {
            val response = adminApi.createCurrency(
                CreateCurrencyRequest(
                    code = normalizeCurrencyCode(currency.code),
                    name = currency.name,
                    symbol = currency.symbol,
                    symbolAfter = currency.symbolAfter,
                    decimalDigits = currency.decimalDigits,
                    isActive = currency.isActive
                )
            )
            val body = response.body()
            if (!response.isSuccessful || body?.currency == null) {
                Result.failure(IllegalStateException("Failed to create currency (${response.code()})"))
            } else {
                refreshCurrencies()
                Result.success(body.currency.toEntity())
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun updateCurrency(
        code: String,
        name: String? = null,
        symbol: String? = null,
        symbolAfter: Boolean? = null,
        decimalDigits: Int? = null,
        isActive: Boolean? = null
    ): Result<CurrencyEntity> {
        return try {
            val response = adminApi.updateCurrency(
                code = normalizeCurrencyCode(code),
                request = UpdateCurrencyRequest(
                    name = name,
                    symbol = symbol,
                    symbolAfter = symbolAfter,
                    decimalDigits = decimalDigits,
                    isActive = isActive
                )
            )
            val body = response.body()
            if (!response.isSuccessful || body?.currency == null) {
                Result.failure(IllegalStateException("Failed to update currency (${response.code()})"))
            } else {
                refreshCurrencies()
                Result.success(body.currency.toEntity())
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun syncRates(
        date: String? = null,
        days: Int? = null,
        force: Boolean? = null
    ): Result<RateSyncSummary> {
        return try {
            val response = adminApi.syncRates(date = date, days = days, force = force)
            val body = response.body()
            if (!response.isSuccessful || body == null) {
                Result.failure(IllegalStateException("Failed to sync rates (${response.code()})"))
            } else {
                val from = body.from ?: body.date ?: date
                val to = body.to ?: body.date ?: date
                refreshRates(fromDate = from, toDate = to, force = true)
                Result.success(
                    RateSyncSummary(
                        success = body.success,
                        date = body.date,
                        from = body.from,
                        to = body.to,
                        source = body.source,
                        storedCount = body.storedCount
                    )
                )
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    suspend fun upsertManualRate(
        baseCurrencyCode: String? = null,
        targetCurrencyCode: String,
        rate: Double,
        rateDate: String? = null
    ): Result<Unit> {
        return try {
            val normalizedTarget = normalizeCurrencyCode(targetCurrencyCode)
            val response = adminApi.upsertRate(
                UpsertRateRequest(
                    baseCurrencyCode = baseCurrencyCode,
                    targetCurrencyCode = normalizedTarget,
                    rate = rate,
                    rateDate = rateDate
                )
            )
            if (!response.isSuccessful) {
                Result.failure(IllegalStateException("Failed to store rate (${response.code()})"))
            } else {
                val saved = response.body()?.rate
                val targetDate = saved?.rateDate?.takeIf { it.isNotBlank() }
                    ?: rateDate?.takeIf { it.isNotBlank() }
                    ?: LocalDate.now(ZoneOffset.UTC).toString()
                val savedRate = saved?.rate ?: rate
                val savedBase = normalizeCurrencyCode(saved?.baseCurrencyCode ?: baseCurrencyCode)
                if (savedBase == CANONICAL_BASE_CURRENCY && savedRate.isFinite() && savedRate > 0.0) {
                    storeSnapshot(targetDate, mapOf(normalizedTarget to savedRate))
                }
                refreshRates(fromDate = targetDate, toDate = targetDate, force = true)
                Result.success(Unit)
            }
        } catch (e: Exception) {
            Result.failure(e)
        }
    }

    private suspend fun cachedActiveCurrencies(): List<CurrencyEntity> =
        currencyDao.getActiveCurrencies().ifEmpty { listOf(FALLBACK_CURRENCY) }

    private suspend fun syncLegacyCurrencyMetadata(currencies: List<CurrencyEntity>) {
        val defaultCode = preferencesRepository.defaultCurrencyCode.first()
        val selected = currencies.firstOrNull { it.code == defaultCode }
            ?: currencies.firstOrNull { it.code == CANONICAL_BASE_CURRENCY }
            ?: return
        preferencesRepository.setCurrencyMetadata(selected.symbol, selected.symbolAfter)
    }

    private suspend fun fetchHistoryRange(start: LocalDate, end: LocalDate) {
        var cursor = start
        while (!cursor.isAfter(end)) {
            val chunkEnd = minOf(cursor.plusDays(MAX_HISTORY_DAYS - 1), end)
            val fetched = fetchHistory(cursor, chunkEnd)
            if (!fetched) break
            cursor = chunkEnd.plusDays(1)
        }
    }

    private suspend fun refreshLatestRates() {
        try {
            val response = currencyApi.getRates()
            val body = response.body()
            if (response.isSuccessful && body != null) {
                storeSnapshot(body.date, body.rates)
            } else {
                Log.w(TAG, "Unable to refresh latest rates (${response.code()})")
            }
        } catch (e: Exception) {
            Log.w(TAG, "Latest rate refresh failed: ${e.message}")
        }
    }

    private suspend fun fetchHistory(from: LocalDate, to: LocalDate): Boolean {
        return try {
            val response = currencyApi.getRatesHistory(
                from = from.toString(),
                to = to.toString()
            )
            val body = response.body()
            if (response.isSuccessful && body != null) {
                val allEntities = buildList {
                    body.rates.forEach { (date, rates) ->
                        addAll(buildSnapshotEntities(date, rates))
                    }
                }
                if (allEntities.isNotEmpty()) {
                    exchangeRateDao.upsertAll(allEntities)
                }
                true
            } else {
                Log.w(TAG, "Unable to refresh rate history (${response.code()})")
                false
            }
        } catch (e: Exception) {
            Log.w(TAG, "Rate history refresh failed: ${e.message}")
            false
        }
    }

    private suspend fun storeSnapshot(date: String, rates: Map<String, Double>) {
        val entities = buildSnapshotEntities(date, rates)
        if (entities.isNotEmpty()) {
            exchangeRateDao.upsertAll(entities)
        }
    }

    private fun buildSnapshotEntities(
        date: String,
        rates: Map<String, Double>
    ): List<ExchangeRateEntity> {
        if (date.isBlank() || rates.isEmpty()) return emptyList()
        val dateKey = CurrencyConverter.normalizeDate(date)
        val entities = rates
            .mapNotNull { (code, rate) ->
                if (!rate.isFinite() || rate <= 0.0) return@mapNotNull null
                ExchangeRateEntity(
                    rateDate = dateKey,
                    currencyCode = normalizeCurrencyCode(code),
                    rate = rate
                )
            }
            .toMutableList()

        if (entities.none { it.currencyCode == CANONICAL_BASE_CURRENCY }) {
            entities.add(
                ExchangeRateEntity(
                    rateDate = dateKey,
                    currencyCode = CANONICAL_BASE_CURRENCY,
                    rate = 1.0
                )
            )
        }
        return entities
    }

    private suspend fun earliestEntryDate(): LocalDate? =
        entryDao.getEarliestActiveEntryDate()?.let { millis ->
            Instant.ofEpochMilli(millis).atZone(ZoneOffset.UTC).toLocalDate()
        }

    private fun parseDate(date: String?): LocalDate? {
        val value = date?.trim().orEmpty()
        if (value.length < 10) return null
        return try {
            LocalDate.parse(value.substring(0, 10))
        } catch (_: Exception) {
            null
        }
    }
}

private fun CurrencyResponseDto.toEntity() = CurrencyEntity(
    code = normalizeCurrencyCode(code),
    name = name,
    symbol = symbol,
    symbolAfter = symbolAfter,
    decimalDigits = decimalDigits,
    isActive = true
)

private fun AdminCurrencyDto.toEntity() = CurrencyEntity(
    code = normalizeCurrencyCode(code),
    name = name,
    symbol = symbol,
    symbolAfter = symbolAfter,
    decimalDigits = decimalDigits,
    isActive = isActive
)
