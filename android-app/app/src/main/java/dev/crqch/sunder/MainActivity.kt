package dev.crqch.sunder

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import dagger.hilt.android.AndroidEntryPoint
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.navigation.compose.rememberNavController
import dev.crqch.sunder.ui.SunderApp
import dev.crqch.sunder.ui.auth.AuthViewModel
import dev.crqch.sunder.ui.theme.SunderTheme

@AndroidEntryPoint
class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            SunderTheme {
                RootContainer()
            }
        }
    }
}

@Composable
fun RootContainer(
    viewModel: AuthViewModel = hiltViewModel()
) {
    val isInitializing by viewModel.isInitializing.collectAsState()
    val user by viewModel.currentUser.collectAsState()

    when {
        isInitializing -> {
            Column(Modifier.fillMaxSize(), Arrangement.Center, Alignment.CenterHorizontally) {
                Text("Loading…", style = MaterialTheme.typography.headlineLarge)
            }
        }

        else -> {
            SunderApp(viewModel, user)
        }
    }
}
