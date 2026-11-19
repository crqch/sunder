package dev.crqch.sunder.ui.entries

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
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
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ReceiptLong
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.ArrowDownward
import androidx.compose.material.icons.filled.ArrowUpward
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Tune
import androidx.compose.material3.Badge
import androidx.compose.material3.BadgedBox
import androidx.compose.material3.BottomSheetDefaults
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.InputChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import dev.crqch.sunder.R
import dev.crqch.sunder.ui.components.ConfirmDeleteDialog
import dev.crqch.sunder.ui.components.CurrencyPickerDialog
import dev.crqch.sunder.ui.components.EntriesList
import dev.crqch.sunder.ui.components.SelectionTopBar
import dev.crqch.sunder.ui.components.SunderEmptyState
import dev.crqch.sunder.ui.components.SunderSearchBar
import dev.crqch.sunder.ui.theme.sunderColors
import dev.crqch.sunder.utils.CurrencyFormatter
import dev.crqch.sunder.utils.normalizeCurrencyCode
import dev.crqch.sunder.utils.parseHexColor
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class, ExperimentalLayoutApi::class)
@Composable
fun EntriesScreen(
    onAddEntry: (accountId: String?, categoryId: String?) -> Unit,
    onAddAccount: () -> Unit,
    onAddCategory: () -> Unit,
    viewModel: EntriesViewModel = hiltViewModel(),
    onEntryPick: (id: String) -> Unit
) {
    val accounts by viewModel.availableAccounts.collectAsState()
    val categories by viewModel.categories.collectAsState()
    val allEntries by viewModel.allEntries.collectAsState()
    val entries by viewModel.entries.collectAsState()
    val filter by viewModel.filterState.collectAsState()
    val currencyState by viewModel.currencyState.collectAsState()

    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    var showFilterModal by remember { mutableStateOf(false) }
    var showCurrencyDialog by remember { mutableStateOf(false) }

    var selectedIds by remember { mutableStateOf<Set<String>>(emptySet()) }
    var showDeleteDialog by remember { mutableStateOf(false) }

    val snackbarHostState = remember { SnackbarHostState() }
    val scope = rememberCoroutineScope()
    val context = LocalContext.current

    val allSelected = entries.isNotEmpty() && selectedIds.size == entries.size
    val filteredNet = remember(entries, currencyState) {
        entries.sumOf { item ->
            currencyState.convertOrSame(
                item.entry.amount.toDouble(),
                item.entry.currencyCode,
                item.entry.date
            )
        }
    }
    val defaultCurrency = currencyState.defaultCurrency
    val entryCurrencyCodes = remember(allEntries) {
        allEntries.map { normalizeCurrencyCode(it.entry.currencyCode) }.distinct().sorted()
    }

    Scaffold(
        containerColor = MaterialTheme.colorScheme.background,
        snackbarHost = { SnackbarHost(snackbarHostState) },
        topBar = {
            TopAppBar(
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.background
                ),
                title = {
                    Text(
                        text = stringResource(R.string.entries),
                        style = MaterialTheme.typography.headlineSmall,
                        fontWeight = FontWeight.Bold
                    )
                },
                actions = {
                    IconButton(onClick = { viewModel.toggleSortDirection() }) {
                        Icon(
                            imageVector = if (filter.sortAscending) Icons.Default.ArrowUpward else Icons.Default.ArrowDownward,
                            contentDescription = stringResource(R.string.sort_by)
                        )
                    }
                    IconButton(onClick = { showFilterModal = true }) {
                        BadgedBox(
                            badge = {
                                if (filter.activeFilterCount > 0) {
                                    Badge(containerColor = MaterialTheme.colorScheme.primary) {
                                        Text(filter.activeFilterCount.toString())
                                    }
                                }
                            }
                        ) {
                            Icon(
                                imageVector = Icons.Default.Tune,
                                contentDescription = stringResource(R.string.filters)
                            )
                        }
                    }
                }
            )
        },
        floatingActionButton = {
            if (selectedIds.isEmpty()) {
                FloatingActionButton(
                    onClick = {
                        onAddEntry(
                            filter.selectedAccountIds.singleOrNull(),
                            filter.selectedCategoryIds.singleOrNull()
                        )
                    },
                    shape = RoundedCornerShape(18.dp),
                    containerColor = MaterialTheme.colorScheme.primary,
                    contentColor = MaterialTheme.colorScheme.onPrimary
                ) {
                    Icon(
                        imageVector = Icons.Default.Add,
                        contentDescription = stringResource(R.string.add_entry)
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
                label = "EntriesHeaderBar",
                modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp)
            ) { inSelectionMode ->
                if (inSelectionMode) {
                    SelectionTopBar(
                        selectedCount = selectedIds.size,
                        allSelected = allSelected,
                        onClearSelection = { selectedIds = emptySet() },
                        onToggleSelectAll = {
                            selectedIds = if (allSelected) emptySet()
                            else entries.map { it.entry.id }.toSet()
                        },
                        onDeleteSelected = { showDeleteDialog = true },
                        onChangeCurrency = { showCurrencyDialog = true }
                    )
                } else {
                    SunderSearchBar(
                        query = filter.query,
                        onQueryChange = { viewModel.updateQuery(it) },
                        placeholder = stringResource(R.string.search_entries)
                    )
                }
            }

            // Active Filter Chips Strip
            AnimatedVisibility(visible = filter.activeFilterCount > 0 && selectedIds.isEmpty()) {
                LazyRow(
                    modifier = Modifier.fillMaxWidth(),
                    contentPadding = PaddingValues(horizontal = 16.dp, vertical = 6.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    if (filter.typeFilter != EntryTypeFilter.ALL) {
                        item(key = "chip_type") {
                            InputChip(
                                selected = true,
                                onClick = { viewModel.setTypeFilter(EntryTypeFilter.ALL) },
                                label = {
                                    Text(
                                        stringResource(
                                            if (filter.typeFilter == EntryTypeFilter.EXPENSE) R.string.expenses
                                            else R.string.income
                                        )
                                    )
                                },
                                trailingIcon = {
                                    Icon(Icons.Default.Close, contentDescription = null, modifier = Modifier.size(16.dp))
                                }
                            )
                        }
                    }

                    if (filter.datePreset != DateRangePreset.ALL) {
                        item(key = "chip_date") {
                            val label = when (filter.datePreset) {
                                DateRangePreset.LAST_1M -> "1M"
                                DateRangePreset.LAST_3M -> "3M"
                                DateRangePreset.LAST_6M -> "6M"
                                DateRangePreset.LAST_1Y -> "1Y"
                                DateRangePreset.ALL -> ""
                            }
                            InputChip(
                                selected = true,
                                onClick = { viewModel.setDatePreset(DateRangePreset.ALL) },
                                label = { Text(label) },
                                trailingIcon = {
                                    Icon(Icons.Default.Close, contentDescription = null, modifier = Modifier.size(16.dp))
                                }
                            )
                        }
                    }

                    if (filter.minAmount != null || filter.maxAmount != null) {
                        item(key = "chip_amount") {
                            val rangeText = "${filter.minAmountInput.ifBlank { "0" }} – ${filter.maxAmountInput.ifBlank { "∞" }}"
                            InputChip(
                                selected = true,
                                onClick = {
                                    viewModel.updateMinAmount("")
                                    viewModel.updateMaxAmount("")
                                },
                                label = { Text(rangeText) },
                                trailingIcon = {
                                    Icon(Icons.Default.Close, contentDescription = null, modifier = Modifier.size(16.dp))
                                }
                            )
                        }
                    }

                    filter.selectedAccountIds.forEach { accId ->
                        val name = accounts.find { it.account.id == accId }?.account?.name
                        if (name != null) {
                            item(key = "chip_acc_$accId") {
                                InputChip(
                                    selected = true,
                                    onClick = { viewModel.toggleAccountFilter(accId) },
                                    label = { Text(name) },
                                    trailingIcon = {
                                        Icon(Icons.Default.Close, contentDescription = null, modifier = Modifier.size(16.dp))
                                    }
                                )
                            }
                        }
                    }

                    filter.selectedCategoryIds.forEach { catId ->
                        val cat = categories.find { it.id == catId }
                        if (cat != null) {
                            item(key = "chip_cat_$catId") {
                                InputChip(
                                    selected = true,
                                    onClick = { viewModel.toggleCategoryFilter(catId) },
                                    leadingIcon = {
                                        Box(
                                            modifier = Modifier
                                                .size(10.dp)
                                                .clip(CircleShape)
                                                .background(parseHexColor(cat.color))
                                        )
                                    },
                                    label = { Text(cat.title) },
                                    trailingIcon = {
                                        Icon(Icons.Default.Close, contentDescription = null, modifier = Modifier.size(16.dp))
                                    }
                                )
                            }
                        }
                    }

                    filter.selectedCurrencyCodes.forEach { code ->
                        item(key = "chip_currency_$code") {
                            InputChip(
                                selected = true,
                                onClick = { viewModel.toggleCurrencyFilter(code) },
                                label = { Text(code) },
                                trailingIcon = {
                                    Icon(Icons.Default.Close, contentDescription = null, modifier = Modifier.size(16.dp))
                                }
                            )
                        }
                    }

                    item(key = "chip_clear") {
                        TextButton(onClick = { viewModel.clearAllFilters() }) {
                            Text(stringResource(R.string.clear_all))
                        }
                    }
                }
            }

            // Summary strip when there are matching entries
            if (entries.isNotEmpty() && selectedIds.isEmpty()) {
                val sunderColors = MaterialTheme.sunderColors
                val netColor = when {
                    filteredNet > 0.0 -> sunderColors.income
                    filteredNet < 0.0 -> sunderColors.expense
                    else -> MaterialTheme.colorScheme.onSurfaceVariant
                }
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 20.dp, vertical = 6.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = "${entries.size} ${stringResource(R.string.transactions).lowercase()}",
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Text(
                        text = CurrencyFormatter.formatSigned(filteredNet, defaultCurrency),
                        style = MaterialTheme.typography.labelLarge,
                        fontWeight = FontWeight.Bold,
                        color = netColor
                    )
                }
            }

            EntriesList(
                entries = entries,
                onEntrySelect = { id ->
                    if (selectedIds.isNotEmpty()) {
                        selectedIds = if (id in selectedIds) selectedIds - id else selectedIds + id
                    } else {
                        onEntryPick(id)
                    }
                },
                onEntryLongPress = { id ->
                    selectedIds = if (id in selectedIds) selectedIds - id else selectedIds + id
                },
                selectedIds = selectedIds,
                filteredAccountId = filter.selectedAccountIds.singleOrNull(),
                filteredCategoryId = filter.selectedCategoryIds.singleOrNull(),
                groupByDate = filter.sortField == EntrySortField.DATE,
                currencyState = currencyState,
                emptyContent = {
                    if (allEntries.isEmpty()) {
                        SunderEmptyState(
                            icon = Icons.AutoMirrored.Filled.ReceiptLong,
                            title = stringResource(R.string.no_entries),
                            subtitle = stringResource(R.string.no_entries_subtitle),
                            actionLabel = stringResource(R.string.add_entry),
                            onAction = { onAddEntry(null, null) },
                            modifier = Modifier.padding(top = 24.dp)
                        )
                    } else {
                        SunderEmptyState(
                            icon = Icons.AutoMirrored.Filled.ReceiptLong,
                            title = stringResource(R.string.no_matching_results),
                            subtitle = stringResource(R.string.filters),
                            actionLabel = stringResource(R.string.clear_all),
                            onAction = { viewModel.clearAllFilters() },
                            modifier = Modifier.padding(top = 24.dp)
                        )
                    }
                }
            )
        }
    }

    if (showFilterModal) {
        ModalBottomSheet(
            onDismissRequest = { showFilterModal = false },
            sheetState = sheetState,
            dragHandle = { BottomSheetDefaults.DragHandle() }
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .verticalScroll(rememberScrollState())
                    .padding(horizontal = 20.dp)
                    .padding(bottom = 32.dp),
                verticalArrangement = Arrangement.spacedBy(18.dp)
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = stringResource(R.string.filters),
                        style = MaterialTheme.typography.titleLarge,
                        fontWeight = FontWeight.Bold
                    )
                    if (filter.hasAnyFilter) {
                        TextButton(onClick = { viewModel.clearAllFilters() }) {
                            Text(stringResource(R.string.clear_all))
                        }
                    }
                }

                // Sort By
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text(
                        text = stringResource(R.string.sort_by),
                        style = MaterialTheme.typography.labelLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        val sortOptions = listOf(
                            EntrySortField.DATE to R.string.sort_date,
                            EntrySortField.AMOUNT to R.string.sort_amount,
                            EntrySortField.TITLE to R.string.sort_title
                        )
                        SingleChoiceSegmentedButtonRow(modifier = Modifier.weight(1f)) {
                            sortOptions.forEachIndexed { idx, (field, labelRes) ->
                                SegmentedButton(
                                    selected = filter.sortField == field,
                                    onClick = { viewModel.setSortField(field) },
                                    shape = SegmentedButtonDefaults.itemShape(idx, sortOptions.size)
                                ) {
                                    Text(stringResource(labelRes))
                                }
                            }
                        }
                        Surface(
                            onClick = { viewModel.toggleSortDirection() },
                            shape = RoundedCornerShape(12.dp),
                            border = BorderStroke(1.dp, MaterialTheme.sunderColors.cardBorder),
                            color = MaterialTheme.colorScheme.surfaceContainerLow,
                            modifier = Modifier.size(42.dp)
                        ) {
                            Box(contentAlignment = Alignment.Center) {
                                Icon(
                                    imageVector = if (filter.sortAscending) Icons.Default.ArrowUpward else Icons.Default.ArrowDownward,
                                    contentDescription = null,
                                    modifier = Modifier.size(20.dp)
                                )
                            }
                        }
                    }
                }

                // Entry Type
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    val types = listOf(
                        EntryTypeFilter.ALL to R.string.all,
                        EntryTypeFilter.EXPENSE to R.string.expenses,
                        EntryTypeFilter.INCOME to R.string.income
                    )
                    SingleChoiceSegmentedButtonRow(modifier = Modifier.fillMaxWidth()) {
                        types.forEachIndexed { idx, (type, labelRes) ->
                            SegmentedButton(
                                selected = filter.typeFilter == type,
                                onClick = { viewModel.setTypeFilter(type) },
                                shape = SegmentedButtonDefaults.itemShape(idx, types.size)
                            ) {
                                Text(stringResource(labelRes))
                            }
                        }
                    }
                }

                // Date Range Presets
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text(
                        text = stringResource(R.string.date_range),
                        style = MaterialTheme.typography.labelLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    val presets = listOf(
                        DateRangePreset.ALL to stringResource(R.string.all),
                        DateRangePreset.LAST_1M to "1M",
                        DateRangePreset.LAST_3M to "3M",
                        DateRangePreset.LAST_6M to "6M",
                        DateRangePreset.LAST_1Y to "1Y"
                    )
                    FlowRow(
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalArrangement = Arrangement.spacedBy(6.dp)
                    ) {
                        presets.forEach { (preset, label) ->
                            FilterChip(
                                selected = filter.datePreset == preset,
                                onClick = { viewModel.setDatePreset(preset) },
                                label = { Text(label) }
                            )
                        }
                    }
                }

                // Amount Range
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text(
                        text = stringResource(R.string.amount_range),
                        style = MaterialTheme.typography.labelLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        OutlinedTextField(
                            value = filter.minAmountInput,
                            onValueChange = { viewModel.updateMinAmount(it) },
                            label = { Text(stringResource(R.string.min_amount)) },
                            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                            singleLine = true,
                            modifier = Modifier.weight(1f),
                            shape = RoundedCornerShape(14.dp)
                        )
                        OutlinedTextField(
                            value = filter.maxAmountInput,
                            onValueChange = { viewModel.updateMaxAmount(it) },
                            label = { Text(stringResource(R.string.max_amount)) },
                            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                            singleLine = true,
                            modifier = Modifier.weight(1f),
                            shape = RoundedCornerShape(14.dp)
                        )
                    }
                }

                HorizontalDivider(color = MaterialTheme.sunderColors.cardBorder)

                // Accounts Multi-select
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text(
                        text = stringResource(R.string.accounts),
                        style = MaterialTheme.typography.labelLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    FlowRow(
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalArrangement = Arrangement.spacedBy(6.dp)
                    ) {
                        FilterChip(
                            selected = filter.selectedAccountIds.isEmpty(),
                            onClick = { viewModel.clearAccountFilters() },
                            label = { Text(stringResource(R.string.any_filter)) }
                        )
                        accounts.forEach { acc ->
                            FilterChip(
                                selected = acc.account.id in filter.selectedAccountIds,
                                onClick = { viewModel.toggleAccountFilter(acc.account.id) },
                                label = { Text(acc.account.name) }
                            )
                        }
                        FilterChip(
                            selected = false,
                            onClick = {
                                showFilterModal = false
                                onAddAccount()
                            },
                            leadingIcon = {
                                Icon(Icons.Default.Add, contentDescription = null, modifier = Modifier.size(16.dp))
                            },
                            label = { Text(stringResource(R.string.add_account)) }
                        )
                    }
                }

                // Categories Multi-select
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text(
                        text = stringResource(R.string.categories),
                        style = MaterialTheme.typography.labelLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    FlowRow(
                        horizontalArrangement = Arrangement.spacedBy(8.dp),
                        verticalArrangement = Arrangement.spacedBy(6.dp)
                    ) {
                        FilterChip(
                            selected = filter.selectedCategoryIds.isEmpty(),
                            onClick = { viewModel.clearCategoryFilters() },
                            label = { Text(stringResource(R.string.any_category)) }
                        )
                        categories.forEach { cat ->
                            FilterChip(
                                selected = cat.id in filter.selectedCategoryIds,
                                onClick = { viewModel.toggleCategoryFilter(cat.id) },
                                leadingIcon = {
                                    Box(
                                        modifier = Modifier
                                            .size(10.dp)
                                            .clip(CircleShape)
                                            .background(parseHexColor(cat.color))
                                    )
                                },
                                label = { Text(cat.title) }
                            )
                        }
                        FilterChip(
                            selected = false,
                            onClick = {
                                showFilterModal = false
                                onAddCategory()
                            },
                            leadingIcon = {
                                Icon(Icons.Default.Add, contentDescription = null, modifier = Modifier.size(16.dp))
                            },
                            label = { Text(stringResource(R.string.add_category)) }
                        )
                    }
                }

                if (entryCurrencyCodes.size > 1) {
                    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        Text(
                            text = stringResource(R.string.filter_currencies),
                            style = MaterialTheme.typography.labelLarge,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        FlowRow(
                            horizontalArrangement = Arrangement.spacedBy(8.dp),
                            verticalArrangement = Arrangement.spacedBy(6.dp)
                        ) {
                            FilterChip(
                                selected = filter.selectedCurrencyCodes.isEmpty(),
                                onClick = { viewModel.clearCurrencyFilters() },
                                label = { Text(stringResource(R.string.any_currency)) }
                            )
                            entryCurrencyCodes.forEach { code ->
                                FilterChip(
                                    selected = code in filter.selectedCurrencyCodes,
                                    onClick = { viewModel.toggleCurrencyFilter(code) },
                                    label = { Text(code) }
                                )
                            }
                        }
                    }
                }

                Spacer(modifier = Modifier.height(4.dp))

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    OutlinedButton(
                        onClick = { viewModel.clearAllFilters() },
                        modifier = Modifier
                            .weight(1f)
                            .height(48.dp),
                        shape = RoundedCornerShape(14.dp)
                    ) {
                        Text(stringResource(R.string.clear_all))
                    }
                    Button(
                        onClick = { showFilterModal = false },
                        modifier = Modifier
                            .weight(1f)
                            .height(48.dp),
                        shape = RoundedCornerShape(14.dp)
                    ) {
                        Text(stringResource(R.string.apply))
                    }
                }
            }
        }
    }

    if (showCurrencyDialog) {
        CurrencyPickerDialog(
            currencies = currencyState.currencies,
            selectedCode = null,
            onSelect = { code ->
                showCurrencyDialog = false
                if (code != null) {
                    val ids = selectedIds.toList()
                    selectedIds = emptySet()
                    viewModel.updateEntriesCurrency(ids, code) { count ->
                        scope.launch {
                            snackbarHostState.showSnackbar(
                                context.getString(R.string.currency_updated, count)
                            )
                        }
                    }
                }
            },
            onDismiss = { showCurrencyDialog = false }
        )
    }

    if (showDeleteDialog) {
        ConfirmDeleteDialog(
            title = stringResource(R.string.delete_selected_title),
            message = stringResource(R.string.delete_selected_message, selectedIds.size),
            onConfirm = {
                val idsToDelete = selectedIds.toList()
                showDeleteDialog = false
                selectedIds = emptySet()
                viewModel.deleteEntries(idsToDelete)
            },
            onDismiss = { showDeleteDialog = false }
        )
    }
}
