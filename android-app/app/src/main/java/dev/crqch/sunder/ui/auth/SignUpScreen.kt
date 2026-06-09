package dev.crqch.sunder.ui.auth

import android.os.Parcelable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import dev.crqch.sunder.R
import kotlinx.parcelize.Parcelize

@Parcelize
data class SignUpFormData(
    val email: String = "",
    val username: String = "",
    val password: String = "",
    val inviteCode: String = ""
) : Parcelable {
    fun isFilled(): Boolean {
        return !email.isEmpty() && !username.isEmpty() && !password.isEmpty() && !inviteCode.isEmpty()
    }
}

@Composable
fun SignUpScreen(
    onSignUpClick: (SignUpFormData) -> Unit,
    onNavigateToSignIn: () -> Unit
) {

    var input by rememberSaveable { mutableStateOf(SignUpFormData()) }

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
                        onSignUpClick(input)
                    },
                    enabled = input.isFilled(),
                    modifier = Modifier
                        .fillMaxWidth()
                ) {
                    Text(stringResource(R.string.sign_up))
                }
                OutlinedButton(
                    onClick = {
                        onNavigateToSignIn()
                    },
                    modifier = Modifier
                        .fillMaxWidth()
                ) {
                    Text(stringResource(R.string.sign_in))
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
                stringResource(R.string.sign_up),
                Modifier.padding(bottom = 16.dp),
                fontSize = 10.em,
                fontWeight = FontWeight.Bold,
            )
            TextField(
                value = input.email,
                onValueChange = { input = input.copy(email = it) },
                label = { Text(stringResource(R.string.email)) },
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email)
            )
            TextField(
                value = input.username,
                onValueChange = { input = input.copy(username = it) },
                label = { Text(stringResource(R.string.username)) })
            TextField(
                value = input.password,
                onValueChange = { input = input.copy(password = it) },
                label = { Text(stringResource(R.string.password)) },
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password)
            )
            TextField(
                value = input.inviteCode,
                onValueChange = { input = input.copy(inviteCode = it) },
                label = { Text("Invite code") },
                visualTransformation = PasswordVisualTransformation(),
                keyboardOptions = KeyboardOptions()
            )
        }
    }
}