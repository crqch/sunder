package dev.crqch.sunder.ui.common

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.crqch.sunder.data.local.CategoryEntity
import dev.crqch.sunder.data.local.EntryWithDetails
import dev.crqch.sunder.data.repositories.CategoryRepository
import dev.crqch.sunder.data.repositories.CurrencyRepository
import dev.crqch.sunder.data.repositories.EntryRepository
import dev.crqch.sunder.data.repositories.SyncRepository
import dev.crqch.sunder.data.repositories.UserRepository
import dev.crqch.sunder.utils.CurrencyUiState
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import javax.inject.Inject
import kotlin.math.abs

data class CategorySpendingItem(
    val category: CategoryEntity,
    val amount: Float,
    val share: Float
)

data class DashboardState(
    val totalBalance: Float = 0f,
    val totalIncome: Float = 0f,
    val totalExpenses: Float = 0f,
    val recentEntries: List<EntryWithDetails> = emptyList(),
    val topCategories: List<CategorySpendingItem> = emptyList(),
    val totalEntryCount: Int = 0
)

@HiltViewModel
class HomeViewModel @Inject constructor(
    entryRepository: EntryRepository,
    categoryRepository: CategoryRepository,
    userRepository: UserRepository,
    currencyRepository: CurrencyRepository,
    private val syncRepository: SyncRepository
) : ViewModel() {

    val currentUser = userRepository.currentUser
    val isSyncing = syncRepository.isSyncing

    val currencyState: StateFlow<CurrencyUiState> = currencyRepository.uiState.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = CurrencyUiState()
    )

    private val _syncMessage = MutableStateFlow<String?>(null)
    val syncMessage: StateFlow<String?> = _syncMessage.asStateFlow()

    private val _unsyncedCount = MutableStateFlow(0)
    val unsyncedCount: StateFlow<Int> = _unsyncedCount.asStateFlow()

    val dashboardState: StateFlow<DashboardState> = combine(
        entryRepository.allEntries,
        categoryRepository.allCategories,
        currencyRepository.uiState,
        syncRepository.lastSyncedAt
    ) { entries, categories, currency, _ ->
        refreshUnsyncedCount()

        val convertedAmounts = entries.associateWith { item ->
            currency.convertOrSame(
                amount = item.entry.amount.toDouble(),
                fromCode = item.entry.currencyCode,
                dateMillis = item.entry.date
            )
        }

        val totalBalance = convertedAmounts.values.sum()
        val totalIncome = convertedAmounts
            .filterKeys { it.entry.amount > 0f }
            .values.sum()
        val totalExpenses = convertedAmounts
            .filterKeys { it.entry.amount < 0f }
            .values.sumOf { abs(it) }

        val spendingMap = mutableMapOf<String, Double>()
        entries.forEach { item ->
            if (item.entry.amount < 0f) {
                val catId = item.entry.categoryId
                val converted = abs(convertedAmounts[item] ?: 0.0)
                spendingMap[catId] = (spendingMap[catId] ?: 0.0) + converted
            }
        }

        val categoryById = categories.associateBy { it.id }
        val topCategories = spendingMap.entries
            .mapNotNull { (catId, amount) ->
                val cat = categoryById[catId] ?: return@mapNotNull null
                val share = if (totalExpenses > 0.0) {
                    (amount / totalExpenses).toFloat().coerceIn(0f, 1f)
                } else {
                    0f
                }
                CategorySpendingItem(
                    category = cat,
                    amount = amount.toFloat(),
                    share = share
                )
            }
            .sortedByDescending { it.amount }
            .take(5)

        DashboardState(
            totalBalance = totalBalance.toFloat(),
            totalIncome = totalIncome.toFloat(),
            totalExpenses = totalExpenses.toFloat(),
            recentEntries = entries.take(8),
            topCategories = topCategories,
            totalEntryCount = entries.size
        )
    }.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = DashboardState()
    )

    private fun refreshUnsyncedCount() {
        viewModelScope.launch {
            _unsyncedCount.value = withContext(Dispatchers.IO) {
                syncRepository.getUnsyncedCount()
            }
        }
    }

    fun triggerManualSync(successMsg: String, failMsg: String) {
        if (isSyncing.value) return
        viewModelScope.launch {
            val result = withContext(Dispatchers.IO) {
                syncRepository.syncNow()
            }
            refreshUnsyncedCount()
            _syncMessage.value = if (result.isSuccess) successMsg else failMsg
        }
    }

    fun clearSyncMessage() {
        _syncMessage.value = null
    }
}
