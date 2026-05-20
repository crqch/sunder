package dev.crqch.sunder.ui.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import dev.crqch.sunder.AuthViewModel
import dev.crqch.sunder.data.repositories.UserRepository

@Composable
fun HomeScreen(viewModel: AuthViewModel = hiltViewModel()) {

    val currentUser by viewModel.currentUser.collectAsState()

    if (currentUser != null) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(vertical = 16.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(
                8.dp
            )
        ) {
            Text(
                text = "Welcome, ${currentUser?.username ?: "User"}",
                style = MaterialTheme.typography.headlineMedium
            )
        }
    } else {
        Text("Loading...")
    }

}