package dev.crqch.sunder.ui.accounts

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.navigation.toRoute
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.crqch.sunder.data.local.AccountEntity
import dev.crqch.sunder.data.local.CategoryEntity
import dev.crqch.sunder.data.repositories.AccountRepository
import dev.crqch.sunder.data.repositories.CategoryRepository
import dev.crqch.sunder.data.repositories.EntryRepository
import dev.crqch.sunder.ui.SubRoute
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject


data class AccountFormState(
    val name: String = ""
) {
    companion object {
        fun ofAccount(account: AccountEntity): AccountFormState = AccountFormState(
            account.name
        )

    }

    fun isFilled(): Boolean {
        return name.isNotBlank()
    }

    fun toAccount(id: String, createdAt: Long, updatedAt: Long): AccountEntity = AccountEntity(
        id = id,
        name = name,
        createdAt = createdAt,
        updatedAt = updatedAt
    )
}

@HiltViewModel
class AccountViewModel @Inject constructor(
    savedStateHandle: SavedStateHandle,
    private val categoryRepository: CategoryRepository,
    private val accountRepository: AccountRepository,
    private val entryRepository: EntryRepository
) : ViewModel() {

    var isEditMode by mutableStateOf(false)
        private set

    private val accountRoute = try {
        savedStateHandle.toRoute<SubRoute.Account>()
    } catch (_: Exception) {
        null
    }

    val accountId = accountRoute?.accountId


    fun updateForm(state: AccountFormState) {
        accountFormState = state
    }

    var accountFormState by mutableStateOf(AccountFormState())
        private set

    init {
        accountId?.let { id ->
            viewModelScope.launch {
                accountRepository.getAccount(id).collect { entity ->
                    entity?.let {
                        accountFormState = AccountFormState.ofAccount(it)
                    }
                }
            }
        }
    }


    @OptIn(ExperimentalCoroutinesApi::class)
    val account: StateFlow<AccountEntity?> = flowOf(accountId).flatMapLatest { id ->
        if (id != null) accountRepository.getAccount(id) else flowOf(null)
    }.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = null
    )

    @OptIn(ExperimentalCoroutinesApi::class)
    val entries = flowOf(accountId).flatMapLatest { id ->
        entryRepository.getEntries(id)
    }.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = listOf()
    )

    fun delete(onComplete: () -> Unit) {
        accountId?.let { id ->
            viewModelScope.launch {
                accountRepository.deleteAccount(id)
                onComplete()
            }
        }
    }

    fun toggleEditMode() {
        viewModelScope.launch {
            accountRepository.saveAccount(accountFormState, accountId)
        }
        isEditMode = !isEditMode
    }
}