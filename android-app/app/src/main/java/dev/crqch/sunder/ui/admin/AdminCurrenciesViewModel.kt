package dev.crqch.sunder.ui.admin

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.crqch.sunder.data.local.CurrencyEntity
import dev.crqch.sunder.data.repositories.CurrencyRepository
import dev.crqch.sunder.data.repositories.RateSyncSummary
import dev.crqch.sunder.utils.normalizeCurrencyCode
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.time.LocalDate
import javax.inject.Inject

enum class AdminCurrencyFilter { ALL, ACTIVE, INACTIVE }

enum class AdminEditorError { CODE, NAME, DIGITS }

data class AdminCurrencyEditor(
    val editingCode: String? = null,
    val code: String = "",
    val name: String = "",
    val symbol: String = "",
    val symbolAfter: Boolean = false,
    val decimalDigits: String = "2",
    val isActive: Boolean = true,
    val error: AdminEditorError? = null
)

sealed interface AdminStatus {
    data class RatesSynced(val summary: RateSyncSummary) : AdminStatus
    data class RateStored(val code: String) : AdminStatus
    data class CurrencyStored(val code: String) : AdminStatus
    data class CurrencyToggled(val code: String, val active: Boolean) : AdminStatus
    data object InvalidDate : AdminStatus
    data object InvalidRate : AdminStatus
    data object InvalidDays : AdminStatus
    data object InvalidCode : AdminStatus
    data class Failed(val detail: String? = null) : AdminStatus
}

/**
 * Drives the admin currencies & exchange-rates screen. All mutations go through
 * [CurrencyRepository], which keeps the local cache and rate cache in sync.
 */
