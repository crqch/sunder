package dev.crqch.sunder.ui

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ReceiptLong
import androidx.compose.material.icons.automirrored.outlined.ReceiptLong
import androidx.compose.material.icons.filled.AccountBalanceWallet
import androidx.compose.material.icons.filled.Category
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.outlined.AccountBalanceWallet
import androidx.compose.material.icons.outlined.Category
import androidx.compose.material.icons.outlined.Home
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarDefaults
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.navigation.NavDestination.Companion.hasRoute
import androidx.navigation.NavDestination.Companion.hierarchy
import androidx.navigation.NavGraph.Companion.findStartDestination
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import dev.crqch.sunder.R
import dev.crqch.sunder.ui.accounts.AccountScreen
import dev.crqch.sunder.ui.accounts.AccountsScreen
import dev.crqch.sunder.ui.accounts.CreateAccountScreen
import dev.crqch.sunder.ui.categories.CategoriesScreen
import dev.crqch.sunder.ui.categories.CategoryScreen
import dev.crqch.sunder.ui.categories.CreateCategoryScreen
import dev.crqch.sunder.ui.common.HomeScreen
import dev.crqch.sunder.ui.common.SettingsScreen
import dev.crqch.sunder.ui.entries.CreateEntryScreen
import dev.crqch.sunder.ui.entries.EntriesScreen
import dev.crqch.sunder.ui.entries.EntryScreen
import dev.crqch.sunder.ui.theme.SunderMotion
import kotlinx.serialization.Serializable

sealed interface SubRoute {
    @Serializable
    data class CreateEntry(val accountId: String?, val categoryId: String?) : SubRoute

    @Serializable
    object CreateCategory : SubRoute

    @Serializable
    object CreateAccount : SubRoute

    @Serializable
    data class Entry(val entryId: String) : SubRoute

    @Serializable
    data class Category(val categoryId: String) : SubRoute

    @Serializable
    data class Account(val accountId: String) : SubRoute

    @Serializable
    object Settings : SubRoute
}

sealed class TopLevelDestination {
    abstract val labelRes: Int
    abstract val selectedIcon: ImageVector
    abstract val unselectedIcon: ImageVector
    abstract val contentDescriptionRes: Int
    abstract val order: Int

    @Serializable
    object Home : TopLevelDestination() {
        override val labelRes = R.string.home
        override val selectedIcon = Icons.Filled.Home
        override val unselectedIcon = Icons.Outlined.Home
        override val contentDescriptionRes = R.string.home
        override val order = 0
    }

    @Serializable
    object Entries : TopLevelDestination() {
        override val labelRes = R.string.entries
        override val selectedIcon = Icons.AutoMirrored.Filled.ReceiptLong
        override val unselectedIcon = Icons.AutoMirrored.Outlined.ReceiptLong
        override val contentDescriptionRes = R.string.entries
        override val order = 1
    }

    @Serializable
    object Accounts : TopLevelDestination() {
        override val labelRes = R.string.accounts
        override val selectedIcon = Icons.Filled.AccountBalanceWallet
        override val unselectedIcon = Icons.Outlined.AccountBalanceWallet
        override val contentDescriptionRes = R.string.accounts
        override val order = 2
    }

    @Serializable
    object Categories : TopLevelDestination() {
        override val labelRes = R.string.categories
        override val selectedIcon = Icons.Filled.Category
        override val unselectedIcon = Icons.Outlined.Category
        override val contentDescriptionRes = R.string.categories
        override val order = 3
    }

    companion object {
        val entries by lazy { listOf(Home, Entries, Accounts, Categories) }

        fun fromRouteClass(routeClass: String?): TopLevelDestination? {
            return entries.find { routeClass?.contains(it::class.qualifiedName ?: "") == true }
        }
    }
}

