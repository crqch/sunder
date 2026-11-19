package dev.crqch.sunder.ui.entries

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawing
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.AccountBalanceWallet
import androidx.compose.material.icons.filled.CalendarToday
import androidx.compose.material.icons.filled.Category
import androidx.compose.material.icons.filled.ChevronRight
import androidx.compose.material.icons.filled.Description
import androidx.compose.material.icons.filled.LocationOn
import androidx.compose.material.icons.filled.Title
import androidx.compose.material3.BottomSheetDefaults
import androidx.compose.material3.Button
import androidx.compose.material3.CenterAlignedTopAppBar
import androidx.compose.material3.DatePicker
import androidx.compose.material3.DatePickerDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.material3.rememberDatePickerState
import androidx.compose.material3.rememberModalBottomSheetState
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
import dev.crqch.sunder.ui.components.SelectionSheet
import dev.crqch.sunder.ui.theme.sunderColors
import dev.crqch.sunder.utils.formatCurrencyAmount
import dev.crqch.sunder.utils.parseHexColor
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

    // Auto-select first available account/category if none pre-selected (matches web EntryForm)
    LaunchedEffect(accounts) {
        if (entryFormState.accountId.isBlank() && accounts.size == 1) {
            viewModel.updateForm(entryFormState.copy(accountId = accounts.first().account.id))
        }
    }
    LaunchedEffect(categories) {
        if (entryFormState.categoryId.isBlank() && categories.size == 1) {
            viewModel.updateForm(entryFormState.copy(categoryId = categories.first().id))
        }
    }

    val amountInput = viewModel.amountInput
    val sunderColors = MaterialTheme.sunderColors

    var selectionType by remember { mutableStateOf(SelectionType.NONE) }
    val sheetState = rememberModalBottomSheetState()
    var showSheet by remember { mutableStateOf(false) }

    var showDatePicker by remember { mutableStateOf(false) }
    val datePickerState = rememberDatePickerState(
        initialSelectedDateMillis = entryFormState.date
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

    val accentColor by animateColorAsState(
        targetValue = if (entryFormState.isExpense) sunderColors.expense else sunderColors.income,
        animationSpec = tween(durationMillis = 320),
        label = "AccentColorAnim"
    )

    val glintColor by animateColorAsState(
        targetValue = if (entryFormState.isExpense) sunderColors.expenseContainer else sunderColors.incomeContainer,
        animationSpec = tween(durationMillis = 380),
        label = "GlintColorAnimation"
    )

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(MaterialTheme.colorScheme.background)
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(340.dp)
                .background(
                    Brush.verticalGradient(
                        colors = listOf(
                            glintColor.copy(alpha = if (sunderColors.isDark) 0.45f else 0.55f),
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
                    title = {
                        Text(
                            text = stringResource(R.string.new_entry),
                            fontWeight = FontWeight.SemiBold
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
            },
            bottomBar = {
                Surface(
                    color = MaterialTheme.colorScheme.surface,
                    tonalElevation = 2.dp
                ) {
                    Button(
                        onClick = {
                            viewModel.createEntry(entryFormState) {
                                onNavigateBack()
                            }
                        },
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(horizontal = 16.dp, vertical = 14.dp)
                            .navigationBarsPadding()
                            .imePadding()
                            .height(54.dp),
                        enabled = entryFormState.isFilled(),
                        shape = RoundedCornerShape(16.dp)
                    ) {
                        Text(
                            text = stringResource(R.string.create_entry),
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
                    .padding(horizontal = 20.dp, vertical = 16.dp),
                verticalArrangement = Arrangement.spacedBy(20.dp)
            ) {
                Column(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    SingleChoiceSegmentedButtonRow(modifier = Modifier.fillMaxWidth()) {
                        SegmentedButton(
                            selected = entryFormState.isExpense,
                            onClick = { viewModel.updateForm(entryFormState.copy(isExpense = true)) },
                            shape = SegmentedButtonDefaults.itemShape(index = 0, count = 2),
                            colors = SegmentedButtonDefaults.colors(
                                activeContainerColor = sunderColors.expenseContainer,
                                activeContentColor = sunderColors.onExpenseContainer
                            )
                        ) {
                            Text(stringResource(R.string.expense))
                        }
                        SegmentedButton(
                            selected = !entryFormState.isExpense,
                            onClick = { viewModel.updateForm(entryFormState.copy(isExpense = false)) },
                            shape = SegmentedButtonDefaults.itemShape(index = 1, count = 2),
                            colors = SegmentedButtonDefaults.colors(
                                activeContainerColor = sunderColors.incomeContainer,
                                activeContentColor = sunderColors.onIncomeContainer
                            )
                        ) {
                            Text(stringResource(R.string.income))
                        }
                    }

                    Text(
                        text = stringResource(R.string.amount),
                        style = MaterialTheme.typography.labelLarge,
                        color = accentColor,
                        fontWeight = FontWeight.SemiBold
                    )

                    TextField(
                        value = amountInput,
                        onValueChange = { viewModel.updateAmount(it) },
                        textStyle = MaterialTheme.typography.displayLarge.copy(
                            textAlign = TextAlign.Center,
                            fontWeight = FontWeight.Bold,
                            color = accentColor
                        ),
                        placeholder = {
                            Text(
                                text = "0.00",
                                modifier = Modifier.fillMaxWidth(),
                                textAlign = TextAlign.Center,
                                style = MaterialTheme.typography.displayLarge.copy(
                                    fontWeight = FontWeight.Bold,
                                    color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.25f)
                                )
                            )
                        },
                        visualTransformation = CentsAmountVisualTransformation,
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                        modifier = Modifier.fillMaxWidth(),
                        colors = TextFieldDefaults.colors(
                            focusedContainerColor = Color.Transparent,
                            unfocusedContainerColor = Color.Transparent,
                            disabledContainerColor = Color.Transparent,
                            focusedIndicatorColor = accentColor,
                            unfocusedIndicatorColor = sunderColors.cardBorder,
                            cursorColor = accentColor
                        ),
                        singleLine = true
                    )
                }

                // Account, Category, Date Selectors Card
                Surface(
                    shape = MaterialTheme.shapes.large,
                    color = MaterialTheme.colorScheme.surface,
                    border = BorderStroke(1.dp, sunderColors.cardBorder)
                ) {
                    Column {
                        SelectorField(
                            label = stringResource(R.string.account),
                            selectedName = selectedAccount?.account?.name
                                ?: stringResource(R.string.select_account),
                            isPlaceholder = selectedAccount == null,
                            icon = Icons.Default.AccountBalanceWallet,
                            onClick = {
                                selectionType = SelectionType.ACCOUNT
                                showSheet = true
                            }
                        )
                        HorizontalDivider(
                            modifier = Modifier.padding(horizontal = 16.dp),
                            color = sunderColors.cardBorder
                        )
                        SelectorField(
                            label = stringResource(R.string.category),
                            selectedName = selectedCategory?.title
                                ?: stringResource(R.string.select_category),
                            isPlaceholder = selectedCategory == null,
                            icon = Icons.Default.Category,
                            badgeColor = selectedCategory?.let { parseHexColor(it.color) },
                            onClick = {
                                selectionType = SelectionType.CATEGORY
                                showSheet = true
                            }
                        )
                        HorizontalDivider(
                            modifier = Modifier.padding(horizontal = 16.dp),
                            color = sunderColors.cardBorder
                        )
                        SelectorField(
                            label = stringResource(R.string.date),
                            selectedName = formattedDate,
                            isPlaceholder = false,
                            icon = Icons.Default.CalendarToday,
                            onClick = { showDatePicker = true }
                        )
                    }
                }

                // Title, Location, Note Card
                Column(verticalArrangement = Arrangement.spacedBy(14.dp)) {
                    OutlinedTextField(
                        value = entryFormState.title,
                        onValueChange = { viewModel.updateForm(entryFormState.copy(title = it)) },
                        label = { Text(stringResource(R.string.title)) },
                        placeholder = { Text(stringResource(R.string.what_is_this_for)) },
                        leadingIcon = { Icon(Icons.Default.Title, contentDescription = null) },
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(16.dp),
                        singleLine = true,
                        colors = OutlinedTextFieldDefaults.colors(
                            focusedContainerColor = MaterialTheme.colorScheme.surface,
                            unfocusedContainerColor = MaterialTheme.colorScheme.surface,
                            focusedBorderColor = MaterialTheme.colorScheme.primary,
                            unfocusedBorderColor = sunderColors.cardBorder
                        )
                    )

                    OutlinedTextField(
                        value = entryFormState.location,
                        onValueChange = { viewModel.updateForm(entryFormState.copy(location = it)) },
                        label = { Text(stringResource(R.string.location)) },
                        placeholder = { Text(stringResource(R.string.location_placeholder)) },
                        leadingIcon = { Icon(Icons.Default.LocationOn, contentDescription = null) },
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(16.dp),
                        singleLine = true,
                        colors = OutlinedTextFieldDefaults.colors(
                            focusedContainerColor = MaterialTheme.colorScheme.surface,
                            unfocusedContainerColor = MaterialTheme.colorScheme.surface,
                            focusedBorderColor = MaterialTheme.colorScheme.primary,
                            unfocusedBorderColor = sunderColors.cardBorder
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
                                    imageVector = Icons.Default.Description,
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
                        shape = RoundedCornerShape(16.dp),
                        minLines = 3,
                        colors = OutlinedTextFieldDefaults.colors(
                            focusedContainerColor = MaterialTheme.colorScheme.surface,
                            unfocusedContainerColor = MaterialTheme.colorScheme.surface,
                            focusedBorderColor = MaterialTheme.colorScheme.primary,
                            unfocusedBorderColor = sunderColors.cardBorder
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
                SelectionSheet(
                    title = if (selectionType == SelectionType.ACCOUNT) {
                        stringResource(R.string.select_account)
                    } else {
                        stringResource(R.string.select_category)
                    },
                    onAddClick = {
                        showSheet = false
                        if (selectionType == SelectionType.ACCOUNT) onAddAccount() else onAddCategory()
                    }
                ) {
                    if (selectionType == SelectionType.ACCOUNT) {
                        items(accounts, key = { it.account.id }) { accountWithBalance ->
                            SelectionItem(
                                label = accountWithBalance.account.name,
                                supportingText = formatCurrencyAmount(accountWithBalance.balance),
                                isSelected = accountWithBalance.account.id == entryFormState.accountId,
                                icon = Icons.Default.AccountBalanceWallet,
                                onClick = {
                                    viewModel.updateForm(entryFormState.copy(accountId = accountWithBalance.account.id))
                                    showSheet = false
                                }
                            )
                        }
                    } else {
                        items(categories, key = { it.id }) { category ->
                            SelectionItem(
                                label = category.title,
                                colorDot = parseHexColor(category.color),
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
    onClick: () -> Unit,
    isPlaceholder: Boolean = false,
    badgeColor: Color? = null
) {
    Surface(
        onClick = onClick,
        color = Color.Transparent,
        modifier = Modifier.fillMaxWidth()
    ) {
        Row(
            modifier = Modifier
                .padding(horizontal = 16.dp, vertical = 14.dp)
                .fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(14.dp),
                modifier = Modifier.weight(1f)
            ) {
                Surface(
                    color = badgeColor?.copy(alpha = 0.18f)
                        ?: MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.7f),
                    shape = RoundedCornerShape(12.dp),
                    modifier = Modifier.size(40.dp)
                ) {
                    Box(contentAlignment = Alignment.Center) {
                        if (badgeColor != null) {
                            Box(
                                modifier = Modifier
                                    .size(16.dp)
                                    .clip(CircleShape)
                                    .background(badgeColor)
                            )
                        } else {
                            Icon(
                                imageVector = icon,
                                contentDescription = null,
                                tint = MaterialTheme.colorScheme.primary,
                                modifier = Modifier.size(20.dp)
                            )
                        }
                    }
                }
                Column {
                    Text(
                        text = label,
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Text(
                        text = selectedName,
                        style = MaterialTheme.typography.titleMedium,
                        color = if (isPlaceholder) {
                            MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
                        } else {
                            MaterialTheme.colorScheme.onSurface
                        }
                    )
                }
            }
            Icon(
                imageVector = Icons.Default.ChevronRight,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f)
            )
        }
    }
}