@HiltViewModel
class AdminCurrenciesViewModel @Inject constructor(
    private val currencyRepository: CurrencyRepository
) : ViewModel() {

    private val _currencies = MutableStateFlow<List<CurrencyEntity>>(emptyList())
    val currencies: StateFlow<List<CurrencyEntity>> = _currencies.asStateFlow()

    private val _isLoading = MutableStateFlow(false)
    val isLoading: StateFlow<Boolean> = _isLoading.asStateFlow()

    private val _isSyncingRates = MutableStateFlow(false)
    val isSyncingRates: StateFlow<Boolean> = _isSyncingRates.asStateFlow()

    private val _isSavingCurrency = MutableStateFlow(false)
    val isSavingCurrency: StateFlow<Boolean> = _isSavingCurrency.asStateFlow()

    private val _status = MutableStateFlow<AdminStatus?>(null)
    val status: StateFlow<AdminStatus?> = _status.asStateFlow()

    var editor by mutableStateOf<AdminCurrencyEditor?>(null)
        private set

    init {
        load()
    }

    fun load() {
        viewModelScope.launch {
            _isLoading.value = true
            withContext(Dispatchers.IO) {
                currencyRepository.getAdminCurrencies()
            }.onSuccess { list ->
                _currencies.value = list.sortedBy { it.code }
            }.onFailure { error ->
                _status.value = AdminStatus.Failed(error.message)
            }
            _isLoading.value = false
        }
    }

    fun syncLatestRates() {
        syncRates()
    }

    fun syncDate(date: String) {
        val trimmed = date.trim()
        if (!isValidDate(trimmed)) {
            _status.value = AdminStatus.InvalidDate
            return
        }
        syncRates(date = trimmed)
    }

    fun backfill(daysInput: String) {
        val days = daysInput.trim().toIntOrNull()
        if (days == null || days <= 0) {
            _status.value = AdminStatus.InvalidDays
            return
        }
        syncRates(days = days)
    }

    private fun syncRates(date: String? = null, days: Int? = null) {
        if (_isSyncingRates.value) return
        viewModelScope.launch {
            _isSyncingRates.value = true
            withContext(Dispatchers.IO) {
                currencyRepository.syncRates(date = date, days = days)
            }.onSuccess { summary ->
                _status.value = AdminStatus.RatesSynced(summary)
            }.onFailure { error ->
                _status.value = AdminStatus.Failed(error.message)
            }
            _isSyncingRates.value = false
        }
    }

    fun saveManualRate(targetCode: String, rateInput: String, date: String) {
        val code = normalizeCurrencyCode(targetCode)
        if (code.length != 3 || code.any { !it.isLetter() }) {
            _status.value = AdminStatus.InvalidCode
            return
        }
        val rate = rateInput.trim().replace(',', '.').toDoubleOrNull()
        if (rate == null || !rate.isFinite() || rate <= 0.0) {
            _status.value = AdminStatus.InvalidRate
            return
        }
        val rateDate = date.trim().ifBlank { null }
        if (rateDate != null && !isValidDate(rateDate)) {
            _status.value = AdminStatus.InvalidDate
            return
        }

        viewModelScope.launch {
            _isSyncingRates.value = true
            withContext(Dispatchers.IO) {
                currencyRepository.upsertManualRate(
                    targetCurrencyCode = code,
                    rate = rate,
                    rateDate = rateDate
                )
            }.onSuccess {
                _status.value = AdminStatus.RateStored(code)
            }.onFailure { error ->
                _status.value = AdminStatus.Failed(error.message)
            }
            _isSyncingRates.value = false
        }
    }

    fun setCurrencyActive(currency: CurrencyEntity, active: Boolean) {
        if (_isSavingCurrency.value) return
        viewModelScope.launch {
            _isSavingCurrency.value = true
            withContext(Dispatchers.IO) {
                currencyRepository.updateCurrency(code = currency.code, isActive = active)
            }.onSuccess { updated ->
                _currencies.update { list ->
                    list.map { if (it.code == updated.code) updated else it }
                }
                _status.value = AdminStatus.CurrencyToggled(updated.code, updated.isActive)
            }.onFailure { error ->
                _status.value = AdminStatus.Failed(error.message)
            }
            _isSavingCurrency.value = false
        }
    }

    fun openCreateCurrency() {
        editor = AdminCurrencyEditor()
    }

    fun openEditCurrency(currency: CurrencyEntity) {
        editor = AdminCurrencyEditor(
            editingCode = currency.code,
            code = currency.code,
            name = currency.name,
            symbol = currency.symbol,
            symbolAfter = currency.symbolAfter,
            decimalDigits = currency.decimalDigits.toString(),
            isActive = currency.isActive
        )
    }

    fun closeEditor() {
        editor = null
    }

    fun updateEditor(transform: (AdminCurrencyEditor) -> AdminCurrencyEditor) {
        editor = editor?.let(transform)
    }

    fun saveEditor() {
        val current = editor ?: return
        val rawCode = current.code.trim().uppercase()
        if (rawCode.length != 3 || rawCode.any { !it.isLetter() }) {
            editor = current.copy(error = AdminEditorError.CODE)
            return
        }
        val code = normalizeCurrencyCode(rawCode)
        if (current.name.isBlank()) {
            editor = current.copy(error = AdminEditorError.NAME)
            return
        }
        val digits = current.decimalDigits.trim().toIntOrNull()
        if (digits == null || digits !in 0..6) {
            editor = current.copy(error = AdminEditorError.DIGITS)
            return
        }

        viewModelScope.launch {
            _isSavingCurrency.value = true
            val result = withContext(Dispatchers.IO) {
                if (current.editingCode == null) {
                    currencyRepository.createCurrency(
                        CurrencyEntity(
                            code = code,
                            name = current.name.trim(),
                            symbol = current.symbol.trim(),
                            symbolAfter = current.symbolAfter,
                            decimalDigits = digits,
                            isActive = current.isActive
                        )
                    )
                } else {
                    currencyRepository.updateCurrency(
                        code = current.editingCode,
                        name = current.name.trim(),
                        symbol = current.symbol.trim(),
                        symbolAfter = current.symbolAfter,
                        decimalDigits = digits,
                        isActive = current.isActive
                    )
                }
            }
            result.onSuccess { saved ->
                editor = null
                _currencies.update { list ->
                    val merged = if (current.editingCode == null) {
                        list + saved
                    } else {
                        list.map { if (it.code == saved.code) saved else it }
                    }
                    merged.distinctBy { it.code }.sortedBy { it.code }
                }
                _status.value = AdminStatus.CurrencyStored(saved.code)
            }.onFailure { error ->
                _status.value = AdminStatus.Failed(error.message)
            }
            _isSavingCurrency.value = false
        }
    }

    fun clearStatus() {
        _status.value = null
    }

    private fun isValidDate(value: String): Boolean {
        if (!value.matches(Regex("^\\d{4}-\\d{2}-\\d{2}$"))) return false
        return try {
            LocalDate.parse(value)
            true
        } catch (_: Exception) {
            false
        }
    }
}
