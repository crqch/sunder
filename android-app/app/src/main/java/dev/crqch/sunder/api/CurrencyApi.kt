package dev.crqch.sunder.api

import com.squareup.moshi.Json
import retrofit2.Response
import retrofit2.http.GET
import retrofit2.http.Query

data class CurrencyResponseDto(
    @Json(name = "code") val code: String,
    @Json(name = "name") val name: String,
    @Json(name = "symbol") val symbol: String,
    @Json(name = "symbol_after") val symbolAfter: Boolean = false,
    @Json(name = "decimal_digits") val decimalDigits: Int = 2
)

data class CurrenciesResponseDto(
    @Json(name = "currencies") val currencies: List<CurrencyResponseDto> = emptyList()
)

data class RatesResponseDto(
    @Json(name = "base") val base: String = "USD",
    @Json(name = "date") val date: String,
    @Json(name = "requested_date") val requestedDate: String? = null,
    @Json(name = "rates") val rates: Map<String, Double> = emptyMap()
)

data class RatesHistoryResponseDto(
    @Json(name = "base") val base: String = "USD",
    @Json(name = "from") val from: String,
    @Json(name = "to") val to: String,
    @Json(name = "rates") val rates: Map<String, Map<String, Double>> = emptyMap()
)

interface CurrencyApi {

    @GET("currencies")
    suspend fun getCurrencies(): Response<CurrenciesResponseDto>

    @GET("currencies/rates")
    suspend fun getRates(
        @Query("base") base: String = "USD",
        @Query("date") date: String? = null,
        @Query("symbols") symbols: String? = null
    ): Response<RatesResponseDto>

    @GET("currencies/rates/history")
    suspend fun getRatesHistory(
        @Query("base") base: String = "USD",
        @Query("from") from: String,
        @Query("to") to: String? = null,
        @Query("symbols") symbols: String? = null
    ): Response<RatesHistoryResponseDto>
}
