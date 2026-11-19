package dev.crqch.sunder.ui.categories

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.filled.ReceiptLong
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Description
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material.icons.filled.Title
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
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBarDefaults
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
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import dev.crqch.sunder.R
import dev.crqch.sunder.ui.components.ColorPickerSection
import dev.crqch.sunder.ui.components.ConfirmDeleteDialog
import dev.crqch.sunder.ui.components.EntriesList
import dev.crqch.sunder.ui.components.SunderEmptyState
import dev.crqch.sunder.ui.theme.SunderMotion
import dev.crqch.sunder.ui.theme.sunderColors
import dev.crqch.sunder.utils.formatCurrencyAmount
import dev.crqch.sunder.utils.parseHexColor
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CategoryScreen(
    onNavigateBack: () -> Unit,
    onEntryClick: (String) -> Unit,
    onAddEntry: (categoryId: String) -> Unit,
    viewModel: CategoryViewModel = hiltViewModel(),
) {
    val category by viewModel.category.collectAsState()
    val entries by viewModel.entries.collectAsState()
    val categoryFormState = viewModel.categoryFormState

    var showDeleteDialog by remember { mutableStateOf(false) }
    val snackbarHostState = remember { SnackbarHostState() }
    val scope = rememberCoroutineScope()

    val blockedDeleteMessage = stringResource(R.string.cannot_delete_category_with_entries, entries.size)
    val totalAmount = remember(entries) {
        entries.sumOf { it.entry.amount.toDouble() }.toFloat()
    }

    fun requestDelete() {
        if (entries.isNotEmpty()) {
            scope.launch {
                snackbarHostState.showSnackbar(blockedDeleteMessage)
            }
        } else {
            showDeleteDialog = true
        }
    }

    category?.let { currentCategory ->
        val catColor = remember(currentCategory.color) { parseHexColor(currentCategory.color) }

        Scaffold(
            containerColor = MaterialTheme.colorScheme.background,
            snackbarHost = { SnackbarHost(snackbarHostState) },
            topBar = {
                CenterAlignedTopAppBar(
                    colors = TopAppBarDefaults.centerAlignedTopAppBarColors(
                        containerColor = MaterialTheme.colorScheme.background
                    ),
                    title = {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(8.dp)
                        ) {
                            Box(
                                modifier = Modifier
                                    .size(12.dp)
                                    .clip(CircleShape)
                                    .background(catColor)
                            )
                            Text(
                                text = currentCategory.title,
                                fontWeight = FontWeight.SemiBold,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis
                            )
                        }
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
                                enabled = categoryFormState.isFilled()
                            ) {
                                Icon(
                                    imageVector = Icons.Default.Check,
                                    contentDescription = stringResource(R.string.save),
                                    tint = if (categoryFormState.isFilled()) MaterialTheme.colorScheme.primary
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
                            IconButton(onClick = { requestDelete() }) {
                                Icon(
                                    imageVector = Icons.Default.Delete,
                                    contentDescription = stringResource(R.string.delete_category),
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
                        onClick = { onAddEntry(currentCategory.id) },
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
                label = "CategoryEditTransition",
                modifier = Modifier
                    .fillMaxSize()
                    .padding(padding)
            ) { isEditing ->
                if (isEditing) {
                    Column(
                        modifier = Modifier
                            .fillMaxSize()
                            .verticalScroll(rememberScrollState())
                            .padding(horizontal = 20.dp, vertical = 16.dp),
                        verticalArrangement = Arrangement.spacedBy(18.dp)
                    ) {
                        Surface(
                            modifier = Modifier.fillMaxWidth(),
                            shape = MaterialTheme.shapes.large,
                            color = MaterialTheme.colorScheme.surface,
                            border = BorderStroke(1.dp, MaterialTheme.sunderColors.cardBorder)
                        ) {
                            Column(
                                modifier = Modifier.padding(18.dp),
                                verticalArrangement = Arrangement.spacedBy(16.dp)
                            ) {
                                OutlinedTextField(
                                    value = categoryFormState.title,
                                    onValueChange = { viewModel.updateForm(categoryFormState.copy(title = it)) },
                                    label = { Text(stringResource(R.string.category_title)) },
                                    placeholder = { Text(stringResource(R.string.what_is_this_for)) },
                                    leadingIcon = { Icon(Icons.Default.Title, contentDescription = null) },
                                    modifier = Modifier.fillMaxWidth(),
                                    shape = RoundedCornerShape(14.dp),
                                    singleLine = true,
                                    colors = OutlinedTextFieldDefaults.colors(
                                        focusedBorderColor = MaterialTheme.colorScheme.primary,
                                        unfocusedBorderColor = MaterialTheme.sunderColors.cardBorder
                                    )
                                )

                                OutlinedTextField(
                                    value = categoryFormState.description,
                                    onValueChange = {
                                        viewModel.updateForm(categoryFormState.copy(description = it))
                                    },
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
                                    shape = RoundedCornerShape(14.dp),
                                    minLines = 3,
                                    colors = OutlinedTextFieldDefaults.colors(
                                        focusedBorderColor = MaterialTheme.colorScheme.primary,
                                        unfocusedBorderColor = MaterialTheme.sunderColors.cardBorder
                                    )
                                )

                                ColorPickerSection(
                                    selectedHex = categoryFormState.color,
                                    onColorSelected = {
                                        viewModel.updateForm(categoryFormState.copy(color = it))
                                    }
                                )
                            }
                        }

                        if (viewModel.categoryId != null) {
                            FilledTonalButton(
                                onClick = { requestDelete() },
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
                                Text(stringResource(R.string.delete_category))
                            }
                        }
                    }
                } else {
                    val sunderColors = MaterialTheme.sunderColors
                    val totalColor = when {
                        totalAmount > 0f -> sunderColors.income
                        totalAmount < 0f -> sunderColors.expense
                        else -> MaterialTheme.colorScheme.onSurface
                    }

                    EntriesList(
                        entries = entries,
                        onEntrySelect = onEntryClick,
                        filteredAccountId = null,
                        filteredCategoryId = currentCategory.id,
                        contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 8.dp, bottom = 104.dp),
                        headerContent = {
                            item(key = "category_hero_card") {
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
                                                    colors = listOf(
                                                        catColor.copy(alpha = if (sunderColors.isDark) 0.22f else 0.12f),
                                                        Color.Transparent
                                                    )
                                                )
                                            )
                                            .padding(20.dp)
                                    ) {
                                        Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                                            Row(
                                                verticalAlignment = Alignment.CenterVertically,
                                                horizontalArrangement = Arrangement.spacedBy(12.dp)
                                            ) {
                                                Box(
                                                    modifier = Modifier
                                                        .size(40.dp)
                                                        .clip(CircleShape)
                                                        .background(catColor.copy(alpha = 0.2f)),
                                                    contentAlignment = Alignment.Center
                                                ) {
                                                    Box(
                                                        modifier = Modifier
                                                            .size(18.dp)
                                                            .clip(CircleShape)
                                                            .background(catColor)
                                                    )
                                                }
                                                Column(modifier = Modifier.weight(1f)) {
                                                    Text(
                                                        text = currentCategory.title,
                                                        style = MaterialTheme.typography.titleLarge,
                                                        fontWeight = FontWeight.Bold
                                                    )
                                                    if (currentCategory.description.isNotBlank()) {
                                                        Text(
                                                            text = currentCategory.description,
                                                            style = MaterialTheme.typography.bodyMedium,
                                                            color = MaterialTheme.colorScheme.onSurfaceVariant
                                                        )
                                                    }
                                                }
                                            }

                                            Text(
                                                text = formatCurrencyAmount(totalAmount, showPlusSign = true),
                                                style = MaterialTheme.typography.displaySmall,
                                                fontWeight = FontWeight.Bold,
                                                color = totalColor
                                            )
                                        }
                                    }
                                }
                            }
                            item(key = "category_entries_header") {
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
                                onAction = { onAddEntry(currentCategory.id) },
                                modifier = Modifier.padding(top = 8.dp)
                            )
                        }
                    )
                }
            }
        }
    }

    if (showDeleteDialog) {
        ConfirmDeleteDialog(
            title = stringResource(R.string.delete_category),
            message = stringResource(R.string.confirm_delete_category),
            onConfirm = {
                showDeleteDialog = false
                viewModel.delete { onNavigateBack() }
            },
            onDismiss = { showDeleteDialog = false }
        )
    }
}
