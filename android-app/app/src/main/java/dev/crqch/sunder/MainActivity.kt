package dev.crqch.sunder

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.rememberNavController
import dev.crqch.sunder.ui.SunderApp
import dev.crqch.sunder.ui.theme.SunderTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            SunderTheme {
                Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                    RootContainer(innerPadding = innerPadding)
                }
            }
        }
    }
}

@Composable
fun RootContainer(
    viewModel: AuthViewModel = viewModel(factory = AuthViewModel.Factory),
    innerPadding: PaddingValues
) {
    rememberNavController()

    val isInitializing by viewModel.isInitializing.collectAsState()
    val user by viewModel.currentUser.collectAsState()

    when {
        isInitializing -> {
            Text("Splash screen")
        }

        user != null -> {
            SunderApp(null)
        }

        else -> {
            SunderApp(user)
        }

    }
}