package dev.crqch.sunder.data.repositories

import dev.crqch.sunder.api.DefaultApi
import dev.crqch.sunder.data.local.User
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class UserRepository @Inject constructor(
    private val authRepository: AuthRepository,
    private val api: DefaultApi
) {
    private val _currentUser = MutableStateFlow<User?>(null)
    val currentUser: StateFlow<User?> = _currentUser

    suspend fun fetchCurrentUser() {
        val res = api.dashboardIndexFHMHL3Q()
        val body = res.body()

        if (res.isSuccessful && body != null) {
            _currentUser.value = User(
                username = body.username ?: "",
                email = body.email ?: "",
                flags = body.flags ?: emptyList()
            )
        } else {
            _currentUser.value = null
        }
    }

    suspend fun logout() {
        authRepository.clearTokens()
        _currentUser.value = null
    }
}
