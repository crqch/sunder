package dev.crqch.sunder.ui

import android.widget.Toast
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import dev.crqch.sunder.R
import dev.crqch.sunder.data.local.User
import dev.crqch.sunder.ui.auth.AuthViewModel
import dev.crqch.sunder.ui.auth.SignInScreen
import dev.crqch.sunder.ui.auth.SignUpScreen
import dev.crqch.sunder.ui.theme.SunderMotion
import kotlinx.coroutines.launch
import kotlinx.serialization.Serializable

@Serializable
object Main

@Serializable
object SignIn

@Serializable
object SignUp

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
        navController = navController,
        startDestination = if (user == null) SignIn else Main,
        enterTransition = {
            fadeIn(animationSpec = SunderMotion.enterTween()) +
                slideInHorizontally(animationSpec = SunderMotion.offsetTween()) { it / 6 }
        },
        exitTransition = {
            fadeOut(animationSpec = SunderMotion.exitTween()) +
                slideOutHorizontally(animationSpec = SunderMotion.offsetTween()) { -it / 6 }
        },
        popEnterTransition = {
            fadeIn(animationSpec = SunderMotion.enterTween()) +
                slideInHorizontally(animationSpec = SunderMotion.offsetTween()) { -it / 6 }
        },
        popExitTransition = {
            fadeOut(animationSpec = SunderMotion.exitTween()) +
                slideOutHorizontally(animationSpec = SunderMotion.offsetTween()) { it / 6 }
        }
    ) {
        composable<Main> {
            BottomBarNavigation()
        }
        composable<SignIn> {
            val scope = rememberCoroutineScope()
            val context = LocalContext.current
            var isSubmitting by remember { mutableStateOf(false) }

            SignInScreen(
                onSignInClick = { fields ->
                    if (!isSubmitting) {
                        scope.launch {
                            isSubmitting = true
                            val res = authViewModel.signIn(fields)
                            isSubmitting = false
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
                    }
                },
                onNavigateToSignUp = {
                    navController.navigate(SignUp) {
                        popUpTo(SignIn) {
                            inclusive = true
                        }
                    }
                },
                isLoading = isSubmitting
            )
        }
        composable<SignUp> {
            val scope = rememberCoroutineScope()
            val context = LocalContext.current
            var isSubmitting by remember { mutableStateOf(false) }

            SignUpScreen(
                onSignUpClick = { data ->
                    if (!isSubmitting) {
                        scope.launch {
                            isSubmitting = true
                            val res = authViewModel.signUp(data)
                            isSubmitting = false
                            if (res.success) {
                                Toast.makeText(
                                    context,
                                    context.getString(R.string.account_created),
                                    Toast.LENGTH_SHORT
                                ).show()
                                navController.navigate(SignIn) {
                                    popUpTo(SignUp) {
                                        inclusive = true
                                    }
                                }
                            } else {
                                Toast.makeText(
                                    context,
                                    res.errorMessage ?: context.getString(R.string.sign_up_failed),
                                    Toast.LENGTH_SHORT
                                ).show()
                            }
                        }
                    }
                },
                onNavigateToSignIn = {
                    navController.navigate(SignIn) {
                        popUpTo(SignUp) {
                            inclusive = true
                        }
                    }
                },
                isLoading = isSubmitting
            )
        }
    }
}