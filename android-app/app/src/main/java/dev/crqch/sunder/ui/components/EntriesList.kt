package dev.crqch.sunder.ui.components

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
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ReceiptLong
import androidx.compose.material.icons.filled.LocationOn
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.crqch.sunder.R
import dev.crqch.sunder.data.local.CurrencyEntity
import dev.crqch.sunder.data.local.EntryWithDetails
import dev.crqch.sunder.ui.theme.sunderColors
import dev.crqch.sunder.utils.CurrencyFormatter
import dev.crqch.sunder.utils.CurrencyUiState
import dev.crqch.sunder.utils.formatDateShort
import dev.crqch.sunder.utils.formatRelativeDay
import dev.crqch.sunder.utils.parseHexColor
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun EntriesList(
    entries: List<EntryWithDetails>,
    onEntrySelect: (String) -> Unit,
    modifier: Modifier = Modifier,
    filteredAccountId: String? = null,
    filteredCategoryId: String? = null,
    groupByDate: Boolean = true,
    selectedIds: Set<String> = emptySet(),
    onEntryLongPress: ((String) -> Unit)? = null,
    currencyState: CurrencyUiState = CurrencyUiState(),
    contentPadding: PaddingValues = PaddingValues(start = 16.dp, end = 16.dp, top = 8.dp, bottom = 96.dp),
    headerContent: (LazyListScope.() -> Unit)? = null,
    emptyContent: (@Composable () -> Unit)? = null
) {
    val groupedEntries = remember(entries, groupByDate) {
        if (!groupByDate) {
            emptyMap()
        } else {
            entries.groupBy {
                Instant.ofEpochMilli(it.entry.date)
                    .atZone(ZoneId.systemDefault())
                    .toLocalDate()
            }
        }
    }

    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = contentPadding,
        verticalArrangement = Arrangement.spacedBy(10.dp)
    ) {
        headerContent?.invoke(this)

        if (entries.isEmpty()) {
            item(key = "empty_entries_state") {
                if (emptyContent != null) {
                    emptyContent()
                } else {
                    SunderEmptyState(
                        icon = Icons.AutoMirrored.Filled.ReceiptLong,
                        title = stringResource(R.string.no_entries),
                        subtitle = stringResource(R.string.no_entries_subtitle),
                        modifier = Modifier.padding(top = 12.dp)
                    )
                }
            }
        } else if (groupByDate) {
            groupedEntries.forEach { (date, dateEntries) ->
                val dayNet = dateEntries.sumOf { item ->
                    currencyState.convertOrSame(
                        item.entry.amount.toDouble(),
                        item.entry.currencyCode,
                        item.entry.date
                    )
                }
                stickyHeader(key = "date_$date") {
                    DateSeparator(
                        date = date,
                        dayNet = dayNet,
                        currency = currencyState.defaultCurrency
                    )
                }

                items(
                    items = dateEntries,
                    key = { it.entry.id }
                ) { entryWithDetails ->
                    val isSelected = selectedIds.contains(entryWithDetails.entry.id)
                    EntryCard(
                        modifier = Modifier.animateItem(),
                        entryWithDetails = entryWithDetails,
                        onClick = { onEntrySelect(entryWithDetails.entry.id) },
                        onLongClick = onEntryLongPress?.let { cb -> { cb(entryWithDetails.entry.id) } },
                        filteredAccountId = filteredAccountId,
                        filteredCategoryId = filteredCategoryId,
                        inSelectionMode = selectedIds.isNotEmpty(),
                        isSelected = isSelected,
                        currencyState = currencyState
                    )
                }
            }
        } else {
            items(
                items = entries,
                key = { it.entry.id }
            ) { entryWithDetails ->
                val isSelected = selectedIds.contains(entryWithDetails.entry.id)
                EntryCard(
                    modifier = Modifier.animateItem(),
                    entryWithDetails = entryWithDetails,
                    onClick = { onEntrySelect(entryWithDetails.entry.id) },
                    onLongClick = onEntryLongPress?.let { cb -> { cb(entryWithDetails.entry.id) } },
                    filteredAccountId = filteredAccountId,
                    filteredCategoryId = filteredCategoryId,
                    inSelectionMode = selectedIds.isNotEmpty(),
                    isSelected = isSelected,
                    showDate = true,
                    currencyState = currencyState
                )
            }
        }
    }
}

