package dev.crqch.sunder.ui.entries

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.*
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import dev.crqch.sunder.R
import dev.crqch.sunder.ui.components.SelectionItem
import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.time.format.FormatStyle

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CreateEntryScreen(
    onNavigateBack: () -> Unit,
    onAddCategory: () -> Unit,
    onAddAccount: () -> Unit,
    viewModel: EntriesViewModel = hiltViewModel(),
) {
    val entryFormState = viewModel.entryFormState
    val accounts by viewModel.availableAccounts.collectAsState()
    val categories by viewModel.categories.collectAsState()

    val amountInput = viewModel.amountInput

    var selectionType by remember { mutableStateOf(SelectionType.NONE) }
    val sheetState = rememberModalBottomSheetState()
    var showSheet by remember { mutableStateOf(false) }

    var showDatePicker by remember { mutableStateOf(false) }
    val datePickerState = rememberDatePickerState(
        initialSelectedDateMillis = entryFormState.date,
        selectableDates = object : SelectableDates {
            override fun isSelectableDate(utcTimeMillis: Long): Boolean {
                return utcTimeMillis <= System.currentTimeMillis()
            }
        }
    )

    val selectedAccount = accounts.find { it.account.id == entryFormState.accountId }
    val selectedCategory = categories.find { it.id == entryFormState.categoryId }

    val dateFormatter = remember { DateTimeFormatter.ofLocalizedDate(FormatStyle.MEDIUM) }
    val formattedDate = remember(entryFormState.date) {
        Instant.ofEpochMilli(entryFormState.date)
            .atZone(ZoneId.systemDefault())
            .toLocalDate()
            .format(dateFormatter)
    }

    val glintColor = if (entryFormState.isExpense) {
        MaterialTheme.colorScheme.errorContainer
    } else {
        MaterialTheme.colorScheme.primaryContainer
    }

    val animatedGlintColor by animateColorAsState(
        targetValue = glintColor,
        animationSpec = tween(durationMillis = 500),
        label = "GlintColorAnimation"
    )

    Box(modifier = Modifier.fillMaxSize()) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(400.dp)
                .background(
                    Brush.verticalGradient(
                        colors = listOf(
                            animatedGlintColor.copy(alpha = 0.5f),
                            Color.Transparent
                        )
                    )
                )
        )

        Scaffold(
            containerColor = Color.Transparent,
            contentWindowInsets = WindowInsets.safeDrawing,
            topBar = {
                CenterAlignedTopAppBar(
                    colors = TopAppBarDefaults.centerAlignedTopAppBarColors(
                        containerColor = Color.Transparent
                    ),
                    title = { Text(stringResource(R.string.new_entry)) },
                    navigationIcon = {
                        IconButton(onClick = onNavigateBack) {
                            Icon(
                                Icons.AutoMirrored.Filled.ArrowBack,
                                contentDescription = stringResource(
                                    R.string.back
                                )
                            )
                        }
                    }
                )
            },
            bottomBar = {
                Surface(
                    tonalElevation = 3.dp,
                    shadowElevation = 8.dp
                ) {
                    Button(
                        onClick = {
                            viewModel.createEntry(entryFormState) {
                                onNavigateBack()
                            }
                        },
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(16.dp)
                            .navigationBarsPadding()
                            .imePadding()
                            .height(56.dp),
                        enabled = entryFormState.isFilled(),
                        shape = MaterialTheme.shapes.extraLarge
                    ) {
                        Text(
                            stringResource(R.string.create_entry),
                            style = MaterialTheme.typography.titleMedium
                        )
                    }
                }
            }
        ) { padding ->
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(padding)
                    .consumeWindowInsets(padding)
                    .verticalScroll(rememberScrollState())
                    .padding(horizontal = 20.dp, vertical = 24.dp),
                verticalArrangement = Arrangement.spacedBy(24.dp)
            ) {
                Column(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    SingleChoiceSegmentedButtonRow(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(horizontal = 4.dp)
                    ) {
                        SegmentedButton(
                            selected = entryFormState.isExpense,
                            onClick = { viewModel.updateForm(entryFormState.copy(isExpense = true)) },
                            shape = SegmentedButtonDefaults.itemShape(index = 0, count = 2),
                            colors = SegmentedButtonDefaults.colors(
                                activeContainerColor = MaterialTheme.colorScheme.errorContainer,
                                activeContentColor = MaterialTheme.colorScheme.onErrorContainer
                            )
                        ) {
                            Text(stringResource(R.string.expense))
                        }
                        SegmentedButton(
                            selected = !entryFormState.isExpense,
                            onClick = { viewModel.updateForm(entryFormState.copy(isExpense = false)) },
                            shape = SegmentedButtonDefaults.itemShape(index = 1, count = 2),
                            colors = SegmentedButtonDefaults.colors(
                                activeContainerColor = MaterialTheme.colorScheme.primaryContainer,
                                activeContentColor = MaterialTheme.colorScheme.onPrimaryContainer
                            )
                        ) {
                            Text(stringResource(R.string.income))
                        }
                    }

                    Text(
                        stringResource(R.string.amount),
                        style = MaterialTheme.typography.titleSmall,
                        color = MaterialTheme.colorScheme.primary,
                        fontWeight = FontWeight.Bold
                    )
                    TextField(
                        value = amountInput,
                        onValueChange = { viewModel.updateAmount(it) },
                        textStyle = MaterialTheme.typography.displayLarge.copy(
                            textAlign = TextAlign.Center,
                            fontWeight = FontWeight.Bold,
                            color = if (entryFormState.isExpense) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.primary
                        ),
                        placeholder = {
                            Text(
                                "0.00",
                                modifier = Modifier.fillMaxWidth(),
                                textAlign = TextAlign.Center,
                                style = MaterialTheme.typography.displayLarge.copy(
                                    fontWeight = FontWeight.Bold,
                                    color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.3f)
                                )
                            )
                        },
                        keyboardOptions = KeyboardOptions(
                            keyboardType = KeyboardType.Decimal
                        ),
                        modifier = Modifier.fillMaxWidth(),
                        colors = TextFieldDefaults.colors(
                            focusedContainerColor = Color.Transparent,
                            unfocusedContainerColor = Color.Transparent,
                            disabledContainerColor = Color.Transparent,
                            focusedIndicatorColor = if (entryFormState.isExpense) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.primary,
                            unfocusedIndicatorColor = MaterialTheme.colorScheme.outlineVariant.copy(
                                alpha = 0.5f
                            ),
                            cursorColor = if (entryFormState.isExpense) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.primary
                        ),
                        singleLine = true
                    )
                }

                Card(
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.3f)
                    ),
                    shape = MaterialTheme.shapes.extraLarge
                ) {
                    Column {
                        SelectorField(
                            label = stringResource(R.string.account),
                            selectedName = selectedAccount?.account?.name
                                ?: stringResource(R.string.select_account),
                            icon = Icons.Default.AccountBalance,
                            onClick = {
                                selectionType = SelectionType.ACCOUNT
                                showSheet = true
                            }
                        )
                        HorizontalDivider(
                            modifier = Modifier.padding(horizontal = 16.dp),
                            color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.5f)
                        )
                        SelectorField(
                            label = stringResource(R.string.category),
                            selectedName = selectedCategory?.title
                                ?: stringResource(R.string.select_category),
                            icon = Icons.Default.Category,
                            onClick = {
                                selectionType = SelectionType.CATEGORY
                                showSheet = true
                            }
                        )
                        HorizontalDivider(
                            modifier = Modifier.padding(horizontal = 16.dp),
                            color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.5f)
                        )
                        SelectorField(
                            label = stringResource(R.string.date),
                            selectedName = formattedDate,
                            icon = Icons.Default.CalendarToday,
                            onClick = {
                                showDatePicker = true
                            }
                        )
                    }
                }

                Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                    OutlinedTextField(
                        value = entryFormState.title,
                        onValueChange = { viewModel.updateForm(entryFormState.copy(title = it)) },
                        label = { Text(stringResource(R.string.title)) },
                        placeholder = { Text(stringResource(R.string.what_is_this_for)) },
                        leadingIcon = { Icon(Icons.Default.Title, contentDescription = null) },
                        modifier = Modifier.fillMaxWidth(),
                        shape = MaterialTheme.shapes.large,
                        singleLine = true,
                        colors = OutlinedTextFieldDefaults.colors(
                            focusedBorderColor = MaterialTheme.colorScheme.primary,
                            unfocusedBorderColor = MaterialTheme.colorScheme.outlineVariant
                        )
                    )

                    OutlinedTextField(
                        value = entryFormState.description,
                        onValueChange = { viewModel.updateForm(entryFormState.copy(description = it)) },
                        label = { Text(stringResource(R.string.description)) },
                        placeholder = { Text(stringResource(R.string.add_more_details)) },
                        leadingIcon = {
                            Box(modifier = Modifier.fillMaxHeight()) {
                                Icon(
                                    Icons.Default.Description,
                                    contentDescription = null,
                                    modifier = Modifier
                                        .align(Alignment.TopCenter)
                                        .padding(top = 16.dp)
                                )
                            }
                        },
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(IntrinsicSize.Min),
                        shape = MaterialTheme.shapes.large,
                        minLines = 3,
                        colors = OutlinedTextFieldDefaults.colors(
                            focusedBorderColor = MaterialTheme.colorScheme.primary,
                            unfocusedBorderColor = MaterialTheme.colorScheme.outlineVariant
                        )
                    )
                }
            }
        }

        if (showSheet) {
            ModalBottomSheet(
                onDismissRequest = { showSheet = false },
                sheetState = sheetState,
                dragHandle = { BottomSheetDefaults.DragHandle() }
            ) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(bottom = 32.dp)
                ) {
                    Text(
                        text = if (selectionType == SelectionType.ACCOUNT) stringResource(R.string.select_account) else stringResource(
                            R.string.select_category
                        ),
                        style = MaterialTheme.typography.titleLarge,
                        modifier = Modifier.padding(horizontal = 24.dp, vertical = 16.dp)
                    )

                    LazyColumn(
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        item {
                            SelectionItem(
                                label = stringResource(R.string.add_new),
                                icon = Icons.Default.Add,
                                onClick = {
                                    showSheet = false
                                    if (selectionType == SelectionType.ACCOUNT) onAddAccount() else onAddCategory()
                                },
                                isHighlight = true
                            )
                        }

                        if (selectionType == SelectionType.ACCOUNT) {
                            items(accounts) { accountWithBalance ->
                                SelectionItem(
                                    label = accountWithBalance.account.name,
                                    isSelected = accountWithBalance.account.id == entryFormState.accountId,
                                    onClick = {
                                        viewModel.updateForm(entryFormState.copy(accountId = accountWithBalance.account.id))
                                        showSheet = false
                                    }
                                )
                            }
                        } else {
                            items(categories) { category ->
                                SelectionItem(
                                    label = category.title,
                                    isSelected = category.id == entryFormState.categoryId,
                                    onClick = {
                                        viewModel.updateForm(entryFormState.copy(categoryId = category.id))
                                        showSheet = false
                                    }
                                )
                            }
                        }
                    }
                }
            }
        }

        if (showDatePicker) {
            DatePickerDialog(
                onDismissRequest = { showDatePicker = false },
                confirmButton = {
                    TextButton(onClick = {
                        datePickerState.selectedDateMillis?.let {
                            viewModel.updateForm(entryFormState.copy(date = it))
                        }
                        showDatePicker = false
                    }) {
                        Text(stringResource(R.string.ok))
                    }
                },
                dismissButton = {
                    TextButton(onClick = { showDatePicker = false }) {
                        Text(stringResource(R.string.cancel))
                    }
                }
            ) {
                DatePicker(state = datePickerState)
            }
        }
    }
}

@Composable
fun SelectorField(
    label: String,
    selectedName: String,
    icon: ImageVector,
    onClick: () -> Unit
) {
    Surface(
        onClick = onClick,
        color = Color.Transparent,
        modifier = Modifier.fillMaxWidth()
    ) {
        Row(
            modifier = Modifier
                .padding(16.dp)
                .fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                Surface(
                    color = MaterialTheme.colorScheme.primaryContainer,
                    shape = MaterialTheme.shapes.medium,
                    modifier = Modifier.size(40.dp)
                ) {
                    Box(contentAlignment = Alignment.Center) {
                        Icon(
                            icon,
                            contentDescription = null,
                            tint = MaterialTheme.colorScheme.onPrimaryContainer,
                            modifier = Modifier.size(20.dp)
                        )
                    }
                }
                Column {
                    Text(
                        label,
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Text(
                        text = selectedName,
                        style = MaterialTheme.typography.titleMedium,
                        color = if (selectedName.startsWith(stringResource(R.string.select))) {
                            MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                        } else {
                            MaterialTheme.colorScheme.onSurface
                        }
                    )
                }
            }
            Icon(
                Icons.Default.ChevronRight,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}
