package dev.crqch.sunder.api

import com.squareup.moshi.Json
import retrofit2.Response
import retrofit2.http.Body
import retrofit2.http.GET
import retrofit2.http.POST
import retrofit2.http.PUT
import retrofit2.http.Path
import retrofit2.http.Query

data class AdminCurrencyDto(
    @Json(name = "code") val code: String,
    @Json(name = "name") val name: String,
    @Json(name = "symbol") val symbol: String,
    @Json(name = "symbol_after") val symbolAfter: Boolean = false,
    @Json(name = "decimal_digits") val decimalDigits: Int = 2,
    @Json(name = "is_active") val isActive: Boolean = true
)

data class AdminCurrenciesResponseDto(
    @Json(name = "currencies") val currencies: List<AdminCurrencyDto> = emptyList()
)

data class CreateCurrencyRequest(
    @Json(name = "code") val code: String,
    @Json(name = "name") val name: String,
    @Json(name = "symbol") val symbol: String,
    @Json(name = "symbol_after") val symbolAfter: Boolean,
    @Json(name = "decimal_digits") val decimalDigits: Int,
    @Json(name = "is_active") val isActive: Boolean = true
)

data class UpdateCurrencyRequest(
    @Json(name = "name") val name: String? = null,
    @Json(name = "symbol") val symbol: String? = null,
    @Json(name = "symbol_after") val symbolAfter: Boolean? = null,
    @Json(name = "decimal_digits") val decimalDigits: Int? = null,
    @Json(name = "is_active") val isActive: Boolean? = null
)

data class CurrencyMutationResponseDto(
    @Json(name = "success") val success: Boolean = false,
    @Json(name = "currency") val currency: AdminCurrencyDto? = null
)

data class SyncRatesResponseDto(
    @Json(name = "success") val success: Boolean = false,
    @Json(name = "date") val date: String? = null,
    @Json(name = "from") val from: String? = null,
    @Json(name = "to") val to: String? = null,
    @Json(name = "source") val source: String? = null,
    @Json(name = "stored_count") val storedCount: Int? = null
)

data class UpsertRateRequest(
    @Json(name = "base_currency_code") val baseCurrencyCode: String? = null,
    @Json(name = "target_currency_code") val targetCurrencyCode: String,
    @Json(name = "rate") val rate: Double,
    @Json(name = "rate_date") val rateDate: String? = null
)

data class ExchangeRateDto(
    @Json(name = "id") val id: String? = null,
    @Json(name = "base_currency_code") val baseCurrencyCode: String? = null,
    @Json(name = "target_currency_code") val targetCurrencyCode: String? = null,
    @Json(name = "rate") val rate: Double? = null,
    @Json(name = "rate_date") val rateDate: String? = null,
    @Json(name = "source") val source: String? = null
)

data class UpsertRateResponseDto(
    @Json(name = "success") val success: Boolean = false,
    @Json(name = "rate") val rate: ExchangeRateDto? = null
)

interface AdminApi {

    @GET("admin/currencies")
    suspend fun getCurrencies(): Response<AdminCurrenciesResponseDto>

    @POST("admin/currencies")
    suspend fun createCurrency(
        @Body request: CreateCurrencyRequest
    ): Response<CurrencyMutationResponseDto>

    @PUT("admin/currencies/{code}")
    suspend fun updateCurrency(
        @Path("code") code: String,
        @Body request: UpdateCurrencyRequest
    ): Response<CurrencyMutationResponseDto>

    @POST("admin/currencies/sync-rates")
    suspend fun syncRates(
        @Query("date") date: String? = null,
        @Query("days") days: Int? = null,
        @Query("force") force: Boolean? = null
    ): Response<SyncRatesResponseDto>

    @PUT("admin/currencies/rates")
    suspend fun upsertRate(
        @Body request: UpsertRateRequest
    ): Response<UpsertRateResponseDto>
}
