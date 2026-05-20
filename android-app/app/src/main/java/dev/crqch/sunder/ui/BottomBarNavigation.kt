package dev.crqch.sunder.ui

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.layout.PaddingValues
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
import androidx.compose.ui.unit.dp
import androidx.navigation.NavDestination.Companion.hierarchy
import androidx.navigation.NavGraph.Companion.findStartDestination
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import dev.crqch.sunder.ui.screens.AccountsScreen
import dev.crqch.sunder.ui.screens.CreateAccountScreen
import dev.crqch.sunder.ui.screens.EntriesScreen
import dev.crqch.sunder.ui.screens.HomeScreen

enum class Destination(
    val route: String,
    val label: String,
    val icon: ImageVector,
    val contentDescription: String
) {
    ENTRIES("entries", "Entries", Icons.Default.Payments, "Entries"),
    HOME("home", "Home", Icons.Default.Home, "Home"),
    ACCOUNTS("accounts", "Accounts", Icons.Default.SwitchAccount, "Accounts")
}

@Composable
fun AppNavHost(
    navController: NavHostController,
    startDestination: Destination,
    modifier: Modifier = Modifier
) {
    NavHost(
        navController,
        startDestination = startDestination.route,
        modifier = modifier,
        enterTransition = {
            val initial = Destination.entries.find { it.route == initialState.destination.route }
            val target = Destination.entries.find { it.route == targetState.destination.route }
            if (initial != null && target != null) {
                val isLeft = target.ordinal < initial.ordinal
                slideInHorizontally(
                    animationSpec = smoothOffsetSpec,
                    initialOffsetX = { if (isLeft) -it else it })
            } else {
                slideInHorizontally(
                    animationSpec = smoothOffsetSpec,
                    initialOffsetX = { it })
            }
        },
        exitTransition = {
            val initial = Destination.entries.find { it.route == initialState.destination.route }
            val target = Destination.entries.find { it.route == targetState.destination.route }
            if (initial != null && target != null) {
                val isLeft = target.ordinal < initial.ordinal
                slideOutHorizontally(
                    animationSpec = smoothOffsetSpec,
                    targetOffsetX = { if (isLeft) it else -it })
            } else {
                slideOutHorizontally(
                    animationSpec = smoothOffsetSpec,
                    targetOffsetX = { -it })
            }
        },
        popEnterTransition = {
            val initial = Destination.entries.find { it.route == initialState.destination.route }
            val target = Destination.entries.find { it.route == targetState.destination.route }
            if (initial != null && target != null) {
                val isLeft = target.ordinal < initial.ordinal
                slideInHorizontally(
                    animationSpec = smoothOffsetSpec,
                    initialOffsetX = { if (isLeft) -it else it })
            } else {
                slideInHorizontally(
                    animationSpec = smoothOffsetSpec,
                    initialOffsetX = { -it })
            }
        },
        popExitTransition = {
            val initial = Destination.entries.find { it.route == initialState.destination.route }
            val target = Destination.entries.find { it.route == targetState.destination.route }
            if (initial != null && target != null) {
                val isLeft = target.ordinal < initial.ordinal
                slideOutHorizontally(
                    animationSpec = smoothOffsetSpec,
                    targetOffsetX = { if (isLeft) it else -it })
            } else {
                slideOutHorizontally(
                    animationSpec = smoothOffsetSpec,
                    targetOffsetX = { it })
            }
        }
    ) {
        Destination.entries.forEach { destination ->
            composable(destination.route) {
                when (destination) {
                    Destination.ENTRIES -> EntriesScreen()
                    Destination.HOME -> HomeScreen()
                    Destination.ACCOUNTS -> AccountsScreen(
                        onAddAccount = { navController.navigate("create_account") }
                    )
                }
            }
        }
        composable("create_account") {
            CreateAccountScreen(onNavigateBack = { navController.popBackStack() })
        }
    }
}

@Composable
fun BottomBarNavigation(modifier: Modifier = Modifier) {
    val navController = rememberNavController()
    val startDestination = Destination.HOME

    val navBackStackEntry by navController.currentBackStackEntryAsState()
    val currentDestination = navBackStackEntry?.destination

    Scaffold(
        modifier = modifier,
        bottomBar = {
            NavigationBar(windowInsets = NavigationBarDefaults.windowInsets) {
                Destination.entries.forEach { destination ->
                    NavigationBarItem(
                        selected = currentDestination?.hierarchy?.any { it.route == destination.route } == true,
                        onClick = {
                            navController.navigate(destination.route) {
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
                                contentDescription = destination.contentDescription
                            )
                        },
                        label = { Text(destination.label) }
                    )
                }
            }
        }
    ) { contentPadding ->
        AppNavHost(
            navController,
            startDestination,
            modifier = Modifier.padding(contentPadding).consumeWindowInsets(contentPadding),
        )
    }
}
