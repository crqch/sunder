package dev.crqch.sunder.ui

import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Home
import androidx.compose.material.icons.filled.Payments
import androidx.compose.material.icons.filled.SwitchAccount
import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarDefaults
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.navigation.NavDestination.Companion.hasRoute
import androidx.navigation.NavDestination.Companion.hierarchy
import androidx.navigation.NavGraph.Companion.findStartDestination
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import dev.crqch.sunder.R
import dev.crqch.sunder.ui.accounts.AccountsScreen
import dev.crqch.sunder.ui.accounts.CreateAccountScreen
import dev.crqch.sunder.ui.categories.CreateCategoryScreen
import dev.crqch.sunder.ui.entries.CreateEntryScreen
import dev.crqch.sunder.ui.entries.EntriesScreen
import dev.crqch.sunder.ui.entries.EntryScreen
import dev.crqch.sunder.ui.common.HomeScreen
import kotlinx.serialization.Serializable

sealed interface SubRoute {
    @Serializable

    data class CreateEntry(val accountId: String?, val categoryId: String?) : SubRoute

    @Serializable
    object CreateCategory : SubRoute {

    }

    @Serializable
    object CreateAccount : SubRoute

    @Serializable
    data class Entry(val entryId: String) : SubRoute {

    }
}

sealed class TopLevelDestination {
    abstract val labelRes: Int
    abstract val icon: ImageVector
    abstract val contentDescriptionRes: Int
    abstract val order: Int

    // 2. Now the children can be cleanly @Serializable
    @Serializable
    object Entries : TopLevelDestination() {
        override val labelRes = R.string.entries
        override val icon = Icons.Default.Payments
        override val contentDescriptionRes = R.string.entries
        override val order = 0
    }

    @Serializable
    object Home : TopLevelDestination() {
        override val labelRes = R.string.home
        override val icon = Icons.Default.Home
        override val contentDescriptionRes = R.string.home
        override val order = 1
    }

    @Serializable
    object Accounts : TopLevelDestination() {
        override val labelRes = R.string.accounts
        override val icon = Icons.Default.SwitchAccount
        override val contentDescriptionRes = R.string.accounts
        override val order = 2
    }

    companion object {
        val entries by lazy { listOf(Entries, Home, Accounts) }

        fun fromRouteClass(routeClass: String?): TopLevelDestination? {
            return entries.find { routeClass?.contains(it::class.qualifiedName ?: "") == true }
        }
    }
}

@Composable
fun AppNavHost(
    navController: NavHostController,
    startDestination: TopLevelDestination, modifier: Modifier = Modifier
) {
    NavHost(
        navController,
        startDestination = startDestination,
        modifier = modifier,
        enterTransition = {
            val initial = TopLevelDestination.fromRouteClass(initialState.destination.route)
            val target = TopLevelDestination.fromRouteClass(targetState.destination.route)
            if (initial != null && target != null) {
                val isLeft = target.order < initial.order
                slideInHorizontally(
                    animationSpec = smoothOffsetSpec,
                    initialOffsetX = { if (isLeft) -it else it })
            } else slideInHorizontally(animationSpec = smoothOffsetSpec, initialOffsetX = { it })
        },
        exitTransition = {
            val initial = TopLevelDestination.fromRouteClass(initialState.destination.route)
            val target = TopLevelDestination.fromRouteClass(targetState.destination.route)
            if (initial != null && target != null) {
                val isLeft = target.order < initial.order
                slideOutHorizontally(
                    animationSpec = smoothOffsetSpec,
                    targetOffsetX = { if (isLeft) it else -it })
            } else slideOutHorizontally(animationSpec = smoothOffsetSpec, targetOffsetX = { -it })
        },
        popEnterTransition = {
            val initial = TopLevelDestination.fromRouteClass(initialState.destination.route)
            val target = TopLevelDestination.fromRouteClass(targetState.destination.route)
            if (initial != null && target != null) {
                val isLeft = target.order < initial.order
                slideInHorizontally(
                    animationSpec = smoothOffsetSpec,
                    initialOffsetX = { if (isLeft) -it else it })
            } else slideInHorizontally(animationSpec = smoothOffsetSpec, initialOffsetX = { -it })
        },
        popExitTransition = {
            val initial = TopLevelDestination.fromRouteClass(initialState.destination.route)
            val target = TopLevelDestination.fromRouteClass(targetState.destination.route)
            if (initial != null && target != null) {
                val isLeft = target.order < initial.order
                slideOutHorizontally(
                    animationSpec = smoothOffsetSpec,
                    targetOffsetX = { if (isLeft) it else -it })
            } else slideOutHorizontally(animationSpec = smoothOffsetSpec, targetOffsetX = { it })
        }
    ) {
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
                onAddAccount = {
                    navController.navigate(SubRoute.CreateAccount)
                },
                onEntryPick = { entryId ->
                    navController.navigate(SubRoute.Entry(entryId))
                }
            )
        }

        composable<TopLevelDestination.Home> {
            HomeScreen()
        }

        composable<TopLevelDestination.Accounts> {
            AccountsScreen(
                onAddAccount = {
                    navController.navigate(SubRoute.CreateAccount)
                }
            )
        }

        composable<SubRoute.CreateAccount> {
            CreateAccountScreen(onNavigateBack = { navController.popBackStack() })
        }

        composable<SubRoute.CreateEntry> {
            CreateEntryScreen(
                onNavigateBack = { navController.popBackStack() },
                onAddCategory = { navController.navigate(SubRoute.CreateCategory) },
                onAddAccount = {
                    navController.navigate(SubRoute.CreateAccount)
                }
            )
        }

        composable<SubRoute.CreateCategory> {
            CreateCategoryScreen(
                onNavigateBack = { navController.popBackStack() }
            )
        }

        composable<SubRoute.Entry> {
            EntryScreen()
        }
    }
}

@Composable
fun BottomBarNavigation(modifier: Modifier = Modifier) {
    val navController = rememberNavController()
    val startDestination = TopLevelDestination.Home

    val navBackStackEntry by navController.currentBackStackEntryAsState()
    val currentDestination = navBackStackEntry?.destination

    Scaffold(
        modifier = modifier,
        bottomBar = {
            NavigationBar(windowInsets = NavigationBarDefaults.windowInsets) {
                TopLevelDestination.entries.forEach { destination ->
                    NavigationBarItem(
                        selected = currentDestination?.hierarchy?.any { it.hasRoute(destination::class) } == true,
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
                                destination.icon,
                                contentDescription = stringResource(destination.contentDescriptionRes)
                            )
                        },
                        label = { Text(stringResource(destination.labelRes)) }
                    )
                }
            }
        }
    ) { contentPadding ->
        AppNavHost(
            navController,
            startDestination,
            modifier = Modifier
                .padding(contentPadding)
                .consumeWindowInsets(contentPadding),
        )
    }
}
