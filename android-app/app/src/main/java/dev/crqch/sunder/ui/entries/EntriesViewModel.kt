package dev.crqch.sunder.ui.entries

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.navigation.toRoute
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.crqch.sunder.data.local.AccountWithBalance
import dev.crqch.sunder.data.local.EntryWithDetails
import dev.crqch.sunder.data.repositories.AccountRepository
import dev.crqch.sunder.data.repositories.CategoryRepository
import dev.crqch.sunder.data.repositories.CurrencyRepository
import dev.crqch.sunder.data.repositories.EntryRepository
import dev.crqch.sunder.ui.SubRoute
import dev.crqch.sunder.utils.CurrencyUiState
import dev.crqch.sunder.utils.normalizeCurrencyCode
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import javax.inject.Inject
import kotlin.math.abs

enum class EntrySortField { DATE, AMOUNT, TITLE }
enum class EntryTypeFilter { ALL, EXPENSE, INCOME }
enum class DateRangePreset { ALL, LAST_1M, LAST_3M, LAST_6M, LAST_1Y }

data class EntriesFilterState(
    val query: String = "",
    val selectedAccountIds: Set<String> = emptySet(),
    val selectedCategoryIds: Set<String> = emptySet(),
    val selectedCurrencyCodes: Set<String> = emptySet(),
    val typeFilter: EntryTypeFilter = EntryTypeFilter.ALL,
    val datePreset: DateRangePreset = DateRangePreset.ALL,
    val minAmountInput: String = "",
    val maxAmountInput: String = "",
    val sortField: EntrySortField = EntrySortField.DATE,
    val sortAscending: Boolean = false
) {
    val minAmount: Float? get() = minAmountInput.toFloatOrNull()
    val maxAmount: Float? get() = maxAmountInput.toFloatOrNull()

    val activeFilterCount: Int
        get() {
            var count = 0
            if (selectedAccountIds.isNotEmpty()) count++
            if (selectedCategoryIds.isNotEmpty()) count++
            if (selectedCurrencyCodes.isNotEmpty()) count++
            if (typeFilter != EntryTypeFilter.ALL) count++
            if (datePreset != DateRangePreset.ALL) count++
            if (minAmount != null || maxAmount != null) count++
            return count
        }

    val hasAnyFilter: Boolean
        get() = query.isNotBlank() || activeFilterCount > 0
}

