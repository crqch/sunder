package dev.crqch.sunder.ui

import androidx.compose.runtime.Composable
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

@Composable
fun SunderApp(user: User?) {
    val navController = rememberNavController()
    NavHost(navController, startDestination = if (user == null) SignIn else Home) {
        composable<Home> {
            HomeScreen()
        }
        composable<SignIn> {
            SignInScreen(
                {

                },
                {
                    navController.navigate(SignUp)
                }
            )
        }
        composable<SignUp> {
            SignUpScreen(
                {

                },
                {
                    navController.navigate(SignIn)
                }
            )
        }
    }
}