@Composable
fun DateSeparator(
    date: LocalDate,
    dayNet: Double,
    currency: CurrencyEntity,
    modifier: Modifier = Modifier
) {
    val label = remember(date) { formatRelativeDay(date) }
    val sunderColors = MaterialTheme.sunderColors
    val netColor = when {
        dayNet > 0.0 -> sunderColors.income
        dayNet < 0.0 -> sunderColors.expense
        else -> MaterialTheme.colorScheme.onSurfaceVariant
    }

    Surface(
        color = MaterialTheme.colorScheme.background.copy(alpha = 0.96f),
        modifier = modifier.fillMaxWidth()
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(top = 8.dp, bottom = 4.dp, start = 4.dp, end = 4.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = label,
                style = MaterialTheme.typography.labelLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                fontWeight = FontWeight.SemiBold
            )
            Text(
                text = CurrencyFormatter.formatSigned(dayNet, currency),
                style = MaterialTheme.typography.labelMedium,
                color = netColor.copy(alpha = 0.85f),
                fontWeight = FontWeight.SemiBold
            )
        }
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun EntryCard(
    entryWithDetails: EntryWithDetails,
    onClick: () -> Unit,
    filteredAccountId: String?,
    filteredCategoryId: String?,
    modifier: Modifier = Modifier,
    onLongClick: (() -> Unit)? = null,
    inSelectionMode: Boolean = false,
    isSelected: Boolean = false,
    showDate: Boolean = false,
    currencyState: CurrencyUiState = CurrencyUiState()
) {
    val entry = entryWithDetails.entry
    val isNegative = entry.amount < 0f
    val sunderColors = MaterialTheme.sunderColors
    val amountColor = if (isNegative) sunderColors.expense else sunderColors.income
    val categoryColor = remember(entryWithDetails.category.color) {
        parseHexColor(entryWithDetails.category.color)
    }
    val nativeCurrency = currencyState.currency(entry.currencyCode)
    val nativeAmount = CurrencyFormatter.formatSigned(entry.amount.toDouble(), nativeCurrency)
    val convertedPreview = currencyState.convertedPreview(
        amount = entry.amount.toDouble(),
        fromCode = entry.currencyCode,
        dateMillis = entry.date
    )
    val showNonDefaultCode = !currencyState.isDefault(entry.currencyCode)

    val containerColor by animateColorAsState(
        targetValue = if (isSelected) {
            MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.35f)
        } else {
            MaterialTheme.colorScheme.surface
        },
        label = "EntryCardBg"
    )

    val borderColor by animateColorAsState(
        targetValue = if (isSelected) {
            MaterialTheme.colorScheme.primary
        } else {
            sunderColors.cardBorder
        },
        label = "EntryCardBorder"
    )

    Surface(
        modifier = modifier
            .fillMaxWidth()
            .clip(MaterialTheme.shapes.medium)
            .combinedClickable(
                onClick = onClick,
                onLongClick = onLongClick
            ),
        shape = MaterialTheme.shapes.medium,
        color = containerColor,
        border = BorderStroke(if (isSelected) 1.5.dp else 1.dp, borderColor),
        tonalElevation = if (isSelected) 2.dp else 0.dp
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 14.dp, vertical = 13.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            AnimatedVisibility(visible = inSelectionMode) {
                Row {
                    SelectionCheckCircle(selected = isSelected)
                    Spacer(modifier = Modifier.width(12.dp))
                }
            }

            // Category color indicator pill
            Box(
                modifier = Modifier
                    .width(5.dp)
                    .height(40.dp)
                    .clip(RoundedCornerShape(4.dp))
                    .background(categoryColor)
            )

            Spacer(modifier = Modifier.width(12.dp))

            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(3.dp)
            ) {
                Text(
                    text = entry.title,
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.SemiBold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    color = MaterialTheme.colorScheme.onSurface
                )

                val showAccount = filteredAccountId != entryWithDetails.account.id
                val showCategory = filteredCategoryId != entry.categoryId

                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(6.dp)
                ) {
                    if (showCategory) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(4.dp)
                        ) {
                            Box(
                                modifier = Modifier
                                    .size(7.dp)
                                    .clip(CircleShape)
                                    .background(categoryColor)
                            )
                            Text(
                                text = entryWithDetails.category.title,
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis
                            )
                        }
                    }

                    if (showCategory && showAccount) {
                        Text(
                            text = "•",
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.outline
                        )
                    }

                    if (showAccount) {
                        Text(
                            text = entryWithDetails.account.name,
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    }
                }

                if (entry.location.isNotBlank() || entry.description.isNotBlank()) {
                    Row(
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(4.dp)
                    ) {
                        if (entry.location.isNotBlank()) {
                            Icon(
                                imageVector = Icons.Default.LocationOn,
                                contentDescription = null,
                                modifier = Modifier.size(12.dp),
                                tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.7f)
                            )
                            Text(
                                text = entry.location,
                                style = MaterialTheme.typography.labelSmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.8f),
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis
                            )
                        } else if (entry.description.isNotBlank()) {
                            Text(
                                text = entry.description,
                                style = MaterialTheme.typography.labelSmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.8f),
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis
                            )
                        }
                    }
                }
            }

            Spacer(modifier = Modifier.width(12.dp))

            Column(horizontalAlignment = Alignment.End) {
                Text(
                    text = nativeAmount,
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold,
                    color = amountColor
                )
                if (convertedPreview != null) {
                    Text(
                        text = stringResource(R.string.approx_amount, convertedPreview),
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                } else if (showNonDefaultCode) {
                    Text(
                        text = nativeCurrency.code,
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
                if (showDate) {
                    Text(
                        text = formatDateShort(entry.date),
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
        }
    }
}

@Composable
fun CurrencyCodeBadge(
    code: String,
    modifier: Modifier = Modifier
) {
    Surface(
        modifier = modifier,
        shape = RoundedCornerShape(6.dp),
        color = MaterialTheme.colorScheme.surfaceContainerHigh,
        border = BorderStroke(1.dp, MaterialTheme.sunderColors.cardBorder)
    ) {
        Text(
            text = code,
            style = MaterialTheme.typography.labelSmall,
            fontWeight = FontWeight.SemiBold,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.padding(horizontal = 5.dp, vertical = 1.dp)
        )
    }
}

@Composable
fun CompressedEntryCard(
    entryWithDetails: EntryWithDetails,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    currencyState: CurrencyUiState = CurrencyUiState()
) {
    EntryCard(
        entryWithDetails = entryWithDetails,
        onClick = onClick,
        filteredAccountId = null,
        filteredCategoryId = null,
        modifier = modifier,
        showDate = true,
        currencyState = currencyState
    )
}
