package dev.crqch.sunder.ui.accounts

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.ReceiptLong
import androidx.compose.material.icons.filled.AccountBalanceWallet
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.ChevronRight
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.CurrencyExchange
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CenterAlignedTopAppBar
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExtendedFloatingActionButton
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
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
import dev.crqch.sunder.ui.components.CurrencyPickerDialog
import dev.crqch.sunder.ui.components.EntriesList
import dev.crqch.sunder.ui.components.SunderEmptyState
import dev.crqch.sunder.ui.theme.SunderMotion
import dev.crqch.sunder.ui.theme.sunderColors
import dev.crqch.sunder.utils.CurrencyFormatter
import kotlin.math.abs

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AccountScreen(
    onNavigateBack: () -> Unit,
    onEntryClick: (String) -> Unit,
    onAddEntry: (accountId: String) -> Unit,
    viewModel: AccountViewModel = hiltViewModel(),
) {
    val account by viewModel.account.collectAsState()
    val entries by viewModel.entries.collectAsState()
    val currencyState by viewModel.currencyState.collectAsState()
    val accountFormState = viewModel.accountFormState
    var showDeleteDialog by remember { mutableStateOf(false) }
    var showCurrencyDialog by remember { mutableStateOf(false) }

    val defaultCurrency = currencyState.defaultCurrency
    val balance = remember(entries, currencyState) {
        entries.sumOf { item ->
            currencyState.convertOrSame(
                item.entry.amount.toDouble(),
                item.entry.currencyCode,
                item.entry.date
            )
        }
    }
    val income = remember(entries, currencyState) {
        entries.filter { it.entry.amount > 0f }.sumOf { item ->
            currencyState.convertOrSame(
                item.entry.amount.toDouble(),
                item.entry.currencyCode,
                item.entry.date
            )
        }
    }
    val expenses = remember(entries, currencyState) {
        entries.filter { it.entry.amount < 0f }.sumOf { item ->
            abs(
                currencyState.convertOrSame(
                    item.entry.amount.toDouble(),
                    item.entry.currencyCode,
                    item.entry.date
                )
            )
        }
    }

    account?.let { currentAccount ->
        Scaffold(
            containerColor = MaterialTheme.colorScheme.background,
            topBar = {
                CenterAlignedTopAppBar(
                    colors = TopAppBarDefaults.centerAlignedTopAppBarColors(
                        containerColor = MaterialTheme.colorScheme.background
                    ),
                    title = {
                        Text(
                            text = currentAccount.name,
                            fontWeight = FontWeight.SemiBold,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    },
                    navigationIcon = {
                        IconButton(
                            onClick = {
                                if (viewModel.isEditMode) viewModel.cancelEdit()
                                else onNavigateBack()
                            }
                        ) {
                            Icon(
                                imageVector = if (viewModel.isEditMode) Icons.Default.Close else Icons.AutoMirrored.Filled.ArrowBack,
                                contentDescription = stringResource(
                                    if (viewModel.isEditMode) R.string.cancel else R.string.back
                                )
                            )
                        }
                    },
                    actions = {
                        if (viewModel.isEditMode) {
                            IconButton(
                                onClick = { viewModel.saveEdit() },
                                enabled = accountFormState.isFilled()
                            ) {
                                Icon(
                                    imageVector = Icons.Default.Check,
                                    contentDescription = stringResource(R.string.save),
                                    tint = if (accountFormState.isFilled()) MaterialTheme.colorScheme.primary
                                    else MaterialTheme.colorScheme.onSurface.copy(alpha = 0.38f)
                                )
                            }
                        } else {
                            IconButton(onClick = { viewModel.startEdit() }) {
                                Icon(
                                    imageVector = Icons.Default.Edit,
                                    contentDescription = stringResource(R.string.edit)
                                )
                            }
                            IconButton(onClick = { showDeleteDialog = true }) {
                                Icon(
                                    imageVector = Icons.Default.Delete,
                                    contentDescription = stringResource(R.string.delete_account),
                                    tint = MaterialTheme.colorScheme.error
                                )
                            }
                        }
                    }
                )
            },
            floatingActionButton = {
                if (!viewModel.isEditMode) {
                    ExtendedFloatingActionButton(
                        onClick = { onAddEntry(currentAccount.id) },
                        icon = {
                            Icon(
                                imageVector = Icons.Default.Add,
                                contentDescription = null
                            )
                        },
                        text = { Text(stringResource(R.string.add_entry)) },
                        shape = RoundedCornerShape(18.dp),
                        containerColor = MaterialTheme.colorScheme.primary,
                        contentColor = MaterialTheme.colorScheme.onPrimary
                    )
                }
            }
        ) { padding ->
            AnimatedContent(
                targetState = viewModel.isEditMode,
                transitionSpec = {
                    fadeIn(SunderMotion.enterTween()) togetherWith fadeOut(SunderMotion.exitTween())
                },
                label = "AccountEditTransition",
                modifier = Modifier
                    .fillMaxSize()
                    .padding(padding)
            ) { isEditing ->
                if (isEditing) {
                    Column(
                        modifier = Modifier
                            .fillMaxSize()
                            .padding(horizontal = 20.dp, vertical = 16.dp),
                        verticalArrangement = Arrangement.spacedBy(20.dp)
                    ) {
                        Surface(
                            modifier = Modifier.fillMaxWidth(),
                            shape = MaterialTheme.shapes.large,
                            color = MaterialTheme.colorScheme.surface,
                            border = BorderStroke(1.dp, MaterialTheme.sunderColors.cardBorder)
                        ) {
                            Column(
                                modifier = Modifier.padding(18.dp),
                                verticalArrangement = Arrangement.spacedBy(12.dp)
                            ) {
                                Text(
                                    text = stringResource(R.string.account_name),
                                    style = MaterialTheme.typography.labelLarge,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                                OutlinedTextField(
                                    value = accountFormState.name,
                                    onValueChange = { viewModel.updateForm(accountFormState.copy(name = it)) },
                                    placeholder = { Text(stringResource(R.string.account_name)) },
                                    leadingIcon = {
                                        Icon(Icons.Default.AccountBalanceWallet, contentDescription = null)
                                    },
                                    modifier = Modifier.fillMaxWidth(),
                                    shape = RoundedCornerShape(14.dp),
                                    singleLine = true,
                                    colors = OutlinedTextFieldDefaults.colors(
                                        focusedBorderColor = MaterialTheme.colorScheme.primary,
                                        unfocusedBorderColor = MaterialTheme.sunderColors.cardBorder
                                    )
                                )

                                Text(
                                    text = stringResource(R.string.default_currency_desc),
                                    style = MaterialTheme.typography.bodySmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant
                                )

                                val formCurrency = accountFormState.currencyCode?.let {
                                    currencyState.currency(it)
                                }
                                Surface(
                                    onClick = { showCurrencyDialog = true },
                                    modifier = Modifier.fillMaxWidth(),
                                    shape = RoundedCornerShape(14.dp),
                                    color = MaterialTheme.colorScheme.surfaceContainerLow,
                                    border = BorderStroke(1.dp, MaterialTheme.sunderColors.cardBorder)
                                ) {
                                    Row(
                                        modifier = Modifier
                                            .fillMaxWidth()
                                            .padding(horizontal = 14.dp, vertical = 12.dp),
                                        verticalAlignment = Alignment.CenterVertically,
                                        horizontalArrangement = Arrangement.spacedBy(12.dp)
                                    ) {
                                        Icon(
                                            imageVector = Icons.Default.CurrencyExchange,
                                            contentDescription = null,
                                            tint = MaterialTheme.colorScheme.primary
                                        )
                                        Column(modifier = Modifier.weight(1f)) {
                                            Text(
                                                text = stringResource(R.string.default_currency),
                                                style = MaterialTheme.typography.labelMedium,
                                                color = MaterialTheme.colorScheme.onSurfaceVariant
                                            )
                                            Text(
                                                text = formCurrency?.let {
                                                    "${it.code} (${it.symbol}) · ${it.name}"
                                                } ?: stringResource(R.string.default_currency_none),
                                                style = MaterialTheme.typography.titleSmall,
                                                fontWeight = FontWeight.SemiBold
                                            )
                                        }
                                        Icon(
                                            imageVector = Icons.Default.ChevronRight,
                                            contentDescription = null,
                                            tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                                        )
                                    }
                                }
                            }
                        }

                        if (viewModel.accountId != null) {
                            FilledTonalButton(
                                onClick = { showDeleteDialog = true },
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .height(50.dp),
                                shape = RoundedCornerShape(14.dp),
                                colors = ButtonDefaults.filledTonalButtonColors(
                                    contentColor = MaterialTheme.colorScheme.error,
                                    containerColor = MaterialTheme.colorScheme.errorContainer
                                )
                            ) {
                                Icon(
                                    imageVector = Icons.Default.Delete,
                                    contentDescription = null,
                                    modifier = Modifier.size(ButtonDefaults.IconSize)
                                )
                                Spacer(Modifier.width(ButtonDefaults.IconSpacing))
                                Text(stringResource(R.string.delete_account))
                            }
                        }
                    }
                } else {
                    val sunderColors = MaterialTheme.sunderColors
                    val balanceColor = if (balance >= 0.0) sunderColors.income else sunderColors.expense
                    val heroGlow = if (balance >= 0.0) {
                        sunderColors.income.copy(alpha = if (sunderColors.isDark) 0.16f else 0.08f)
                    } else {
                        sunderColors.expense.copy(alpha = if (sunderColors.isDark) 0.16f else 0.08f)
                    }

                    EntriesList(
                        entries = entries,
                        onEntrySelect = onEntryClick,
                        filteredAccountId = currentAccount.id,
                        filteredCategoryId = null,
                        currencyState = currencyState,
                        contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 8.dp, bottom = 104.dp),
                        headerContent = {
                            item(key = "account_balance_hero") {
                                Surface(
                                    modifier = Modifier.fillMaxWidth(),
                                    shape = MaterialTheme.shapes.extraLarge,
                                    color = MaterialTheme.colorScheme.surface,
                                    border = BorderStroke(1.dp, sunderColors.cardBorder)
                                ) {
                                    Box(
                                        modifier = Modifier
                                            .fillMaxWidth()
                                            .background(
                                                Brush.linearGradient(
                                                    colors = listOf(heroGlow, Color.Transparent)
                                                )
                                            )
                                            .padding(20.dp)
                                    ) {
                                        Column(
                                            verticalArrangement = Arrangement.spacedBy(14.dp)
                                        ) {
                                            Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                                                Row(
                                                    verticalAlignment = Alignment.CenterVertically,
                                                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                                                ) {
                                                    Text(
                                                        text = stringResource(R.string.current_balance),
                                                        style = MaterialTheme.typography.labelLarge,
                                                        color = MaterialTheme.colorScheme.onSurfaceVariant
                                                    )
                                                    currentAccount.currencyCode?.let { code ->
                                                        CurrencyCodeBadge(code = code)
                                                    }
                                                }
                                                Text(
                                                    text = CurrencyFormatter.format(balance, defaultCurrency),
                                                    style = MaterialTheme.typography.displaySmall,
                                                    fontWeight = FontWeight.Bold,
                                                    color = balanceColor
                                                )
                                            }
                                            Row(
                                                modifier = Modifier.fillMaxWidth(),
                                                horizontalArrangement = Arrangement.spacedBy(12.dp)
                                            ) {
                                                AccountMiniStat(
                                                    label = stringResource(R.string.income),
                                                    value = CurrencyFormatter.format(
                                                        income,
                                                        defaultCurrency,
                                                        showPlusSign = income > 0.0
                                                    ),
                                                    valueColor = sunderColors.income,
                                                    modifier = Modifier.weight(1f)
                                                )
                                                AccountMiniStat(
                                                    label = stringResource(R.string.expenses),
                                                    value = CurrencyFormatter.format(
                                                        -expenses,
                                                        defaultCurrency,
                                                        showMinusSign = expenses > 0.0
                                                    ),
                                                    valueColor = sunderColors.expense,
                                                    modifier = Modifier.weight(1f)
                                                )
                                            }
                                        }
                                    }
                                }
                            }
                            item(key = "account_transactions_header") {
                                Row(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .padding(top = 8.dp, bottom = 2.dp),
                                    horizontalArrangement = Arrangement.SpaceBetween,
                                    verticalAlignment = Alignment.CenterVertically
                                ) {
                                    Text(
                                        text = stringResource(R.string.transactions),
                                        style = MaterialTheme.typography.titleLarge,
                                        fontWeight = FontWeight.Bold
                                    )
                                    Text(
                                        text = "${entries.size}",
                                        style = MaterialTheme.typography.labelLarge,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant
                                    )
                                }
                            }
                        },
                        emptyContent = {
                            SunderEmptyState(
                                icon = Icons.AutoMirrored.Filled.ReceiptLong,
                                title = stringResource(R.string.no_entries),
                                subtitle = stringResource(R.string.no_entries_subtitle),
                                actionLabel = stringResource(R.string.add_entry),
                                onAction = { onAddEntry(currentAccount.id) },
                                modifier = Modifier.padding(top = 8.dp)
                            )
                        }
                    )
                }
            }
        }
    }

    if (showCurrencyDialog) {
        CurrencyPickerDialog(
            currencies = currencyState.currencies,
            selectedCode = accountFormState.currencyCode,
            allowNullOption = true,
            onSelect = { code ->
                viewModel.selectCurrency(code)
                showCurrencyDialog = false
            },
            onDismiss = { showCurrencyDialog = false }
        )
    }

    if (showDeleteDialog) {
        ConfirmDeleteDialog(
            title = stringResource(R.string.delete_account),
            message = stringResource(R.string.confirm_delete_account),
            onConfirm = {
                showDeleteDialog = false
                viewModel.delete { onNavigateBack() }
            },
            onDismiss = { showDeleteDialog = false }
        )
    }
}

@Composable
private fun AccountMiniStat(
    label: String,
    value: String,
    valueColor: Color,
    modifier: Modifier = Modifier
) {
    Surface(
        modifier = modifier,
        shape = RoundedCornerShape(14.dp),
        color = MaterialTheme.colorScheme.surfaceContainerLow,
        border = BorderStroke(1.dp, MaterialTheme.sunderColors.cardBorder.copy(alpha = 0.6f))
    ) {
        Column(
            modifier = Modifier.padding(horizontal = 12.dp, vertical = 10.dp),
            verticalArrangement = Arrangement.spacedBy(2.dp)
        ) {
            Text(
                text = label,
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant
            )
            Text(
                text = value,
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold,
                color = valueColor,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }
    }
}
