package dev.crqch.sunder.ui.categories

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
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Category
import androidx.compose.material.icons.filled.ChevronRight
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import dev.crqch.sunder.R
import dev.crqch.sunder.ui.components.ConfirmDeleteDialog
import dev.crqch.sunder.ui.components.SelectionCheckCircle
import dev.crqch.sunder.ui.components.SelectionTopBar
import dev.crqch.sunder.ui.components.SunderEmptyState
import dev.crqch.sunder.ui.components.SunderSearchBar
import dev.crqch.sunder.ui.theme.sunderColors
import dev.crqch.sunder.utils.formatCurrencyAmount
import dev.crqch.sunder.utils.parseHexColor
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class, ExperimentalFoundationApi::class)
@Composable
fun CategoriesScreen(
    onAddCategory: () -> Unit,
    viewModel: CategoriesViewModel = hiltViewModel(),
    visitCategory: (categoryId: String) -> Unit,
) {
    val allCategories by viewModel.allCategoriesWithStats.collectAsState()
    val categories by viewModel.categories.collectAsState()
    val searchQuery by viewModel.searchQuery.collectAsState()

    var selectedIds by remember { mutableStateOf<Set<String>>(emptySet()) }
    var showDeleteDialog by remember { mutableStateOf(false) }

    val snackbarHostState = remember { SnackbarHostState() }
    val scope = rememberCoroutineScope()
    val allSelected = categories.isNotEmpty() && selectedIds.size == categories.size

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
                        text = stringResource(R.string.categories),
                        style = MaterialTheme.typography.headlineSmall,
                        fontWeight = FontWeight.Bold
                    )
                }
            )
        },
        floatingActionButton = {
            if (selectedIds.isEmpty()) {
                FloatingActionButton(
                    onClick = onAddCategory,
                    shape = RoundedCornerShape(18.dp),
                    containerColor = MaterialTheme.colorScheme.primary,
                    contentColor = MaterialTheme.colorScheme.onPrimary
                ) {
                    Icon(
                        imageVector = Icons.Default.Add,
                        contentDescription = stringResource(R.string.add_category)
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
                label = "CategoriesHeaderMode",
                modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp)
            ) { inSelectionMode ->
                if (inSelectionMode) {
                    SelectionTopBar(
                        selectedCount = selectedIds.size,
                        allSelected = allSelected,
                        onClearSelection = { selectedIds = emptySet() },
                        onToggleSelectAll = {
                            selectedIds = if (allSelected) emptySet()
                            else categories.map { it.category.id }.toSet()
                        },
                        onDeleteSelected = { showDeleteDialog = true }
                    )
                } else if (allCategories.isNotEmpty()) {
                    SunderSearchBar(
                        query = searchQuery,
                        onQueryChange = { viewModel.updateSearchQuery(it) },
                        placeholder = stringResource(R.string.search_categories)
                    )
                }
            }

            if (allCategories.isEmpty()) {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(24.dp),
                    contentAlignment = Alignment.Center
                ) {
                    SunderEmptyState(
                        icon = Icons.Default.Category,
                        title = stringResource(R.string.no_categories),
                        subtitle = stringResource(R.string.create_first_category),
                        actionLabel = stringResource(R.string.add_category),
                        onAction = onAddCategory
                    )
                }
            } else if (categories.isEmpty()) {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(24.dp),
                    contentAlignment = Alignment.TopCenter
                ) {
                    SunderEmptyState(
                        icon = Icons.Default.Category,
                        title = stringResource(R.string.no_matching_results),
                        subtitle = stringResource(R.string.search_categories)
                    )
                }
            } else {
                LazyColumn(
                    modifier = Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 10.dp, bottom = 96.dp),
                    verticalArrangement = Arrangement.spacedBy(10.dp)
                ) {
                    items(categories, key = { it.category.id }) { item ->
                        val category = item.category
                        val catColor = remember(category.color) { parseHexColor(category.color) }
                        val isSelected = selectedIds.contains(category.id)
                        val sunderColors = MaterialTheme.sunderColors

                        val cardBg by animateColorAsState(
                            targetValue = if (isSelected) {
                                MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.35f)
                            } else {
                                MaterialTheme.colorScheme.surface
                            },
                            label = "CatCardBg"
                        )
                        val cardBorder by animateColorAsState(
                            targetValue = if (isSelected) {
                                MaterialTheme.colorScheme.primary
                            } else {
                                sunderColors.cardBorder
                            },
                            label = "CatCardBorder"
                        )

                        Surface(
                            modifier = Modifier
                                .fillMaxWidth()
                                .animateItem()
                                .clip(MaterialTheme.shapes.large)
                                .combinedClickable(
                                    onClick = {
                                        if (selectedIds.isNotEmpty()) {
                                            selectedIds = if (isSelected) selectedIds - category.id
                                            else selectedIds + category.id
                                        } else {
                                            visitCategory(category.id)
                                        }
                                    },
                                    onLongClick = {
                                        selectedIds = if (isSelected) selectedIds - category.id
                                        else selectedIds + category.id
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

                                Box(
                                    modifier = Modifier
                                        .size(42.dp)
                                        .clip(CircleShape)
                                        .background(catColor.copy(alpha = 0.18f)),
                                    contentAlignment = Alignment.Center
                                ) {
                                    Box(
                                        modifier = Modifier
                                            .size(18.dp)
                                            .clip(CircleShape)
                                            .background(catColor)
                                    )
                                }

                                Spacer(modifier = Modifier.width(14.dp))

                                Column(
                                    modifier = Modifier.weight(1f),
                                    verticalArrangement = Arrangement.spacedBy(2.dp)
                                ) {
                                    Text(
                                        text = category.title,
                                        style = MaterialTheme.typography.titleMedium,
                                        fontWeight = FontWeight.SemiBold,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis
                                    )
                                    if (category.description.isNotBlank()) {
                                        Text(
                                            text = category.description,
                                            style = MaterialTheme.typography.bodySmall,
                                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis
                                        )
                                    } else {
                                        Text(
                                            text = "${item.entryCount} ${stringResource(R.string.entries).lowercase()}",
                                            style = MaterialTheme.typography.labelSmall,
                                            color = MaterialTheme.colorScheme.onSurfaceVariant
                                        )
                                    }
                                }

                                Spacer(modifier = Modifier.width(10.dp))

                                Column(horizontalAlignment = Alignment.End) {
                                    if (item.entryCount > 0) {
                                        val amountColor = when {
                                            item.totalAmount > 0f -> sunderColors.income
                                            item.totalAmount < 0f -> sunderColors.expense
                                            else -> MaterialTheme.colorScheme.onSurfaceVariant
                                        }
                                        Text(
                                            text = formatCurrencyAmount(item.totalAmount, showPlusSign = true),
                                            style = MaterialTheme.typography.labelLarge,
                                            fontWeight = FontWeight.Bold,
                                            color = amountColor
                                        )
                                        Text(
                                            text = "${item.entryCount}",
                                            style = MaterialTheme.typography.labelSmall,
                                            color = MaterialTheme.colorScheme.onSurfaceVariant
                                        )
                                    }
                                }

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

    if (showDeleteDialog) {
        ConfirmDeleteDialog(
            title = stringResource(R.string.delete_selected_title),
            message = stringResource(R.string.delete_selected_message, selectedIds.size),
            onConfirm = {
                val idsToDelete = selectedIds.toList()
                showDeleteDialog = false
                selectedIds = emptySet()
                viewModel.deleteCategories(
                    ids = idsToDelete,
                    onBlockedByEntries = { blockedCount ->
                        scope.launch {
                            snackbarHostState.showSnackbar(
                                "$blockedCount category(ies) with active entries were kept."
                            )
                        }
                    }
                )
            },
            onDismiss = { showDeleteDialog = false }
        )
    }
}