@Composable
fun AppNavHost(
    navController: NavHostController,
    startDestination: TopLevelDestination,
    modifier: Modifier = Modifier
) {
    NavHost(
        navController = navController,
        startDestination = startDestination,
        modifier = modifier,
        enterTransition = {
            val initial = TopLevelDestination.fromRouteClass(initialState.destination.route)
            val target = TopLevelDestination.fromRouteClass(targetState.destination.route)
            if (initial != null && target != null) {
                val isLeft = target.order < initial.order
                fadeIn(SunderMotion.enterTween(SunderMotion.DURATION_SHORT)) +
                    slideInHorizontally(
                        animationSpec = SunderMotion.offsetTween(SunderMotion.DURATION_MEDIUM),
                        initialOffsetX = { if (isLeft) -it / 8 else it / 8 }
                    )
            } else {
                fadeIn(SunderMotion.enterTween()) +
                    slideInHorizontally(
                        animationSpec = SunderMotion.offsetTween(),
                        initialOffsetX = { it / 4 }
                    )
            }
        },
        exitTransition = {
            val initial = TopLevelDestination.fromRouteClass(initialState.destination.route)
            val target = TopLevelDestination.fromRouteClass(targetState.destination.route)
            if (initial != null && target != null) {
                val isLeft = target.order < initial.order
                fadeOut(SunderMotion.exitTween()) +
                    slideOutHorizontally(
                        animationSpec = SunderMotion.offsetTween(SunderMotion.DURATION_MEDIUM),
                        targetOffsetX = { if (isLeft) it / 8 else -it / 8 }
                    )
            } else {
                fadeOut(SunderMotion.exitTween()) +
                    slideOutHorizontally(
                        animationSpec = SunderMotion.offsetTween(),
                        targetOffsetX = { -it / 8 }
                    )
            }
        },
        popEnterTransition = {
            val initial = TopLevelDestination.fromRouteClass(initialState.destination.route)
            val target = TopLevelDestination.fromRouteClass(targetState.destination.route)
            if (initial != null && target != null) {
                val isLeft = target.order < initial.order
                fadeIn(SunderMotion.enterTween(SunderMotion.DURATION_SHORT)) +
                    slideInHorizontally(
                        animationSpec = SunderMotion.offsetTween(SunderMotion.DURATION_MEDIUM),
                        initialOffsetX = { if (isLeft) -it / 8 else it / 8 }
                    )
            } else {
                fadeIn(SunderMotion.enterTween()) +
                    slideInHorizontally(
                        animationSpec = SunderMotion.offsetTween(),
                        initialOffsetX = { -it / 8 }
                    )
            }
        },
        popExitTransition = {
            val initial = TopLevelDestination.fromRouteClass(initialState.destination.route)
            val target = TopLevelDestination.fromRouteClass(targetState.destination.route)
            if (initial != null && target != null) {
                val isLeft = target.order < initial.order
                fadeOut(SunderMotion.exitTween()) +
                    slideOutHorizontally(
                        animationSpec = SunderMotion.offsetTween(SunderMotion.DURATION_MEDIUM),
                        targetOffsetX = { if (isLeft) it / 8 else -it / 8 }
                    )
            } else {
                fadeOut(SunderMotion.exitTween()) +
                    slideOutHorizontally(
                        animationSpec = SunderMotion.offsetTween(),
                        targetOffsetX = { it / 4 }
                    )
            }
        }
    ) {
        composable<TopLevelDestination.Home> {
            HomeScreen(
                onNavigateSettings = {
                    navController.navigate(SubRoute.Settings)
                },
                onNavigateEntries = {
                    navController.navigate(TopLevelDestination.Entries) {
                        popUpTo(navController.graph.findStartDestination().id) {
                            saveState = true
                        }
                        launchSingleTop = true
                        restoreState = true
                    }
                },
                onAddEntry = {
                    navController.navigate(
                        SubRoute.CreateEntry(accountId = null, categoryId = null)
                    )
                },
                onAddAccount = {
                    navController.navigate(SubRoute.CreateAccount)
                },
                onEntryClick = { entryId ->
                    navController.navigate(SubRoute.Entry(entryId))
                },
                onCategoryClick = { categoryId ->
                    navController.navigate(SubRoute.Category(categoryId))
                }
            )
        }

        composable<TopLevelDestination.Entries> {
            EntriesScreen(
                onAddEntry = { accountId, categoryId ->
                    navController.navigate(
                        SubRoute.CreateEntry(
                            accountId = accountId,
                            categoryId = categoryId
                        )
                    )
                },
                onAddCategory = { navController.navigate(SubRoute.CreateCategory) },
                onAddAccount = { navController.navigate(SubRoute.CreateAccount) },
                onEntryPick = { entryId ->
                    navController.navigate(SubRoute.Entry(entryId))
                }
            )
        }

        composable<TopLevelDestination.Accounts> {
            AccountsScreen(
                onAddAccount = {
                    navController.navigate(SubRoute.CreateAccount)
                },
                visitAccount = { accountId ->
                    navController.navigate(SubRoute.Account(accountId))
                }
            )
        }

        composable<TopLevelDestination.Categories> {
            CategoriesScreen(
                onAddCategory = {
                    navController.navigate(SubRoute.CreateCategory)
                },
                visitCategory = { categoryId ->
                    navController.navigate(SubRoute.Category(categoryId))
                }
            )
        }

        composable<SubRoute.Settings> {
            SettingsScreen(
                onNavigateBack = { navController.popBackStack() }
            )
        }

        composable<SubRoute.CreateAccount> {
            CreateAccountScreen(
                onNavigateBack = { navController.popBackStack() }
            )
        }

        composable<SubRoute.CreateEntry> {
            CreateEntryScreen(
                onNavigateBack = { navController.popBackStack() },
                onAddCategory = { navController.navigate(SubRoute.CreateCategory) },
                onAddAccount = { navController.navigate(SubRoute.CreateAccount) }
            )
        }

        composable<SubRoute.CreateCategory> {
            CreateCategoryScreen(
                onNavigateBack = { navController.popBackStack() }
            )
        }

        composable<SubRoute.Entry> {
            EntryScreen(
                onNavigateBack = { navController.popBackStack() },
                onAddCategory = { navController.navigate(SubRoute.CreateCategory) },
                onAddAccount = { navController.navigate(SubRoute.CreateAccount) },
                visitCategory = { categoryId ->
                    navController.navigate(SubRoute.Category(categoryId))
                },
                visitAccount = { accountId ->
                    navController.navigate(SubRoute.Account(accountId))
                }
            )
        }

        composable<SubRoute.Category> {
            CategoryScreen(
                onNavigateBack = { navController.popBackStack() },
                onEntryClick = { entryId ->
                    navController.navigate(SubRoute.Entry(entryId))
                },
                onAddEntry = { categoryId ->
                    navController.navigate(
                        SubRoute.CreateEntry(accountId = null, categoryId = categoryId)
                    )
                }
            )
        }

        composable<SubRoute.Account> {
            AccountScreen(
                onNavigateBack = { navController.popBackStack() },
                onEntryClick = { entryId ->
                    navController.navigate(SubRoute.Entry(entryId))
                },
                onAddEntry = { accountId ->
                    navController.navigate(
                        SubRoute.CreateEntry(accountId = accountId, categoryId = null)
                    )
                }
            )
        }
    }
}

