package dev.crqch.sunder.ui

import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.tween
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.runtime.Composable
import androidx.compose.ui.unit.IntOffset
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import dev.crqch.sunder.data.local.User
import dev.crqch.sunder.ui.screens.HomeScreen
import dev.crqch.sunder.ui.screens.auth.SignInScreen
import dev.crqch.sunder.ui.screens.auth.SignUpScreen
import kotlinx.serialization.Serializable

@Serializable
object Home

@Serializable
object SignIn

@Serializable
object SignUp

val smoothSpec = tween<Float>(
    durationMillis = 500,
    easing = FastOutSlowInEasing
)

val smoothOffsetSpec = tween<IntOffset>(
    durationMillis = 500,
    easing = FastOutSlowInEasing
)

@Composable
fun SunderApp(user: User?) {
    val navController = rememberNavController()
    NavHost(
        navController, startDestination = if (user == null) SignIn else Home,
        enterTransition = {
            slideInHorizontally(animationSpec = smoothOffsetSpec, initialOffsetX = { it }) +
                    scaleIn(animationSpec = smoothSpec, initialScale = 0.85f)
        },
        exitTransition = {
            slideOutHorizontally(animationSpec = smoothOffsetSpec, targetOffsetX = { -it }) +
                    scaleOut(animationSpec = smoothSpec, targetScale = 0.85f)
        },
        popEnterTransition = {
            slideInHorizontally(animationSpec = smoothOffsetSpec, initialOffsetX = { -it }) +
                    scaleIn(animationSpec = smoothSpec, initialScale = 0.85f)
        },
        popExitTransition = {
            slideOutHorizontally(animationSpec = smoothOffsetSpec, targetOffsetX = { it }) +
                    scaleOut(animationSpec = smoothSpec, targetScale = 0.85f)
        }) {
        composable<Home> {
            HomeScreen()
        }
        composable<SignIn> {
            SignInScreen(
                {
                    navController.navigate(Home) {
                        popUpTo(SignIn) {
                            inclusive = true
                        }
                    }
                },
                {
                    navController.navigate(SignUp) {
                        popUpTo(SignIn) {
                            inclusive = true
                        }
                    }
                }
            )
        }
        composable<SignUp> {
            SignUpScreen(
                {
                    navController.navigate(Home) {
                        popUpTo(SignUp) {
                            inclusive = true
                        }
                    }
                },
                {
                    navController.navigate(SignIn) {
                        popUpTo(SignUp) {
                            inclusive = true
                        }
                    }
                }
            )
        }
    }
}