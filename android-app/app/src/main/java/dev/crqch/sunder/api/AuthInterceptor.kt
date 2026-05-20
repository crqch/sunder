package dev.crqch.sunder.api

import dev.crqch.sunder.data.repositories.AuthRepository
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import okhttp3.Interceptor
import okhttp3.Response
import javax.inject.Provider

class AuthInterceptor(private val authRepositoryProvider: Provider<AuthRepository>) : Interceptor {

    private class RetryMarker

    override fun intercept(chain: Interceptor.Chain): Response {
        val authRepository = authRepositoryProvider.get()
        val originalRequest = chain.request()

        // 1. Skip interception for auth endpoints to avoid loops
        if (originalRequest.url.encodedPath.contains("auth/refresh_token") ||
            originalRequest.url.encodedPath.contains("auth/login")
        ) {
            return chain.proceed(originalRequest)
        }

        // 2. Get current access token synchronously from Flow to ensure it's loaded from disk
        val accessToken = runBlocking { authRepository.accessTokenFlow.first() }

        // 3. Add token to request if available
        val requestWithToken = if (accessToken != null) {
            originalRequest.newBuilder()
                .header("Cookie", "authorization=$accessToken")
                .build()
        } else {
            originalRequest
        }

        // 4. Proceed with the request
        val response = chain.proceed(requestWithToken)

        // 5. If 401 and not already a retry, try to refresh token
        if (response.code == 401 && originalRequest.tag(RetryMarker::class.java) == null) {
            val refreshToken = authRepository.refreshToken.value

            if (refreshToken != null) {
                // Synchronize to avoid multiple simultaneous refreshes
                val latestToken = synchronized(this) {
                    val currentToken = authRepository.accessToken.value
                    if (currentToken != accessToken) {
                        // Someone else already refreshed the token
                        currentToken
                    } else {
                        // Try to refresh - this still needs runBlocking as interceptor is sync
                        runBlocking { authRepository.refreshTokens() }
                    }
                }

                if (latestToken != null) {
                    response.close()
                    val retryRequest = originalRequest.newBuilder()
                        .header("Cookie", "authorization=$latestToken")
                        .tag(RetryMarker::class.java, RetryMarker())
                        .build()
                    return chain.proceed(retryRequest)
                }
            }
        }

        return response
    }
}
