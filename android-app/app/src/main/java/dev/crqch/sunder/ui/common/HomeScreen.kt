package dev.crqch.sunder.ui.common

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import dev.crqch.sunder.ui.auth.AuthViewModel
import dev.crqch.sunder.R

@Composable
fun HomeScreen(viewModel: AuthViewModel = hiltViewModel()) {

    val currentUser by viewModel.currentUser.collectAsState()

    if (currentUser != null) {
        Scaffold { innerPadding ->
            Column(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(innerPadding)
                    .padding(vertical = 24.dp, horizontal = 16.dp),
                verticalArrangement = Arrangement.spacedBy(
                    8.dp
                )
            ) {
                Text(
                    text = stringResource(R.string.welcome, currentUser?.username ?: "User"),
                    style = MaterialTheme.typography.headlineMedium
                )
            }
        }
    } else {
        Text(stringResource(R.string.loading))
    }

}