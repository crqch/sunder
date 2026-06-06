package dev.crqch.sunder.ui.entries

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Menu
import androidx.compose.material3.BottomSheetDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.IconButton
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SearchBar
import androidx.compose.material3.SearchBarDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import dev.crqch.sunder.R
import dev.crqch.sunder.ui.components.CompressedEntryCard
import dev.crqch.sunder.ui.components.EntriesList

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun EntriesScreen(
    onAddEntry: (accountId: String?, categoryId: String?) -> Unit,
    onAddAccount: () -> Unit,
    onAddCategory: () -> Unit,
    viewModel: EntriesViewModel = hiltViewModel(),
    onEntryPick: (id: String) -> Unit
) {
    val accounts by viewModel.availableAccounts.collectAsState()
    val selectedAccountId by viewModel.selectedAccountId.collectAsState()
    val entries by viewModel.entries.collectAsState()
    val categories by viewModel.categories.collectAsState()
    val selectedCategoryId by viewModel.selectedCategoryId.collectAsState()

    val sheetState = rememberModalBottomSheetState()
    var showFilterModal by remember { mutableStateOf(false) }

    var searchBarExpanded by remember { mutableStateOf(false) }
    var searchBarQuery by remember { mutableStateOf("") }

    Scaffold(
        topBar = {
            Row(
                modifier = Modifier
                    .padding(8.dp, 0.dp)
                    .fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically
            ) {
                SearchBar(
                    inputField = {
                        SearchBarDefaults.InputField(
                            query = searchBarQuery,
                            onQueryChange = { new ->
                                searchBarQuery = new
                                viewModel.updateQuery(new.takeIf { it.isNotBlank() })
                            },
                            onSearch = { search ->
                                searchBarExpanded = false
                                viewModel.updateQuery(search.takeIf { it.isNotBlank() })
                            },
                            expanded = searchBarExpanded,
                            onExpandedChange = { exp ->
                                searchBarExpanded = exp
                                if (!exp) {
                                    searchBarQuery = ""
                                    viewModel.updateQuery(null)
                                }
                            },
                            placeholder = { Text(stringResource(R.string.search)) },
                            trailingIcon = {
                                IconButton(onClick = { showFilterModal = true }) {
                                    Icon(Icons.Filled.Menu, stringResource(R.string.menu))
                                }
                            }
                        )
                    },
                    expanded = searchBarExpanded,
                    onExpandedChange = { exp ->
                        searchBarExpanded = exp
                        if (!exp) {
                            searchBarQuery = ""
                            viewModel.updateQuery(null)
                        }
                    },
                    modifier = Modifier.weight(1f),
                    content = {
                        LazyColumn(
                            modifier = Modifier.fillMaxSize(),
                            contentPadding = PaddingValues(16.dp),
                            verticalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            items(
                                items = entries,
                                key = { it.entry.id }
                            ) { entryWithDetails ->
                                CompressedEntryCard(
                                    modifier = Modifier.animateItem(),
                                    entryWithDetails = entryWithDetails,
                                    onClick = { onEntryPick(entryWithDetails.entry.id) }
                                )
                            }
                        }
                    })
            }
        },
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
            EntriesList(
                entries,
                onEntryPick,
                filteredAccountId = selectedAccountId,
                filteredCategoryId = selectedCategoryId
            )
        }

    }

    if (showFilterModal)
        ModalBottomSheet(
            onDismissRequest = { showFilterModal = false },
            sheetState = sheetState,
            dragHandle = { BottomSheetDefaults.DragHandle() }
        ) {

            Column(
                Modifier.padding(16.dp, 8.dp),
                Arrangement.spacedBy(8.dp, Alignment.CenterVertically)
            ) {

                Text(
                    stringResource(R.string.account),
                    style = MaterialTheme.typography.labelLarge,
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

                    items(accounts, key = { it.account.id }) { accountWithBalance ->
                        FilterChip(
                            modifier = Modifier.animateItem(),
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
                    style = MaterialTheme.typography.labelLarge,
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
                            label = { Text(stringResource(R.string.any_category)) }
                        )
                    }

                    items(categories, key = { it.id }) { category ->
                        FilterChip(
                            modifier = Modifier.animateItem(),
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
            }

        }

}
