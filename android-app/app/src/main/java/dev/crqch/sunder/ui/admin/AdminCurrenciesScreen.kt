package dev.crqch.sunder.ui.admin

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.CenterAlignedTopAppBar
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import dev.crqch.sunder.R
import dev.crqch.sunder.data.local.CurrencyEntity
import dev.crqch.sunder.ui.components.CurrencyCodeBadge
import dev.crqch.sunder.ui.theme.sunderColors
import dev.crqch.sunder.utils.CurrencyFormatter

@OptIn(ExperimentalMaterial3Api::class, ExperimentalLayoutApi::class)
@Composable
fun AdminCurrenciesScreen(
    onNavigateBack: () -> Unit,
    viewModel: AdminCurrenciesViewModel = hiltViewModel()
) {
    val currencies by viewModel.currencies.collectAsState()
    val isLoading by viewModel.isLoading.collectAsState()
    val isSyncingRates by viewModel.isSyncingRates.collectAsState()
    val isSavingCurrency by viewModel.isSavingCurrency.collectAsState()
    val status by viewModel.status.collectAsState()
    val editor = viewModel.editor

    var searchQuery by remember { mutableStateOf("") }
    var filter by remember { mutableStateOf(AdminCurrencyFilter.ALL) }
    var syncDateInput by remember { mutableStateOf("") }
    var backfillDaysInput by remember { mutableStateOf("30") }
    var manualTargetInput by remember { mutableStateOf("") }
    var manualRateInput by remember { mutableStateOf("") }
    var manualDateInput by remember { mutableStateOf("") }

    val snackbarHostState = remember { SnackbarHostState() }
    val statusText = if (status != null) adminStatusText(status!!) else null

    LaunchedEffect(status) {
        statusText?.let { snackbarHostState.showSnackbar(it) }
    }

    val activeCount = currencies.count { it.isActive }
    val filteredCurrencies = remember(currencies, searchQuery, filter) {
        val term = searchQuery.trim()
        currencies.filter { currency ->
            val matchesFilter = when (filter) {
                AdminCurrencyFilter.ALL -> true
                AdminCurrencyFilter.ACTIVE -> currency.isActive
                AdminCurrencyFilter.INACTIVE -> !currency.isActive
            }
            val matchesQuery = term.isEmpty() ||
                currency.code.contains(term, ignoreCase = true) ||
                currency.name.contains(term, ignoreCase = true) ||
                currency.symbol.contains(term, ignoreCase = true)
            matchesFilter && matchesQuery
        }
    }

    Scaffold(
        containerColor = MaterialTheme.colorScheme.background,
        snackbarHost = { SnackbarHost(snackbarHostState) },
        topBar = {
            CenterAlignedTopAppBar(
                colors = TopAppBarDefaults.centerAlignedTopAppBarColors(
                    containerColor = MaterialTheme.colorScheme.background
                ),
                title = {
                    Text(
                        text = stringResource(R.string.admin_currencies),
                        fontWeight = FontWeight.SemiBold,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                },
                navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(
                            imageVector = Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = stringResource(R.string.back)
                        )
                    }
                }
            )
        }
    ) { innerPadding ->
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding),
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            item(key = "rates_card") {
                AdminCard(title = stringResource(R.string.exchange_rates)) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        Button(
                            onClick = { viewModel.syncLatestRates() },
                            enabled = !isSyncingRates,
                            modifier = Modifier.weight(1f),
                            shape = RoundedCornerShape(14.dp)
                        ) {
                            Icon(
                                imageVector = Icons.Default.Refresh,
                                contentDescription = null,
                                modifier = Modifier.size(18.dp)
                            )
                            Spacer(modifier = Modifier.width(8.dp))
                            Text(stringResource(R.string.sync_latest_rates))
                        }
                        if (isSyncingRates) {
                            CircularProgressIndicator(
                                modifier = Modifier.size(22.dp),
                                strokeWidth = 2.5.dp
                            )
                        }
                    }

                    HorizontalDivider(color = MaterialTheme.sunderColors.cardBorder)

                    Text(
                        text = stringResource(R.string.sync_specific_date),
                        style = MaterialTheme.typography.labelLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    OutlinedTextField(
                        value = syncDateInput,
                        onValueChange = { syncDateInput = it },
                        label = { Text(stringResource(R.string.rate_date)) },
                        placeholder = { Text("2026-11-19") },
                        singleLine = true,
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(14.dp)
                    )
                    OutlinedButton(
                        onClick = { viewModel.syncDate(syncDateInput) },
                        enabled = !isSyncingRates && syncDateInput.isNotBlank(),
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(14.dp)
                    ) {
                        Text(stringResource(R.string.sync_date))
                    }

                    HorizontalDivider(color = MaterialTheme.sunderColors.cardBorder)

                    Text(
                        text = stringResource(R.string.backfill_rates),
                        style = MaterialTheme.typography.labelLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        OutlinedTextField(
                            value = backfillDaysInput,
                            onValueChange = { backfillDaysInput = it.filter { ch -> ch.isDigit() }.take(4) },
                            label = { Text(stringResource(R.string.backfill_days)) },
                            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                            singleLine = true,
                            modifier = Modifier.weight(1f),
                            shape = RoundedCornerShape(14.dp)
                        )
                        OutlinedButton(
                            onClick = { viewModel.backfill(backfillDaysInput) },
                            enabled = !isSyncingRates && backfillDaysInput.isNotBlank(),
                            shape = RoundedCornerShape(14.dp)
                        ) {
                            Text(stringResource(R.string.backfill))
                        }
                    }

                    HorizontalDivider(color = MaterialTheme.sunderColors.cardBorder)

                    Text(
                        text = stringResource(R.string.manual_rate_override),
                        style = MaterialTheme.typography.labelLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(10.dp)
                    ) {
                        OutlinedTextField(
                            value = manualTargetInput,
                            onValueChange = { manualTargetInput = it.uppercase().take(3) },
                            label = { Text(stringResource(R.string.target_currency)) },
                            singleLine = true,
                            modifier = Modifier.weight(1f),
                            shape = RoundedCornerShape(14.dp)
                        )
                        OutlinedTextField(
                            value = manualRateInput,
                            onValueChange = { manualRateInput = it },
                            label = { Text(stringResource(R.string.rate)) },
                            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                            singleLine = true,
                            modifier = Modifier.weight(1f),
                            shape = RoundedCornerShape(14.dp)
                        )
                    }
                    OutlinedTextField(
                        value = manualDateInput,
                        onValueChange = { manualDateInput = it },
                        label = { Text(stringResource(R.string.rate_date)) },
                        placeholder = { Text(stringResource(R.string.optional)) },
                        singleLine = true,
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(14.dp)
                    )
                    OutlinedButton(
                        onClick = {
                            viewModel.saveManualRate(
                                targetCode = manualTargetInput,
                                rateInput = manualRateInput,
                                date = manualDateInput
                            )
                        },
                        enabled = !isSyncingRates && manualTargetInput.isNotBlank() && manualRateInput.isNotBlank(),
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(14.dp)
                    ) {
                        Text(stringResource(R.string.save_rate))
                    }

                    statusText?.let { message ->
                        Surface(
                            modifier = Modifier.fillMaxWidth(),
                            shape = RoundedCornerShape(12.dp),
                            color = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.45f)
                        ) {
                            Text(
                                text = message,
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onPrimaryContainer,
                                modifier = Modifier.padding(horizontal = 12.dp, vertical = 9.dp)
                            )
                        }
                    }
                }
            }

            item(key = "currencies_card") {
                AdminCard(title = stringResource(R.string.supported_currencies)) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.SpaceBetween
                    ) {
                        Text(
                            text = stringResource(R.string.active_currency_count, activeCount, currencies.size),
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        Button(
                            onClick = { viewModel.openCreateCurrency() },
                            shape = RoundedCornerShape(12.dp),
                            contentPadding = PaddingValues(horizontal = 12.dp, vertical = 6.dp)
                        ) {
                            Icon(
                                imageVector = Icons.Default.Add,
                                contentDescription = null,
                                modifier = Modifier.size(16.dp)
                            )
                            Spacer(modifier = Modifier.width(6.dp))
                            Text(
                                text = stringResource(R.string.add_currency),
                                style = MaterialTheme.typography.labelLarge
                            )
                        }
                    }

                    OutlinedTextField(
                        value = searchQuery,
                        onValueChange = { searchQuery = it },
                        placeholder = { Text(stringResource(R.string.search_currency)) },
                        leadingIcon = {
                            Icon(
                                imageVector = Icons.Default.Search,
                                contentDescription = null,
                                tint = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        },
                        singleLine = true,
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(14.dp)
                    )

                    FlowRow(
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalArrangement = Arrangement.spacedBy(6.dp)
                    ) {
                        AdminCurrencyFilter.entries.forEach { option ->
                            FilterChip(
                                selected = filter == option,
                                onClick = { filter = option },
                                label = {
                                    Text(
                                        stringResource(
                                            when (option) {
                                                AdminCurrencyFilter.ALL -> R.string.all
                                                AdminCurrencyFilter.ACTIVE -> R.string.active
                                                AdminCurrencyFilter.INACTIVE -> R.string.inactive
                                            }
                                        )
                                    )
                                }
                            )
                        }
                    }

                    HorizontalDivider(color = MaterialTheme.sunderColors.cardBorder)

                    if (isLoading && currencies.isEmpty()) {
                        Box(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(vertical = 20.dp),
                            contentAlignment = Alignment.Center
                        ) {
                            CircularProgressIndicator(modifier = Modifier.size(26.dp))
                        }
                    } else if (filteredCurrencies.isEmpty()) {
                        Text(
                            text = stringResource(R.string.no_matching_currencies),
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.padding(vertical = 12.dp)
                        )
                    } else {
                        Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                            filteredCurrencies.forEach { currency ->
                                AdminCurrencyRow(
                                    currency = currency,
                                    enabled = !isSavingCurrency,
                                    onToggleActive = { active ->
                                        viewModel.setCurrencyActive(currency, active)
                                    },
                                    onEdit = { viewModel.openEditCurrency(currency) }
                                )
                            }
                        }
                    }
                }
            }

            item(key = "admin_footer") {
                Spacer(modifier = Modifier.height(8.dp))
            }
        }
    }

    if (editor != null) {
        AdminCurrencyEditorDialog(
            editor = editor,
            isSaving = isSavingCurrency,
            onUpdate = viewModel::updateEditor,
            onSave = { viewModel.saveEditor() },
            onDismiss = { viewModel.closeEditor() }
        )
    }
}