@HiltViewModel
class EntriesViewModel @Inject constructor(
    savedStateHandle: SavedStateHandle,
    private val entryRepository: EntryRepository,
    private val accountRepository: AccountRepository,
    private val categoryRepository: CategoryRepository,
    private val currencyRepository: CurrencyRepository,
) : ViewModel() {

    private val entryRoute = try {
        savedStateHandle.toRoute<SubRoute.CreateEntry>()
    } catch (_: Exception) {
        null
    }

    var entryFormState by mutableStateOf(
        EntryFormState(
            accountId = entryRoute?.accountId ?: "",
            categoryId = entryRoute?.categoryId ?: ""
        )
    )
        private set

    var amountInput by mutableStateOf(
        amountToCentsDigits(entryFormState.amount)
    )
        private set

    /** True once the user explicitly picked a currency in the entry editor. */
    var currencyTouched by mutableStateOf(false)
        private set

    val currencyState: StateFlow<CurrencyUiState> = currencyRepository.uiState.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = CurrencyUiState()
    )

    fun updateForm(state: EntryFormState) {
        entryFormState = state
    }

    fun updateAmount(input: String) {
        val digits = currencyState.value.currency(entryFormState.currencyCode).decimalDigits
        val parsed = parseCurrencyAmountInput(input, entryFormState.isExpense, digits)
        amountInput = parsed.digits
        entryFormState = entryFormState.copy(
            amount = parsed.amount,
            isExpense = parsed.isExpense
        )
    }

    /** Explicit user choice from the currency picker. */
    fun selectCurrency(code: String) {
        val currency = currencyState.value.currency(code)
        currencyTouched = true
        entryFormState = entryFormState.copy(currencyCode = currency.code)
        amountInput = amountToCurrencyDigits(entryFormState.amount, currency.decimalDigits)
    }

    /**
     * Adopts the account's currency (or the default display currency) until the user
     * manually changes it. Switching accounts keeps the adopted currency in sync.
     */
    fun adoptCurrency(accountCurrencyCode: String?) {
        if (currencyTouched) return
        val targetCode = accountCurrencyCode ?: currencyState.value.defaultCurrencyCode
        val currency = currencyState.value.currency(targetCode)
        if (entryFormState.currencyCode == currency.code) return
        entryFormState = entryFormState.copy(currencyCode = currency.code)
        amountInput = amountToCurrencyDigits(entryFormState.amount, currency.decimalDigits)
    }

    private val _filterState = MutableStateFlow(EntriesFilterState())
    val filterState: StateFlow<EntriesFilterState> = _filterState.asStateFlow()

    val availableAccounts: StateFlow<List<AccountWithBalance>> =
        accountRepository.accountsWithBalance.stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = emptyList()
        )

    val categories = categoryRepository.allCategories.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = emptyList()
    )

    val allEntries: StateFlow<List<EntryWithDetails>> = entryRepository.allEntries.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = emptyList()
    )

    val entries: StateFlow<List<EntryWithDetails>> =
        combine(allEntries, _filterState) { list, filter ->
            val q = filter.query.trim()
            val minAmt = filter.minAmount
            val maxAmt = filter.maxAmount
            val cutoffDate = when (filter.datePreset) {
                DateRangePreset.ALL -> null
                DateRangePreset.LAST_1M -> LocalDate.now().minusMonths(1)
                DateRangePreset.LAST_3M -> LocalDate.now().minusMonths(3)
                DateRangePreset.LAST_6M -> LocalDate.now().minusMonths(6)
                DateRangePreset.LAST_1Y -> LocalDate.now().minusYears(1)
            }

            val filtered = list.filter { item ->
                val e = item.entry
                if (q.isNotEmpty()) {
                    val matchesText = e.title.contains(q, ignoreCase = true) ||
                            e.description.contains(q, ignoreCase = true) ||
                            e.location.contains(q, ignoreCase = true) ||
                            item.account.name.contains(q, ignoreCase = true) ||
                            item.category.title.contains(q, ignoreCase = true)
                    if (!matchesText) return@filter false
                }

                if (filter.selectedAccountIds.isNotEmpty() && e.accountId !in filter.selectedAccountIds) {
                    return@filter false
                }

                if (filter.selectedCategoryIds.isNotEmpty() && e.categoryId !in filter.selectedCategoryIds) {
                    return@filter false
                }

                if (filter.selectedCurrencyCodes.isNotEmpty() &&
                    normalizeCurrencyCode(e.currencyCode) !in filter.selectedCurrencyCodes
                ) {
                    return@filter false
                }

                when (filter.typeFilter) {
                    EntryTypeFilter.ALL -> Unit
                    EntryTypeFilter.EXPENSE -> if (e.amount >= 0f) return@filter false
                    EntryTypeFilter.INCOME -> if (e.amount <= 0f) return@filter false
                }

                val absAmt = abs(e.amount)
                if (minAmt != null && absAmt < minAmt) return@filter false
                if (maxAmt != null && absAmt > maxAmt) return@filter false

                if (cutoffDate != null) {
                    val entryDate = Instant.ofEpochMilli(e.date)
                        .atZone(ZoneId.systemDefault())
                        .toLocalDate()
                    if (entryDate.isBefore(cutoffDate)) return@filter false
                }

                true
            }

            val comparator = when (filter.sortField) {
                EntrySortField.DATE -> compareBy<EntryWithDetails> { it.entry.date }
                EntrySortField.AMOUNT -> compareBy<EntryWithDetails> { abs(it.entry.amount) }
                EntrySortField.TITLE -> compareBy<EntryWithDetails> { it.entry.title.lowercase() }
            }

            if (filter.sortAscending) {
                filtered.sortedWith(comparator)
            } else {
                filtered.sortedWith(comparator.reversed())
            }
        }.stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = emptyList()
        )

    fun updateQuery(query: String) {
        _filterState.update { it.copy(query = query) }
    }

    fun toggleAccountFilter(accountId: String) {
        _filterState.update { state ->
            val updated = if (accountId in state.selectedAccountIds) {
                state.selectedAccountIds - accountId
            } else {
                state.selectedAccountIds + accountId
            }
            state.copy(selectedAccountIds = updated)
        }
    }

    fun clearAccountFilters() {
        _filterState.update { it.copy(selectedAccountIds = emptySet()) }
    }

    fun toggleCategoryFilter(categoryId: String) {
        _filterState.update { state ->
            val updated = if (categoryId in state.selectedCategoryIds) {
                state.selectedCategoryIds - categoryId
            } else {
                state.selectedCategoryIds + categoryId
            }
            state.copy(selectedCategoryIds = updated)
        }
    }

    fun clearCategoryFilters() {
        _filterState.update { it.copy(selectedCategoryIds = emptySet()) }
    }

    fun toggleCurrencyFilter(code: String) {
        val normalized = normalizeCurrencyCode(code)
        _filterState.update { state ->
            val updated = if (normalized in state.selectedCurrencyCodes) {
                state.selectedCurrencyCodes - normalized
            } else {
                state.selectedCurrencyCodes + normalized
            }
            state.copy(selectedCurrencyCodes = updated)
        }
    }

    fun clearCurrencyFilters() {
        _filterState.update { it.copy(selectedCurrencyCodes = emptySet()) }
    }

    fun setTypeFilter(type: EntryTypeFilter) {
        _filterState.update { it.copy(typeFilter = type) }
    }

    fun setDatePreset(preset: DateRangePreset) {
        _filterState.update { it.copy(datePreset = preset) }
    }

    fun updateMinAmount(input: String) {
        if (input.isEmpty() || input.matches(Regex("^\\d*\\.?\\d{0,2}$"))) {
            _filterState.update { it.copy(minAmountInput = input) }
        }
    }

    fun updateMaxAmount(input: String) {
        if (input.isEmpty() || input.matches(Regex("^\\d*\\.?\\d{0,2}$"))) {
            _filterState.update { it.copy(maxAmountInput = input) }
        }
    }

    fun setSortField(field: EntrySortField) {
        _filterState.update { it.copy(sortField = field) }
    }

    fun toggleSortDirection() {
        _filterState.update { it.copy(sortAscending = !it.sortAscending) }
    }

    fun clearAllFilters() {
        _filterState.update {
            EntriesFilterState(
                sortField = it.sortField,
                sortAscending = it.sortAscending
            )
        }
    }

    fun createEntry(entryFormState: EntryFormState, onComplete: () -> Unit) {
        viewModelScope.launch {
            val saved = runCatching {
                entryRepository.saveEntry(
                    entryFormState,
                    currencyCode = entryFormState.currencyCode
                )
            }
            if (saved.isSuccess) {
                onComplete()
            }
        }
    }

    fun deleteEntries(ids: List<String>, onComplete: () -> Unit = {}) {
        viewModelScope.launch {
            val deleted = runCatching { entryRepository.deleteEntries(ids) }
            if (deleted.isSuccess) {
                onComplete()
            }
        }
    }

    /** Re-labels entries and refreshes rates best-effort so previews stay accurate. */
    fun updateEntriesCurrency(
        ids: List<String>,
        currencyCode: String,
        onComplete: (count: Int) -> Unit = {},
        onError: (String) -> Unit = {}
    ) {
        if (ids.isEmpty()) return
        viewModelScope.launch {
            val result = runCatching {
                entryRepository.updateEntriesCurrency(ids, currencyCode)
            }
            result.onSuccess { updatedCount ->
                onComplete(updatedCount)
                withContext(Dispatchers.IO) {
                    runCatching { currencyRepository.refreshRates() }
                }
            }.onFailure { error ->
                onError(error.message ?: "Failed to update currency")
            }
        }
    }
}
