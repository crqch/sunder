package dev.crqch.sunder.ui.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.Icon
import androidx.compose.material3.ListItem
import androidx.compose.material3.ListItemDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import dev.crqch.sunder.R
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

@Composable
fun SelectionSheet(
    title: String,
    onAddClick: () -> Unit,
    addLabel: String = stringResource(R.string.add_new),
    content: LazyListScope.() -> Unit
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .padding(bottom = 32.dp)
    ) {
        Text(
            text = title,
            style = MaterialTheme.typography.titleLarge,
            modifier = Modifier.padding(horizontal = 24.dp, vertical = 16.dp)
        )
        LazyColumn(modifier = Modifier.fillMaxWidth()) {
            item {
                SelectionItem(
                    label = addLabel,
                    icon = Icons.Default.Add,
                    onClick = onAddClick,
                    isHighlight = true
                )
            }
            content()
        }
    }
}

@Composable
fun SelectionItem(
    label: String,
    isSelected: Boolean = false,
    icon: ImageVector? = null,
    onClick: () -> Unit,
    isHighlight: Boolean = false
) {
    val containerColor = when {
        isSelected -> MaterialTheme.colorScheme.primaryContainer
        isHighlight -> MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.15f)
        else -> Color.Transparent
    }

    val contentColor = when {
        isSelected -> MaterialTheme.colorScheme.onPrimaryContainer
        isHighlight -> MaterialTheme.colorScheme.primary
        else -> MaterialTheme.colorScheme.onSurface
    }

    ListItem(
        headlineContent = {
            Text(
                text = label,
                style = MaterialTheme.typography.titleMedium,
                fontWeight = if (isSelected || isHighlight) FontWeight.Bold else FontWeight.Normal,
                color = contentColor
            )
        },
        leadingContent = icon?.let {
            {
                Icon(
                    it,
                    contentDescription = null,
                    tint = if (isSelected) contentColor else if (isHighlight) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        },
        trailingContent = if (isSelected) {
            {
                Icon(
                    Icons.Default.Check,
                    contentDescription = stringResource(R.string.selected),
                    tint = contentColor
                )
            }
        } else null,
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .padding(horizontal = 12.dp, vertical = 2.dp),
        colors = ListItemDefaults.colors(
            containerColor = containerColor
        )
    )
}
