package dev.crqch.sunder.ui.accounts

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.crqch.sunder.data.local.AccountWithBalance
import dev.crqch.sunder.data.repositories.AccountRepository
import dev.crqch.sunder.data.repositories.CurrencyRepository
import dev.crqch.sunder.data.repositories.EntryRepository
import dev.crqch.sunder.utils.CurrencyUiState
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class AccountsViewModel @Inject constructor(
    private val accountRepository: AccountRepository,
    private val entryRepository: EntryRepository,
    private val currencyRepository: CurrencyRepository
) : ViewModel() {

    private val _searchQuery = MutableStateFlow("")
    val searchQuery: StateFlow<String> = _searchQuery.asStateFlow()

    val currencyState: StateFlow<CurrencyUiState> = currencyRepository.uiState.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = CurrencyUiState()
    )

    /** Per-account balance, converting every entry on its own date. */
    val convertedBalances: StateFlow<Map<String, Double>> = combine(
        entryRepository.allEntries,
        currencyRepository.uiState
    ) { entries, currency ->
        entries.groupBy { it.entry.accountId }.mapValues { (_, accountEntries) ->
            accountEntries.sumOf { item ->
                currency.convertOrSame(
                    item.entry.amount.toDouble(),
                    item.entry.currencyCode,
                    item.entry.date
                )
            }
        }
    }.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = emptyMap()
    )

    val totalBalance: StateFlow<Double> = convertedBalances
        .map { balances -> balances.values.sum() }
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = 0.0
        )

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

    fun createAccount(name: String, currencyCode: String? = null, onComplete: () -> Unit) {
        viewModelScope.launch {
            val created = runCatching {
                accountRepository.createAccount(name.trim(), currencyCode)
            }
            if (created.isSuccess) {
                onComplete()
            }
        }
    }

    fun deleteAccounts(ids: List<String>, onComplete: () -> Unit = {}) {
        viewModelScope.launch {
            accountRepository.deleteAccounts(ids)
            onComplete()
        }
    }
}