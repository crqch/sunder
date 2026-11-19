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
import dev.crqch.sunder.data.local.EntryEntity
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

    private var lastLoadedEntity: EntryEntity? = null

    var entryFormState by mutableStateOf(EntryFormState())
        private set

    var amountInput by mutableStateOf("")
        private set

    init {
        entryId?.let { id ->
            viewModelScope.launch {
                entryRepository.getEntry(id).collect { entity ->
                    entity?.let {
                        lastLoadedEntity = it
                        if (!isEditMode) {
                            applyEntity(it)
                        }
                    }
                }
            }
        }
    }

    private fun applyEntity(entity: EntryEntity) {
        entryFormState = EntryFormState.ofEntity(entity)
        amountInput = amountToCentsDigits(entryFormState.amount)
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

    fun updateForm(state: EntryFormState) {
        entryFormState = state
    }

    fun updateAmount(input: String) {
        val parsed = parseCentsAmountInput(input, entryFormState.isExpense)
        amountInput = parsed.digits
        entryFormState = entryFormState.copy(
            amount = parsed.amount,
            isExpense = parsed.isExpense
        )
    }

    fun startEdit() {
        lastLoadedEntity?.let { applyEntity(it) }
        isEditMode = true
    }

    fun cancelEdit() {
        lastLoadedEntity?.let { applyEntity(it) }
        isEditMode = false
    }

    fun saveEdit() {
        if (!entryFormState.isFilled()) return
        viewModelScope.launch {
            entryRepository.saveEntry(entryFormState, entryId)
            isEditMode = false
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
}