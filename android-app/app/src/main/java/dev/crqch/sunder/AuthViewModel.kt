package dev.crqch.sunder

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.crqch.sunder.api.DefaultApi
import dev.crqch.sunder.data.repositories.AuthRepository
import dev.crqch.sunder.data.repositories.UserRepository
import dev.crqch.sunder.models.AuthLoginTHN4OMARequest
import dev.crqch.sunder.ui.screens.auth.SignInFormFields
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONObject
import javax.inject.Inject

@HiltViewModel
class AuthViewModel @Inject constructor(
    private val api: DefaultApi,
    private val userRepository: UserRepository,
    private val authRepository: AuthRepository
) : ViewModel() {

    private val _isInitializing = MutableStateFlow(true)
    val isInitializing: StateFlow<Boolean> = _isInitializing

    val currentUser = userRepository.currentUser

    init {
        checkAuthStatus()
    }

    private fun checkAuthStatus() {
        viewModelScope.launch {
            _isInitializing.value = true
            try {
                withContext(Dispatchers.IO) {
                    userRepository.fetchCurrentUser()
                }
            } catch (e: Exception) {
                // Handle potential network errors or connection refused gracefully
                // This prevents the app from crashing on start if the server is down
            } finally {
                _isInitializing.value = false
            }
        }
    }

    data class SignInResponse(
        val success: Boolean,
        val errorMessage: String? = null
    )

    suspend fun signIn(signInFormFields: SignInFormFields): SignInResponse =
        withContext(Dispatchers.IO) {
            val res = try {
                api.authLoginTHN4OMA(
                    AuthLoginTHN4OMARequest(
                        signInFormFields.login,
                        signInFormFields.password
                    )
                )
            } catch (e: Exception) {
                return@withContext SignInResponse(success = false, errorMessage = e.message)
            }

            if (res.isSuccessful) {
                authRepository.saveTokens("", res.body()!!.refreshToken!!)
                userRepository.fetchCurrentUser()
                SignInResponse(success = true)
            } else {
                val errorBody = res.errorBody()?.string()
                val message = try {
                    if (errorBody != null) {
                        "${JSONObject(errorBody).getString("message")} (${
                            JSONObject(errorBody).getString(
                                "error_code"
                            )
                        })"
                    } else {
                        res.message()
                    }
                } catch (e: Exception) {
                    res.message()
                }
                SignInResponse(success = false, errorMessage = message)
            }
        }
}
