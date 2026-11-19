package dev.crqch.sunder.ui.common

import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
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
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowForward
import androidx.compose.material.icons.automirrored.filled.ReceiptLong
import androidx.compose.material.icons.filled.AccountBalanceWallet
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.ArrowDownward
import androidx.compose.material.icons.filled.ArrowUpward
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.filled.Sync
import androidx.compose.material3.Badge
import androidx.compose.material3.BadgedBox
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import dev.crqch.sunder.R
import dev.crqch.sunder.data.local.CurrencyEntity
import dev.crqch.sunder.ui.components.EntryCard
import dev.crqch.sunder.ui.components.SunderEmptyState
import dev.crqch.sunder.ui.theme.SunderMotion
import dev.crqch.sunder.ui.theme.sunderColors
import dev.crqch.sunder.utils.CurrencyFormatter
import dev.crqch.sunder.utils.parseHexColor
import kotlin.math.roundToInt

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HomeScreen(
    onNavigateSettings: () -> Unit,
    onNavigateEntries: () -> Unit,
    onAddEntry: () -> Unit,
    onAddAccount: () -> Unit,
    onEntryClick: (String) -> Unit,
    onCategoryClick: (String) -> Unit,
    viewModel: HomeViewModel = hiltViewModel()
) {
    val currentUser by viewModel.currentUser.collectAsState()
    val state by viewModel.dashboardState.collectAsState()
    val currencyState by viewModel.currencyState.collectAsState()
    val isSyncing by viewModel.isSyncing.collectAsState()
    val unsyncedCount by viewModel.unsyncedCount.collectAsState()
    val syncMessage by viewModel.syncMessage.collectAsState()

    val snackbarHostState = remember { SnackbarHostState() }
    val syncSuccessStr = stringResource(R.string.sync_success)
    val syncFailStr = stringResource(R.string.sync_failed)

    LaunchedEffect(syncMessage) {
        syncMessage?.let {
            snackbarHostState.showSnackbar(it)
            viewModel.clearSyncMessage()
        }
    }

    val infiniteTransition = rememberInfiniteTransition(label = "SyncSpin")
    val spinAngle by infiniteTransition.animateFloat(
        initialValue = 0f,
        targetValue = 360f,
        animationSpec = infiniteRepeatable(
            animation = tween(900, easing = LinearEasing),
            repeatMode = RepeatMode.Restart
        ),
        label = "SyncSpinAngle"
    )

    Scaffold(
        snackbarHost = { SnackbarHost(snackbarHostState) },
        containerColor = MaterialTheme.colorScheme.background,
        topBar = {
            TopAppBar(
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.background
                ),
                title = {
                    Column {
                        Text(
                            text = stringResource(
                                R.string.welcome,
                                currentUser?.username?.takeIf { it.isNotBlank() } ?: "User"
                            ),
                            style = MaterialTheme.typography.headlineSmall,
                            fontWeight = FontWeight.Bold,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    }
                },
                actions = {
                    IconButton(
                        onClick = { viewModel.triggerManualSync(syncSuccessStr, syncFailStr) },
                        enabled = !isSyncing
                    ) {
                        BadgedBox(
                            badge = {
                                if (unsyncedCount > 0 && !isSyncing) {
                                    Badge(
                                        containerColor = MaterialTheme.colorScheme.primary
                                    ) {
                                        Text(
                                            text = if (unsyncedCount > 9) "9+" else unsyncedCount.toString(),
                                            style = MaterialTheme.typography.labelSmall
                                        )
                                    }
                                }
                            }
                        ) {
                            Icon(
                                imageVector = Icons.Default.Sync,
                                contentDescription = stringResource(R.string.sync_now),
                                modifier = Modifier.rotate(if (isSyncing) spinAngle else 0f)
                            )
                        }
                    }
                    IconButton(onClick = onNavigateSettings) {
                        Icon(
                            imageVector = Icons.Filled.Settings,
                            contentDescription = stringResource(R.string.settings)
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
            contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 8.dp, bottom = 32.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp)
        ) {
            // Hero Balance Card + Income / Expense summary
            item(key = "balance_hero") {
                DashboardBalanceHero(
                    totalBalance = state.totalBalance,
                    totalIncome = state.totalIncome,
                    totalExpenses = state.totalExpenses,
                    currency = currencyState.defaultCurrency
                )
            }

            // Quick Actions Row (Add Entry + Add Account)
            item(key = "quick_actions") {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    Button(
                        onClick = onAddEntry,
                        modifier = Modifier
                            .weight(1f)
                            .height(50.dp),
                        shape = RoundedCornerShape(16.dp)
                    ) {
                        Icon(
                            imageVector = Icons.Default.Add,
                            contentDescription = null,
                            modifier = Modifier.size(ButtonDefaults.IconSize)
                        )
                        Spacer(modifier = Modifier.width(ButtonDefaults.IconSpacing))
                        Text(
                            text = stringResource(R.string.add_entry),
                            style = MaterialTheme.typography.labelLarge
                        )
                    }

                    FilledTonalButton(
                        onClick = onAddAccount,
                        modifier = Modifier
                            .weight(1f)
                            .height(50.dp),
                        shape = RoundedCornerShape(16.dp)
                    ) {
                        Icon(
                            imageVector = Icons.Default.AccountBalanceWallet,
                            contentDescription = null,
                            modifier = Modifier.size(ButtonDefaults.IconSize)
                        )
                        Spacer(modifier = Modifier.width(ButtonDefaults.IconSpacing))
                        Text(
                            text = stringResource(R.string.add_account),
                            style = MaterialTheme.typography.labelLarge
                        )
                    }
                }
            }

            // Top Spending Categories Card
            if (state.topCategories.isNotEmpty()) {
                item(key = "top_spending") {
                    TopSpendingCard(
                        items = state.topCategories,
                        currency = currencyState.defaultCurrency,
                        onCategoryClick = onCategoryClick
                    )
                }
            }

            // Recent Transactions Header
            item(key = "recent_header") {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = 4.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = stringResource(R.string.recent_transactions),
                        style = MaterialTheme.typography.titleLarge,
                        fontWeight = FontWeight.Bold
                    )
                    if (state.totalEntryCount > 0) {
                        TextButton(onClick = onNavigateEntries) {
                            Text(stringResource(R.string.view_all))
                            Spacer(modifier = Modifier.width(4.dp))
                            Icon(
                                imageVector = Icons.AutoMirrored.Filled.ArrowForward,
                                contentDescription = null,
                                modifier = Modifier.size(16.dp)
                            )
                        }
                    }
                }
            }

            if (state.recentEntries.isEmpty()) {
                item(key = "recent_empty") {
                    SunderEmptyState(
                        icon = Icons.AutoMirrored.Filled.ReceiptLong,
                        title = stringResource(R.string.no_entries),
                        subtitle = stringResource(R.string.no_entries_subtitle),
                        actionLabel = stringResource(R.string.add_entry),
                        onAction = onAddEntry
                    )
                }
            } else {
                items(
                    items = state.recentEntries,
                    key = { it.entry.id }
                ) { entryWithDetails ->
                    EntryCard(
                        modifier = Modifier.animateItem(),
                        entryWithDetails = entryWithDetails,
                        onClick = { onEntryClick(entryWithDetails.entry.id) },
                        filteredAccountId = null,
                        filteredCategoryId = null,
                        showDate = true,
                        currencyState = currencyState
                    )
                }
            }
        }
    }
}

