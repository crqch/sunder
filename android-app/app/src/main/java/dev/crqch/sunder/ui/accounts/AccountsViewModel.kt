package dev.crqch.sunder.ui.accounts

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.crqch.sunder.data.local.AccountWithBalance
import dev.crqch.sunder.data.repositories.AccountRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class AccountsViewModel @Inject constructor(
    private val accountRepository: AccountRepository
) : ViewModel() {

    private val _searchQuery = MutableStateFlow("")
    val searchQuery: StateFlow<String> = _searchQuery.asStateFlow()

    val allAccounts: StateFlow<List<AccountWithBalance>> =
        accountRepository.accountsWithBalance.stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = emptyList()
        )

    val accounts: StateFlow<List<AccountWithBalance>> =
        combine(allAccounts, _searchQuery) { list, query ->
            if (query.isBlank()) list
            else list.filter { it.account.name.contains(query.trim(), ignoreCase = true) }
        }.stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = emptyList()
        )

    fun updateSearchQuery(query: String) {
        _searchQuery.value = query
    }

    fun createAccount(name: String, onComplete: () -> Unit) {
        viewModelScope.launch {
            accountRepository.createAccount(name.trim())
            onComplete()
        }
    }

    fun deleteAccounts(ids: List<String>, onComplete: () -> Unit = {}) {
        viewModelScope.launch {
            accountRepository.deleteAccounts(ids)
            onComplete()
        }
    }
}