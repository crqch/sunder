package dev.crqch.sunder.data.repositories

import dev.crqch.sunder.data.local.User
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.firstOrNull

class UserRepository(
    private val authRepository: AuthRepository
) {
    private val _currentUser = MutableStateFlow<User?>(null)
    val currentUser: StateFlow<User?> = _currentUser

    suspend fun loginWithExistingToken(): Boolean {
        val token = authRepository.accessToken.firstOrNull()
        if (token == null) return false

        return try {
            val mockUser = User(username = "Michał", email = "test@gmail.com", flags = listOf())
            _currentUser.value = mockUser
            true
        } catch (e: Exception) {
            authRepository.clearTokens()
            _currentUser.value = null
            false
        }
    }

    suspend fun logout() {
        authRepository.clearTokens()
        _currentUser.value = null
    }
}
