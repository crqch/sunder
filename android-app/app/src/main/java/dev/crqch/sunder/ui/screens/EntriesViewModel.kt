package dev.crqch.sunder.ui.screens

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.navigation.toRoute
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.crqch.sunder.data.local.AccountWithBalance
import dev.crqch.sunder.data.local.EntryEntity
import dev.crqch.sunder.data.local.EntryWithDetails
import dev.crqch.sunder.data.repositories.AccountRepository
import dev.crqch.sunder.data.repositories.CategoryRepository
import dev.crqch.sunder.data.repositories.EntryRepository
import dev.crqch.sunder.ui.SubRoute
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import java.time.Instant
import javax.inject.Inject

data class EntryFormState(
    val title: String = "",
    val date: Long = Instant.now().toEpochMilli(),
    val location: String = "",
    val amount: Float = 0.00f,
    val accountId: String = "",
    val categoryId: String = "",
    val description: String = "",
) {
    fun isFilled(): Boolean {
        if (title.isBlank()) return false
        if (accountId.isBlank() || categoryId.isBlank()) return false
        return amount > 0
    }
}

enum class SelectionType { NONE, ACCOUNT, CATEGORY }

@HiltViewModel
class EntriesViewModel @Inject constructor(
    savedStateHandle: SavedStateHandle,
    private val entryRepository: EntryRepository,
    private val accountRepository: AccountRepository,
    private val categoryRepository: CategoryRepository,
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

    var amountInput by mutableStateOf(entryFormState.amount.let { if (it == 0f) "" else it.toString() })
        private set

    fun updateForm(state: EntryFormState) {
        entryFormState = state
    }

    fun updateAmount(input: String) {
        if (input.isEmpty() || input.matches(Regex("^\\d*\\.?\\d{0,2}$"))) {
            amountInput = input
            val safeFloat = input.toFloatOrNull() ?: 0f
            entryFormState = entryFormState.copy(amount = safeFloat)
        }
    }

    private val _selectedAccountId = MutableStateFlow<String?>(null)
    val selectedAccountId: StateFlow<String?> = _selectedAccountId.asStateFlow()

    private val _selectedCategoryId = MutableStateFlow<String?>(null)

    val selectedCategoryId: StateFlow<String?> = _selectedCategoryId.asStateFlow()

    private val _query = MutableStateFlow<String?>(null)

    val query: StateFlow<String?> = _query.asStateFlow()

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

    @OptIn(ExperimentalCoroutinesApi::class)
    val entries: StateFlow<List<EntryWithDetails>> =
        combine(_selectedAccountId, _selectedCategoryId, _query) { accountId, categoryId, query ->
            Triple(accountId, categoryId, query)
        }.flatMapLatest { (accountId, categoryId, query) ->
            entryRepository.getEntries(accountId, categoryId, query)
        }
            .stateIn(
                scope = viewModelScope,
                started = SharingStarted.WhileSubscribed(5000),
                initialValue = emptyList()
            )

    fun createEntry(entryFormState: EntryFormState, onComplete: () -> Unit) {
        viewModelScope.launch {
            entryRepository.createEntry(
                entryFormState.title,
                entryFormState.description,
                entryFormState.amount,
                entryFormState.accountId,
                entryFormState.categoryId,
                entryFormState.date
            )
            onComplete()
        }
    }

    fun selectAccount(accountId: String?) {
        _selectedAccountId.value = accountId
    }

    fun selectCategory(categoryId: String?) {
        _selectedCategoryId.value = categoryId
    }

    fun updateQuery(query: String?) {
        _query.value = query
    }
}