@Composable
fun BottomBarNavigation(modifier: Modifier = Modifier) {
    val navController = rememberNavController()
    val startDestination = TopLevelDestination.Home

    val navBackStackEntry by navController.currentBackStackEntryAsState()
    val currentDestination = navBackStackEntry?.destination

    val showBottomBar = currentDestination == null ||
        TopLevelDestination.entries.any { destination ->
            currentDestination.hierarchy.any { it.hasRoute(destination::class) }
        }

    Scaffold(
        modifier = modifier,
        containerColor = MaterialTheme.colorScheme.background,
        contentWindowInsets = WindowInsets(0, 0, 0, 0),
        bottomBar = {
            AnimatedVisibility(
                visible = showBottomBar,
                enter = slideInVertically(
                    animationSpec = SunderMotion.offsetTween(SunderMotion.DURATION_MEDIUM)
                ) { it } + fadeIn(SunderMotion.enterTween(SunderMotion.DURATION_SHORT)),
                exit = slideOutVertically(
                    animationSpec = SunderMotion.offsetTween(SunderMotion.DURATION_SHORT)
                ) { it } + fadeOut(SunderMotion.exitTween())
            ) {
                Column {
                    HorizontalDivider(
                        thickness = 0.5.dp,
                        color = MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.6f)
                    )
                    NavigationBar(
                        containerColor = MaterialTheme.colorScheme.surface,
                        tonalElevation = 0.dp,
                        windowInsets = NavigationBarDefaults.windowInsets
                    ) {
                        TopLevelDestination.entries.forEach { destination ->
                            val selected = currentDestination?.hierarchy?.any {
                                it.hasRoute(destination::class)
                            } == true

                            NavigationBarItem(
                                selected = selected,
                                onClick = {
                                    navController.navigate(destination) {
                                        popUpTo(navController.graph.findStartDestination().id) {
                                            saveState = true
                                        }
                                        launchSingleTop = true
                                        restoreState = true
                                    }
                                },
                                icon = {
                                    Icon(
                                        imageVector = if (selected) destination.selectedIcon else destination.unselectedIcon,
                                        contentDescription = stringResource(destination.contentDescriptionRes)
                                    )
                                },
                                label = {
                                    Text(
                                        text = stringResource(destination.labelRes),
                                        style = MaterialTheme.typography.labelSmall,
                                        fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Medium,
                                        maxLines = 1
                                    )
                                },
                                colors = NavigationBarItemDefaults.colors(
                                    indicatorColor = MaterialTheme.colorScheme.primaryContainer,
                                    selectedIconColor = MaterialTheme.colorScheme.primary,
                                    selectedTextColor = MaterialTheme.colorScheme.primary,
                                    unselectedIconColor = MaterialTheme.colorScheme.onSurfaceVariant,
                                    unselectedTextColor = MaterialTheme.colorScheme.onSurfaceVariant
                                )
                            )
                        }
                    }
                }
            }
        }
    ) { contentPadding ->
        AppNavHost(
            navController = navController,
            startDestination = startDestination,
            modifier = Modifier
                .padding(contentPadding)
                .consumeWindowInsets(contentPadding),
        )
    }
}
