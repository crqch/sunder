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
import dev.crqch.sunder.data.repositories.AccountRepository
import dev.crqch.sunder.data.repositories.CategoryRepository
import dev.crqch.sunder.data.repositories.EntryRepository
import dev.crqch.sunder.ui.SubRoute
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class EntryDetailViewModel @Inject constructor(
    savedStateHandle: SavedStateHandle,
    private val entryRepository: EntryRepository,
    private val accountRepository: AccountRepository,
    private val categoryRepository: CategoryRepository
) : ViewModel() {

    var isEditMode by mutableStateOf(false)
        private set

    private val entryRoute = try {
        savedStateHandle.toRoute<SubRoute.Entry>()
    } catch (_: Exception) {
        null
    }

    val entryId = entryRoute?.entryId

    var entryFormState by mutableStateOf(EntryFormState())
        private set

    init {
        entryId?.let { id ->
            viewModelScope.launch {
                entryRepository.getEntry(id).collect { entity ->
                    entity?.let {
                        entryFormState = EntryFormState.ofEntity(it)
                        amountInput =
                            if (entryFormState.amount == 0f) "" else entryFormState.amount.toString()
                    }
                }
            }
        }
    }

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

    fun updateEntry(state: EntryFormState, onComplete: () -> Unit) {
        viewModelScope.launch {
            entryRepository.saveEntry(state, entryId)
            onComplete()
        }
    }

    fun delete(onComplete: () -> Unit) {
        entryId?.let { id ->
            viewModelScope.launch {
                entryRepository.deleteEntry(id)
                onComplete()
            }
        }
    }

    fun toggleEditMode() {
        viewModelScope.launch {
            entryRepository.saveEntry(entryFormState, entryId)
        }
        isEditMode = !isEditMode
    }
}