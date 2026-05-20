package dev.crqch.sunder.ui.screens.auth

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import android.os.Parcelable
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.text.input.KeyboardType
import kotlinx.parcelize.Parcelize

@Parcelize
data class SignInFormFields(val username: String = "", val password: String = "") : Parcelable {

    fun isFilled(): Boolean {
        return this.username != "" && this.password != ""
    }
}

@Composable
fun SignInScreen(
    onSignInClick: (SignInFormFields) -> Unit,
    onNavigateToSignUp: () -> Unit // <-- Add your navigation callback
) {
    var input by rememberSaveable { mutableStateOf(SignInFormFields()) }

    Scaffold(
        bottomBar = {
            Column(
                Modifier
                    .padding(16.dp)
                    .fillMaxWidth()
                    .imePadding()
                    .offset(y = (-10).dp),

                verticalArrangement = Arrangement.spacedBy(4.dp)
            ) {

                Button(
                    onClick = {
                        onSignInClick(input)
                    },
                    enabled = input.isFilled(),
                    modifier = Modifier
                        .fillMaxWidth()
                ) {
                    Text("Sign in")
                }
                OutlinedButton(
                    onClick = {
                        onNavigateToSignUp()
                    },
                    modifier = Modifier
                        .fillMaxWidth()
                ) {
                    Text("Sign up")
                }
            }
        }
    ) { innerPadding ->
        // The scrollable area
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .verticalScroll(rememberScrollState())
                .padding(horizontal = 16.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(
                8.dp,
                Alignment.CenterVertically
            )
        ) {
            Text(
                "Sign in",
                Modifier.padding(bottom = 16.dp),
                fontSize = 10.em,
                fontWeight = FontWeight.Bold,
            )
            TextField(
                value = input.username,
                onValueChange = { input = input.copy(username = it) },
                label = { Text("Username") })
            TextField(
                value = input.password,
                onValueChange = { input = input.copy(password = it) },
                label = { Text("Password") },
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password)
            )
        }
    }
}