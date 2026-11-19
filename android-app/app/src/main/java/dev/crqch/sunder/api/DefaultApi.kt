package dev.crqch.sunder.api

import com.squareup.moshi.Json
import retrofit2.Response
import retrofit2.http.Body
import retrofit2.http.GET
import retrofit2.http.POST
import retrofit2.http.Path

data class AuthLoginRequest(
    @Json(name = "login") val login: String,
    @Json(name = "pass") val pass: String
)

data class AuthLoginResponse(
    @Json(name = "id") val id: String? = null,
    @Json(name = "message") val message: String? = null,
    @Json(name = "refresh_token") val refreshToken: String? = null
)

data class AuthRegisterRequest(
    @Json(name = "email") val email: String,
    @Json(name = "pass") val pass: String,
    @Json(name = "username") val username: String
)

data class AuthRegisterResponse(
    @Json(name = "id") val id: String? = null,
    @Json(name = "message") val message: String? = null
)

data class AuthRefreshTokenRequest(
    @Json(name = "token") val token: String
)

data class AuthRefreshTokenResponse(
    @Json(name = "id") val id: String? = null,
    @Json(name = "message") val message: String? = null,
    @Json(name = "access_token") val accessToken: String? = null
)

data class DashboardUserResponse(
    @Json(name = "id") val id: String? = null,
    @Json(name = "username") val username: String? = null,
    @Json(name = "email") val email: String? = null,
    @Json(name = "flags") val flags: List<String>? = null
)

interface DefaultApi {
    @POST("auth/login")
    suspend fun login(@Body request: AuthLoginRequest): Response<AuthLoginResponse>

    @POST("auth/register/{invite}")
    suspend fun register(
        @Path("invite") invite: String,
        @Body request: AuthRegisterRequest
    ): Response<AuthRegisterResponse>

    @POST("auth/refresh_token")
    suspend fun refreshToken(@Body request: AuthRefreshTokenRequest): Response<AuthRefreshTokenResponse>

    @GET("dashboard")
    suspend fun getDashboardUser(): Response<DashboardUserResponse>
}
