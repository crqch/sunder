package dev.crqch.sunder.ui.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import dev.crqch.sunder.R

@Composable
fun EntriesScreen(
    onAddEntry: (accountId: String?, categoryId: String?) -> Unit,
    onAddAccount: () -> Unit,
    onAddCategory: () -> Unit,
    viewModel: EntriesViewModel = hiltViewModel()
) {
    val accounts by viewModel.availableAccounts.collectAsState()
    val selectedAccountId by viewModel.selectedAccountId.collectAsState()
    val entries by viewModel.entries.collectAsState()
    val categories by viewModel.categories.collectAsState()
    val selectedCategoryId by viewModel.selectedCategoryId.collectAsState()

    Scaffold(
        floatingActionButton = {
            FloatingActionButton(onClick = { onAddEntry(selectedAccountId, selectedCategoryId) }) {
                Icon(Icons.Default.Add, contentDescription = stringResource(R.string.add_account))
            }
        }
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .padding(horizontal = 4.dp)
        ) {
            Text(
                stringResource(R.string.account),
                style = MaterialTheme.typography.labelSmall,
                fontWeight = FontWeight.Bold
            )
            LazyRow(
                modifier = Modifier.fillMaxWidth(),
                contentPadding = PaddingValues(horizontal = 16.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                item {
                    FilterChip(
                        selected = selectedAccountId == null,
                        onClick = { viewModel.selectAccount(null) },
                        label = { Text(stringResource(R.string.any_filter)) }
                    )
                }

                items(accounts) { accountWithBalance ->
                    FilterChip(
                        selected = selectedAccountId == accountWithBalance.account.id,
                        onClick = { viewModel.selectAccount(accountWithBalance.account.id) },
                        label = { Text(accountWithBalance.account.name) }
                    )
                }


                item {
                    FilterChip(
                        selected = false,
                        onClick = { onAddAccount() },
                        label = { Text(stringResource(R.string.add_account)) }
                    )
                }
            }
            Text(
                stringResource(R.string.category),
                style = MaterialTheme.typography.labelSmall,
                fontWeight = FontWeight.Bold
            )
            LazyRow(
                modifier = Modifier.fillMaxWidth(),
                contentPadding = PaddingValues(horizontal = 16.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                item {
                    FilterChip(
                        selected = selectedCategoryId == null,
                        onClick = { viewModel.selectCategory(null) },
                        label = { Text(stringResource(R.string.any_filter)) }
                    )
                }

                items(categories) { category ->
                    FilterChip(
                        selected = selectedCategoryId == category.id,
                        onClick = { viewModel.selectCategory(category.id) },
                        label = { Text(category.title) }
                    )
                }

                item {
                    FilterChip(
                        selected = false,
                        onClick = { onAddCategory() },
                        label = { Text(stringResource(R.string.add_category)) }
                    )
                }
            }

            LazyColumn(modifier = Modifier.weight(1f)) {
                items(entries) { entry ->
                    Text("${entry.title}: ${entry.amount}")
                }
            }
        }

    }

}