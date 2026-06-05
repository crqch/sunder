package dev.crqch.sunder.ui.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Category
import androidx.compose.material.icons.filled.Person
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.crqch.sunder.data.local.EntryWithDetails
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import kotlin.math.abs

@Composable
fun EntriesList(
    entries: List<EntryWithDetails>,
    onEntrySelect: (String) -> Unit,
    modifier: Modifier = Modifier,
    filteredAccountId: String? = null,
    filteredCategoryId: String? = null
) {
    val groupedEntries = remember(entries) {
        entries.groupBy {
            Instant.ofEpochMilli(it.entry.date)
                .atZone(ZoneId.systemDefault())
                .toLocalDate()
        }.toSortedMap(compareByDescending { it })
    }

    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp)
    ) {

        groupedEntries.forEach { (date, dateEntries) ->
            stickyHeader(key = date.toString()) {
                DateSeparator(date)
            }

            items(
                items = dateEntries,
                key = { it.entry.id }
            ) { entryWithDetails ->
                EntryCard(
                    modifier = Modifier.animateItem(),
                    entryWithDetails = entryWithDetails,
                    onClick = { onEntrySelect(entryWithDetails.entry.id) },
                    filteredAccountId = filteredAccountId,
                    filteredCategoryId = filteredCategoryId
                )
            }
        }


    }
}

@Composable
fun DateSeparator(date: LocalDate, modifier: Modifier = Modifier) {
    val formatter = remember { DateTimeFormatter.ofPattern("dd.MM.yyyy") }
    Surface(
        color = MaterialTheme.colorScheme.background,
        modifier = modifier.fillMaxWidth()
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(vertical = 8.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            HorizontalDivider(modifier = Modifier.weight(1f))
            Text(
                text = date.format(formatter),
                modifier = Modifier.padding(horizontal = 16.dp),
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.outline,
                fontWeight = FontWeight.Bold
            )
            HorizontalDivider(modifier = Modifier.weight(1f))
        }
    }
}

@Composable
fun EntryCard(
    entryWithDetails: EntryWithDetails,
    onClick: () -> Unit,
    filteredAccountId: String?,
    filteredCategoryId: String?,
    modifier: Modifier = Modifier
) {
    val entry = entryWithDetails.entry
    val isNegative = entry.amount < 0
    val amountPrefix = if (isNegative) "-" else "+"
    val amountColor =
        if (isNegative) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.primary
    val hourFormatter = remember { DateTimeFormatter.ofPattern("HH:mm") }

    ElevatedCard(
        modifier = modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Column(
                modifier = Modifier.weight(1f),
                verticalArrangement = Arrangement.spacedBy(4.dp)
            ) {
                Text(
                    text = Instant.ofEpochMilli(entry.date)
                        .atZone(ZoneId.systemDefault())
                        .format(hourFormatter),
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.outline
                )
                Text(
                    text = entry.title,
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )

                if (entry.description.isNotBlank()) {
                    Text(
                        text = entry.description,
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 2,
                        overflow = TextOverflow.Ellipsis
                    )
                }

                val showAccount = filteredAccountId != entryWithDetails.account.id
                val showCategory = filteredCategoryId != entry.categoryId

                AnimatedVisibility(
                    visible = showAccount || showCategory
                ) {
                    Column {
                        Spacer(modifier = Modifier.height(4.dp))
                        Row(
                            horizontalArrangement = Arrangement.spacedBy(8.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            AnimatedVisibility(
                                visible = showAccount
                            ) {
                                InfoTag(
                                    icon = Icons.Filled.Person,
                                    text = entryWithDetails.account.name
                                )
                            }

                            AnimatedVisibility(
                                visible = showCategory
                            ) {
                                InfoTag(
                                    icon = Icons.Filled.Category,
                                    text = entryWithDetails.category.title,
                                    isPrimary = true
                                )
                            }
                        }
                    }
                }
            }

            Spacer(modifier = Modifier.width(16.dp))

            Text(
                text = "$amountPrefix${abs(entry.amount)}",
                style = MaterialTheme.typography.titleLarge,
                fontWeight = FontWeight.Bold,
                color = amountColor
            )
        }
    }
}


@Composable
fun CompressedEntryCard(
    entryWithDetails: EntryWithDetails,
    onClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    val entry = entryWithDetails.entry
    val isNegative = entry.amount < 0
    val amountPrefix = if (isNegative) "-" else "+"
    val amountColor =
        if (isNegative) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.primary
    val dateFormatter = remember { DateTimeFormatter.ofPattern("dd.MM.yyyy HH:mm") }


    Row(
        modifier = modifier
            .clickable(onClick = onClick)
            .fillMaxWidth()
            .padding(4.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(
            modifier = Modifier.weight(1f),
            verticalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            Text(
                text = Instant.ofEpochMilli(entry.date)
                    .atZone(ZoneId.systemDefault())
                    .format(dateFormatter),
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.outline
            )
            Text(
                text = entry.title,
                style = MaterialTheme.typography.bodyMedium,
                fontWeight = FontWeight.Bold,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )

            Row(
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                InfoTag(
                    icon = Icons.Filled.Person,
                    text = entryWithDetails.account.name
                )
                InfoTag(
                    icon = Icons.Filled.Category,
                    text = entryWithDetails.category.title,
                    isPrimary = true
                )
            }
        }

        Spacer(modifier = Modifier.width(16.dp))

        Text(
            text = "$amountPrefix${abs(entry.amount)}",
            style = MaterialTheme.typography.titleMedium,
            fontWeight = FontWeight.Bold,
            color = amountColor
        )
    }
}

@Composable
private fun InfoTag(
    icon: ImageVector,
    text: String,
    isPrimary: Boolean = false
) {
    val containerColor =
        if (isPrimary) MaterialTheme.colorScheme.primaryContainer else MaterialTheme.colorScheme.surfaceVariant
    val contentColor =
        if (isPrimary) MaterialTheme.colorScheme.onPrimaryContainer else MaterialTheme.colorScheme.onSurfaceVariant

    Surface(
        shape = RoundedCornerShape(8.dp),
        color = containerColor,
        contentColor = contentColor
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                modifier = Modifier.size(14.dp)
            )
            Text(
                text = text,
                style = MaterialTheme.typography.labelSmall,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis
            )
        }
    }
}