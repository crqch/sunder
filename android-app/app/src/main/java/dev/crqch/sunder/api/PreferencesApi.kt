package dev.crqch.sunder.api

import com.squareup.moshi.Json
import retrofit2.Response
import retrofit2.http.Body
import retrofit2.http.GET
import retrofit2.http.PUT

data class PreferencesResponseDto(
    @Json(name = "default_currency_code") val defaultCurrencyCode: String = "USD",
    @Json(name = "default_currency") val defaultCurrency: CurrencyResponseDto? = null
)

data class UpdatePreferencesRequest(
    @Json(name = "default_currency_code") val defaultCurrencyCode: String
)

interface PreferencesApi {

    @GET("dashboard/eco/preferences")
    suspend fun getPreferences(): Response<PreferencesResponseDto>

    @PUT("dashboard/eco/preferences")
    suspend fun updatePreferences(
        @Body request: UpdatePreferencesRequest
    ): Response<PreferencesResponseDto>
}
