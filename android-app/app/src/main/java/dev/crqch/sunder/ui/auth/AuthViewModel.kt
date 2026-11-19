package dev.crqch.sunder.ui.auth

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.crqch.sunder.api.AuthLoginRequest
import dev.crqch.sunder.api.AuthRegisterRequest
import dev.crqch.sunder.api.DefaultApi
import dev.crqch.sunder.data.local.SunderDatabase
import dev.crqch.sunder.data.repositories.AppSettings
import dev.crqch.sunder.data.repositories.AuthRepository
import dev.crqch.sunder.data.repositories.PreferencesRepository
import dev.crqch.sunder.data.repositories.SyncRepository
import dev.crqch.sunder.data.repositories.UserRepository
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
    private val authRepository: AuthRepository,
    private val syncRepository: SyncRepository,
    private val database: SunderDatabase,
    private val preferencesRepository: PreferencesRepository
) : ViewModel() {

    private val _isInitializing = MutableStateFlow(true)
    val isInitializing: StateFlow<Boolean> = _isInitializing

    private val _logoutError = MutableStateFlow<String?>(null)
    val logoutError: StateFlow<String?> = _logoutError

    val currentUser = userRepository.currentUser
    val appSettings: StateFlow<AppSettings> = preferencesRepository.settings

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
                api.login(
                    AuthLoginRequest(
                        login = signInFormFields.login,
                        pass = signInFormFields.password
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

    suspend fun signUp(signUpFormData: SignUpFormData): SignInResponse =
        withContext(Dispatchers.IO) {
            val res = try {
                api.register(
                    invite = signUpFormData.inviteCode,
                    request = AuthRegisterRequest(
                        email = signUpFormData.email,
                        pass = signUpFormData.password,
                        username = signUpFormData.username
                    )
                )
            } catch (e: Exception) {
                return@withContext SignInResponse(success = false, errorMessage = e.message)
            }

            if (res.isSuccessful) {
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

    fun signOut(onComplete: () -> Unit) {
        viewModelScope.launch {
            _logoutError.value = null
            try {
                val hasUnsynced = withContext(Dispatchers.IO) {
                    syncRepository.hasUnsyncedData()
                }
                if (hasUnsynced) {
                    _logoutError.value = "Cannot logout: some data is not synced yet."
                    return@launch
                }

                withContext(Dispatchers.IO) {
                    database.clearAllTables()
                    userRepository.logout()
                }
                onComplete()
            } catch (e: Exception) {
                _logoutError.value = "Logout failed: ${e.message}"
            }
        }
    }

    fun clearLogoutError() {
        _logoutError.value = null
    }
}