@Composable
private fun AdminCard(
    title: String,
    content: @Composable () -> Unit
) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = MaterialTheme.shapes.large,
        color = MaterialTheme.colorScheme.surface,
        border = BorderStroke(1.dp, MaterialTheme.sunderColors.cardBorder)
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(18.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            Text(
                text = title,
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold
            )
            content()
        }
    }
}

@Composable
private fun AdminCurrencyRow(
    currency: CurrencyEntity,
    enabled: Boolean,
    onToggleActive: (Boolean) -> Unit,
    onEdit: () -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .padding(horizontal = 4.dp, vertical = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        CurrencyCodeBadge(code = currency.code)
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = currency.name,
                style = MaterialTheme.typography.titleSmall,
                fontWeight = FontWeight.SemiBold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
            Text(
                text = "${currency.symbol} · ${CurrencyFormatter.format(1234.5, currency)}",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1
            )
        }
        Switch(
            checked = currency.isActive,
            onCheckedChange = onToggleActive,
            enabled = enabled
        )
        IconButton(onClick = onEdit, enabled = enabled) {
            Icon(
                imageVector = Icons.Default.Edit,
                contentDescription = stringResource(R.string.edit),
                modifier = Modifier.size(20.dp)
            )
        }
    }
}

@Composable
private fun AdminCurrencyEditorDialog(
    editor: AdminCurrencyEditor,
    isSaving: Boolean,
    onUpdate: ((AdminCurrencyEditor) -> AdminCurrencyEditor) -> Unit,
    onSave: () -> Unit,
    onDismiss: () -> Unit
) {
    val isEditing = editor.editingCode != null
    AlertDialog(
        onDismissRequest = onDismiss,
        title = {
            Text(
                text = if (isEditing) {
                    stringResource(R.string.edit_currency, editor.editingCode.orEmpty())
                } else {
                    stringResource(R.string.add_currency)
                },
                style = MaterialTheme.typography.titleLarge
            )
        },
        text = {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .heightIn(max = 440.dp)
                    .verticalScroll(rememberScrollState()),
                verticalArrangement = Arrangement.spacedBy(10.dp)
            ) {
                OutlinedTextField(
                    value = editor.code,
                    onValueChange = { value ->
                        onUpdate {
                            it.copy(
                                code = value.uppercase().take(3),
                                error = null
                            )
                        }
                    },
                    enabled = !isEditing && !isSaving,
                    label = { Text(stringResource(R.string.currency_code)) },
                    placeholder = { Text("EUR") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(14.dp)
                )
                OutlinedTextField(
                    value = editor.name,
                    onValueChange = { value ->
                        onUpdate { it.copy(name = value, error = null) }
                    },
                    enabled = !isSaving,
                    label = { Text(stringResource(R.string.currency_name)) },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(14.dp)
                )
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(10.dp)
                ) {
                    OutlinedTextField(
                        value = editor.symbol,
                        onValueChange = { value ->
                            onUpdate { it.copy(symbol = value.take(5), error = null) }
                        },
                        enabled = !isSaving,
                        label = { Text(stringResource(R.string.currency_symbol)) },
                        singleLine = true,
                        modifier = Modifier.weight(1f),
                        shape = RoundedCornerShape(14.dp)
                    )
                    OutlinedTextField(
                        value = editor.decimalDigits,
                        onValueChange = { value ->
                            onUpdate {
                                it.copy(
                                    decimalDigits = value.filter { ch -> ch.isDigit() }.take(1),
                                    error = null
                                )
                            }
                        },
                        enabled = !isSaving,
                        label = { Text(stringResource(R.string.decimal_digits_short)) },
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                        singleLine = true,
                        modifier = Modifier.weight(1f),
                        shape = RoundedCornerShape(14.dp)
                    )
                }
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Text(
                        text = stringResource(R.string.symbol_after),
                        style = MaterialTheme.typography.bodyMedium
                    )
                    Switch(
                        checked = editor.symbolAfter,
                        onCheckedChange = { value ->
                            onUpdate { it.copy(symbolAfter = value) }
                        },
                        enabled = !isSaving
                    )
                }
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Text(
                        text = stringResource(R.string.active),
                        style = MaterialTheme.typography.bodyMedium
                    )
                    Switch(
                        checked = editor.isActive,
                        onCheckedChange = { value ->
                            onUpdate { it.copy(isActive = value) }
                        },
                        enabled = !isSaving
                    )
                }

                editor.error?.let { error ->
                    Text(
                        text = stringResource(
                            when (error) {
                                AdminEditorError.CODE -> R.string.invalid_currency_code
                                AdminEditorError.NAME -> R.string.invalid_currency_name
                                AdminEditorError.DIGITS -> R.string.invalid_decimal_digits
                            }
                        ),
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.error
                    )
                }
            }
        },
        confirmButton = {
            Button(
                onClick = onSave,
                enabled = !isSaving,
                shape = RoundedCornerShape(12.dp)
            ) {
                if (isSaving) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(16.dp),
                        strokeWidth = 2.dp,
                        color = MaterialTheme.colorScheme.onPrimary
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                }
                Text(stringResource(R.string.save_currency))
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss, enabled = !isSaving) {
                Text(stringResource(R.string.cancel))
            }
        }
    )
}

@Composable
private fun adminStatusText(status: AdminStatus): String = when (status) {
    is AdminStatus.RatesSynced -> {
        val summary = status.summary
        val range = summary.date
            ?: listOfNotNull(summary.from, summary.to)
                .distinct()
                .joinToString(" – ")
                .ifBlank { "—" }
        stringResource(R.string.rates_synced, range, summary.storedCount ?: 0)
    }

    is AdminStatus.RateStored -> stringResource(R.string.rate_saved, status.code)
    is AdminStatus.CurrencyStored -> stringResource(R.string.currency_saved, status.code)
    is AdminStatus.CurrencyToggled -> stringResource(
        R.string.currency_toggle_status,
        status.code,
        stringResource(if (status.active) R.string.active else R.string.inactive)
    )

    AdminStatus.InvalidDate -> stringResource(R.string.invalid_date)
    AdminStatus.InvalidRate -> stringResource(R.string.invalid_rate)
    AdminStatus.InvalidDays -> stringResource(R.string.invalid_days)
    AdminStatus.InvalidCode -> stringResource(R.string.invalid_currency_code)
    is AdminStatus.Failed -> status.detail
        ?.let { stringResource(R.string.operation_failed_detail, it) }
        ?: stringResource(R.string.operation_failed)
}
