package dev.crqch.sunder.ui.accounts

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateColorAsState
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AccountBalanceWallet
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.ChevronRight
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import dev.crqch.sunder.R
import dev.crqch.sunder.ui.components.ConfirmDeleteDialog
import dev.crqch.sunder.ui.components.CurrencyCodeBadge
import dev.crqch.sunder.ui.components.SelectionCheckCircle
import dev.crqch.sunder.ui.components.SelectionTopBar
import dev.crqch.sunder.ui.components.SunderEmptyState
import dev.crqch.sunder.ui.components.SunderSearchBar
import dev.crqch.sunder.ui.theme.sunderColors
import dev.crqch.sunder.utils.CurrencyFormatter

@OptIn(ExperimentalMaterial3Api::class, ExperimentalFoundationApi::class)
@Composable
fun AccountsScreen(
    onAddAccount: () -> Unit,
    viewModel: AccountsViewModel = hiltViewModel(),
    visitAccount: (accountId: String) -> Unit,
) {
    val allAccounts by viewModel.allAccounts.collectAsState()
    val accounts by viewModel.accounts.collectAsState()
    val searchQuery by viewModel.searchQuery.collectAsState()
    val currencyState by viewModel.currencyState.collectAsState()
    val convertedBalances by viewModel.convertedBalances.collectAsState()
    val totalBalance by viewModel.totalBalance.collectAsState()
    val defaultCurrency = currencyState.defaultCurrency

    var selectedIds by remember { mutableStateOf<Set<String>>(emptySet()) }
    var showDeleteDialog by remember { mutableStateOf(false) }

    val allSelected = accounts.isNotEmpty() && selectedIds.size == accounts.size

    Scaffold(
        containerColor = MaterialTheme.colorScheme.background,
        topBar = {
            TopAppBar(
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.background
                ),
                title = {
                    Text(
                        text = stringResource(R.string.accounts),
                        style = MaterialTheme.typography.headlineSmall,
                        fontWeight = FontWeight.Bold
                    )
                }
            )
        },
        floatingActionButton = {
            if (selectedIds.isEmpty()) {
                FloatingActionButton(
                    onClick = onAddAccount,
                    shape = RoundedCornerShape(18.dp),
                    containerColor = MaterialTheme.colorScheme.primary,
                    contentColor = MaterialTheme.colorScheme.onPrimary
                ) {
                    Icon(
                        imageVector = Icons.Default.Add,
                        contentDescription = stringResource(R.string.add_account)
                    )
                }
            }
        }
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
        ) {
            AnimatedContent(
                targetState = selectedIds.isNotEmpty(),
                label = "AccountsHeaderMode",
                modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp)
            ) { inSelectionMode ->
                if (inSelectionMode) {
                    SelectionTopBar(
                        selectedCount = selectedIds.size,
                        allSelected = allSelected,
                        onClearSelection = { selectedIds = emptySet() },
                        onToggleSelectAll = {
                            selectedIds = if (allSelected) emptySet()
                            else accounts.map { it.account.id }.toSet()
                        },
                        onDeleteSelected = { showDeleteDialog = true }
                    )
                } else if (allAccounts.isNotEmpty()) {
                    SunderSearchBar(
                        query = searchQuery,
                        onQueryChange = { viewModel.updateSearchQuery(it) },
                        placeholder = stringResource(R.string.search_accounts)
                    )
                }
            }

            if (allAccounts.isEmpty()) {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(24.dp),
                    contentAlignment = Alignment.Center
                ) {
                    SunderEmptyState(
                        icon = Icons.Default.AccountBalanceWallet,
                        title = stringResource(R.string.no_accounts),
                        subtitle = stringResource(R.string.create_first_account),
                        actionLabel = stringResource(R.string.add_account),
                        onAction = onAddAccount
                    )
                }
            } else {
                LazyColumn(
                    modifier = Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 10.dp, bottom = 96.dp),
                    verticalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    if (searchQuery.isBlank() && selectedIds.isEmpty()) {
                        item(key = "accounts_total_banner") {
                            val sunderColors = MaterialTheme.sunderColors
                            val balColor = if (totalBalance >= 0.0) sunderColors.income else sunderColors.expense
                            Surface(
                                modifier = Modifier.fillMaxWidth(),
                                shape = MaterialTheme.shapes.large,
                                color = MaterialTheme.colorScheme.surface,
                                border = BorderStroke(1.dp, sunderColors.cardBorder)
                            ) {
                                Box(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .background(
                                            Brush.horizontalGradient(
                                                colors = listOf(
                                                    MaterialTheme.colorScheme.primary.copy(
                                                        alpha = if (sunderColors.isDark) 0.18f else 0.08f
                                                    ),
                                                    Color.Transparent
                                                )
                                            )
                                        )
                                        .padding(horizontal = 18.dp, vertical = 16.dp)
                                ) {
                                    Row(
                                        modifier = Modifier.fillMaxWidth(),
                                        horizontalArrangement = Arrangement.SpaceBetween,
                                        verticalAlignment = Alignment.CenterVertically
                                    ) {
                                        Column {
                                            Text(
                                                text = stringResource(R.string.total_balance),
                                                style = MaterialTheme.typography.labelMedium,
                                                color = MaterialTheme.colorScheme.onSurfaceVariant
                                            )
                                            Text(
                                                text = CurrencyFormatter.format(totalBalance, defaultCurrency),
                                                style = MaterialTheme.typography.headlineMedium,
                                                fontWeight = FontWeight.Bold,
                                                color = balColor
                                            )
                                        }
                                        Surface(
                                            shape = RoundedCornerShape(12.dp),
                                            color = MaterialTheme.colorScheme.primaryContainer
                                        ) {
                                            Text(
                                                text = "${allAccounts.size}",
                                                style = MaterialTheme.typography.titleMedium,
                                                fontWeight = FontWeight.Bold,
                                                color = MaterialTheme.colorScheme.onPrimaryContainer,
                                                modifier = Modifier.padding(horizontal = 12.dp, vertical = 6.dp)
                                            )
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if (accounts.isEmpty()) {
                        item(key = "accounts_no_match") {
                            SunderEmptyState(
                                icon = Icons.Default.AccountBalanceWallet,
                                title = stringResource(R.string.no_matching_results),
                                subtitle = stringResource(R.string.search_accounts)
                            )
                        }
                    } else {
                        items(accounts, key = { it.account.id }) { accountWithBalance ->
                            val account = accountWithBalance.account
                            val isSelected = selectedIds.contains(account.id)
                            val sunderColors = MaterialTheme.sunderColors
                            val accountBalance = convertedBalances[account.id]
                                ?: accountWithBalance.balance.toDouble()
                            val balanceColor = if (accountBalance >= 0.0) {
                                sunderColors.income
                            } else {
                                sunderColors.expense
                            }

                            val cardBg by animateColorAsState(
                                targetValue = if (isSelected) {
                                    MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.35f)
                                } else {
                                    MaterialTheme.colorScheme.surface
                                },
                                label = "AccountCardBg"
                            )
                            val cardBorder by animateColorAsState(
                                targetValue = if (isSelected) {
                                    MaterialTheme.colorScheme.primary
                                } else {
                                    sunderColors.cardBorder
                                },
                                label = "AccountCardBorder"
                            )

                            Surface(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .animateItem()
                                    .clip(MaterialTheme.shapes.large)
                                    .combinedClickable(
                                        onClick = {
                                            if (selectedIds.isNotEmpty()) {
                                                selectedIds = if (isSelected) selectedIds - account.id
                                                else selectedIds + account.id
                                            } else {
                                                visitAccount(account.id)
                                            }
                                        },
                                        onLongClick = {
                                            selectedIds = if (isSelected) selectedIds - account.id
                                            else selectedIds + account.id
                                        }
                                    ),
                                shape = MaterialTheme.shapes.large,
                                color = cardBg,
                                border = BorderStroke(if (isSelected) 1.5.dp else 1.dp, cardBorder)
                            ) {
                                Row(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .padding(16.dp),
                                    verticalAlignment = Alignment.CenterVertically
                                ) {
                                    AnimatedVisibility(visible = selectedIds.isNotEmpty()) {
                                        Row {
                                            SelectionCheckCircle(selected = isSelected)
                                            Spacer(modifier = Modifier.width(12.dp))
                                        }
                                    }

                                    Surface(
                                        modifier = Modifier.size(44.dp),
                                        shape = CircleShape,
                                        color = MaterialTheme.colorScheme.primaryContainer
                                    ) {
                                        Box(contentAlignment = Alignment.Center) {
                                            Text(
                                                text = account.name.take(1).uppercase(),
                                                style = MaterialTheme.typography.titleMedium,
                                                fontWeight = FontWeight.Bold,
                                                color = MaterialTheme.colorScheme.onPrimaryContainer
                                            )
                                        }
                                    }

                                    Spacer(modifier = Modifier.width(14.dp))

                                    Column(modifier = Modifier.weight(1f)) {
                                        Row(
                                            verticalAlignment = Alignment.CenterVertically,
                                            horizontalArrangement = Arrangement.spacedBy(6.dp)
                                        ) {
                                            Text(
                                                text = account.name,
                                                style = MaterialTheme.typography.titleMedium,
                                                fontWeight = FontWeight.SemiBold,
                                                maxLines = 1,
                                                overflow = TextOverflow.Ellipsis,
                                                modifier = Modifier.weight(1f, fill = false)
                                            )
                                            account.currencyCode?.let { code ->
                                                CurrencyCodeBadge(code = code)
                                            }
                                        }
                                        Text(
                                            text = stringResource(R.string.current_balance),
                                            style = MaterialTheme.typography.labelSmall,
                                            color = MaterialTheme.colorScheme.onSurfaceVariant
                                        )
                                    }

                                    Spacer(modifier = Modifier.width(10.dp))

                                    Text(
                                        text = CurrencyFormatter.format(accountBalance, defaultCurrency),
                                        style = MaterialTheme.typography.titleMedium,
                                        fontWeight = FontWeight.Bold,
                                        color = balanceColor
                                    )

                                    Spacer(modifier = Modifier.width(4.dp))

                                    Icon(
                                        imageVector = Icons.Default.ChevronRight,
                                        contentDescription = null,
                                        tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f),
                                        modifier = Modifier.size(20.dp)
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if (showDeleteDialog) {
        ConfirmDeleteDialog(
            title = stringResource(R.string.delete_selected_title),
            message = stringResource(R.string.delete_selected_message, selectedIds.size),
            onConfirm = {
                val idsToDelete = selectedIds.toList()
                showDeleteDialog = false
                selectedIds = emptySet()
                viewModel.deleteAccounts(idsToDelete)
            },
            onDismiss = { showDeleteDialog = false }
        )
    }
}