@Composable
private fun DashboardBalanceHero(
    totalBalance: Float,
    totalIncome: Float,
    totalExpenses: Float,
    currency: CurrencyEntity
) {
    val sunderColors = MaterialTheme.sunderColors
    val balanceColor = if (totalBalance >= 0f) sunderColors.income else sunderColors.expense
    val accentGlow = if (totalBalance >= 0f) {
        MaterialTheme.colorScheme.primary.copy(alpha = if (sunderColors.isDark) 0.22f else 0.10f)
    } else {
        sunderColors.expense.copy(alpha = if (sunderColors.isDark) 0.22f else 0.10f)
    }

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
                            accentGlow,
                            Color.Transparent
                        )
                    )
                )
                .padding(20.dp)
        ) {
            Column(
                modifier = Modifier.fillMaxWidth(),
                verticalArrangement = Arrangement.spacedBy(18.dp)
            ) {
                Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    Text(
                        text = stringResource(R.string.total_balance),
                        style = MaterialTheme.typography.labelLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Text(
                        text = CurrencyFormatter.format(totalBalance.toDouble(), currency),
                        style = MaterialTheme.typography.displaySmall,
                        fontWeight = FontWeight.Bold,
                        color = balanceColor,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                }

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.spacedBy(12.dp)
                ) {
                    SummaryMetricTile(
                        label = stringResource(R.string.total_income),
                        value = CurrencyFormatter.format(
                            totalIncome.toDouble(),
                            currency,
                            showPlusSign = totalIncome > 0f
                        ),
                        accentColor = sunderColors.income,
                        containerColor = sunderColors.incomeContainer.copy(alpha = if (sunderColors.isDark) 0.45f else 0.6f),
                        icon = Icons.Default.ArrowDownward,
                        modifier = Modifier.weight(1f)
                    )
                    SummaryMetricTile(
                        label = stringResource(R.string.total_expenses),
                        value = CurrencyFormatter.format(
                            -totalExpenses.toDouble(),
                            currency,
                            showMinusSign = totalExpenses > 0f
                        ),
                        accentColor = sunderColors.expense,
                        containerColor = sunderColors.expenseContainer.copy(alpha = if (sunderColors.isDark) 0.45f else 0.6f),
                        icon = Icons.Default.ArrowUpward,
                        modifier = Modifier.weight(1f)
                    )
                }
            }
        }
    }
}

