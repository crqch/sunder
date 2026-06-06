package dev.crqch.sunder.ui.accounts

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.crqch.sunder.data.local.AccountWithBalance
import dev.crqch.sunder.data.repositories.AccountRepository
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class AccountsViewModel @Inject constructor(
    private val accountRepository: AccountRepository
) : ViewModel() {
    val accounts: StateFlow<List<AccountWithBalance>> =
        accountRepository.accountsWithBalance.stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = emptyList()
        )

    fun createAccount(name: String, onComplete: () -> Unit) {
        viewModelScope.launch {
            accountRepository.createAccount(name)
            onComplete()
        }
    }
}