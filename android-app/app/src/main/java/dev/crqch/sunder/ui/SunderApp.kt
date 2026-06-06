package dev.crqch.sunder.ui

import android.widget.Toast
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.tween
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.IntOffset
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import dev.crqch.sunder.ui.auth.AuthViewModel
import dev.crqch.sunder.R
import dev.crqch.sunder.data.local.User
import dev.crqch.sunder.ui.auth.SignInScreen
import dev.crqch.sunder.ui.auth.SignUpScreen
import kotlinx.coroutines.launch
import kotlinx.serialization.Serializable

@Serializable
object Main

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
fun SunderApp(authViewModel: AuthViewModel, user: User?) {
    val navController = rememberNavController()

    LaunchedEffect(user) {
        if (user == null) {
            navController.navigate(SignIn) {
                popUpTo(0) { inclusive = true }
            }
        }
    }

    NavHost(
        navController, startDestination = if (user == null) SignIn else Main,
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
        composable<Main> {
            BottomBarNavigation()
        }
        composable<SignIn> {
            val scope = rememberCoroutineScope()
            val context = LocalContext.current
            SignInScreen(
                {
                    scope.launch {
                        val res = authViewModel.signIn(it)
                        if (res.success) {

                            navController.navigate(Main) {
                                popUpTo(SignIn) {
                                    inclusive = true
                                }
                            }
                        } else {
                            Toast.makeText(
                                context,
                                res.errorMessage ?: context.getString(R.string.sign_in_failed),
                                Toast.LENGTH_SHORT
                            ).show()
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
                    navController.navigate(Main) {
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