@Composable
private fun SummaryMetricTile(
    label: String,
    value: String,
    accentColor: Color,
    containerColor: Color,
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    modifier: Modifier = Modifier
) {
    Surface(
        modifier = modifier,
        shape = MaterialTheme.shapes.medium,
        color = MaterialTheme.colorScheme.surfaceContainerLow,
        border = BorderStroke(1.dp, MaterialTheme.sunderColors.cardBorder.copy(alpha = 0.6f))
    ) {
        Row(
            modifier = Modifier.padding(12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(10.dp)
        ) {
            Surface(
                modifier = Modifier.size(34.dp),
                shape = CircleShape,
                color = containerColor
            ) {
                Box(contentAlignment = Alignment.Center) {
                    Icon(
                        imageVector = icon,
                        contentDescription = null,
                        tint = accentColor,
                        modifier = Modifier.size(18.dp)
                    )
                }
            }
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    text = label,
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
                Text(
                    text = value,
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold,
                    color = accentColor,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )
            }
        }
    }
}

@Composable
private fun TopSpendingCard(
    items: List<CategorySpendingItem>,
    currency: CurrencyEntity,
    onCategoryClick: (String) -> Unit
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
            verticalArrangement = Arrangement.spacedBy(14.dp)
        ) {
            Text(
                text = stringResource(R.string.top_spending),
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold
            )

            items.forEach { item ->
                val catColor = remember(item.category.color) {
                    parseHexColor(item.category.color)
                }
                val animatedShare by animateFloatAsState(
                    targetValue = item.share,
                    animationSpec = SunderMotion.enterTween(550),
                    label = "CategoryShare_${item.category.id}"
                )

                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(10.dp))
                        .clickable { onCategoryClick(item.category.id) }
                        .padding(vertical = 4.dp),
                    verticalArrangement = Arrangement.spacedBy(6.dp)
                ) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(8.dp),
                            modifier = Modifier.weight(1f)
                        ) {
                            Box(
                                modifier = Modifier
                                    .size(10.dp)
                                    .clip(CircleShape)
                                    .background(catColor)
                            )
                            Text(
                                text = item.category.title,
                                style = MaterialTheme.typography.bodyMedium,
                                fontWeight = FontWeight.Medium,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis
                            )
                        }
                        Spacer(modifier = Modifier.width(8.dp))
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(6.dp)
                        ) {
                            Text(
                                text = CurrencyFormatter.formatAbsolute(item.amount.toDouble(), currency),
                                style = MaterialTheme.typography.labelLarge,
                                fontWeight = FontWeight.SemiBold,
                                color = MaterialTheme.colorScheme.onSurface
                            )
                            Text(
                                text = "${(item.share * 100f).roundToInt()}%",
                                style = MaterialTheme.typography.labelSmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                    }

                    LinearProgressIndicator(
                        progress = { animatedShare },
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(7.dp)
                            .clip(RoundedCornerShape(4.dp)),
                        color = catColor,
                        trackColor = MaterialTheme.colorScheme.surfaceContainerHigh,
                        strokeCap = StrokeCap.Round
                    )
                }
            }
        }
    }
}