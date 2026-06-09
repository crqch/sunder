package dev.crqch.sunder.api

import dev.crqch.sunder.data.sync.SyncRequestDto
import dev.crqch.sunder.data.sync.SyncResponseDto
import retrofit2.Response
import retrofit2.http.Body
import retrofit2.http.POST
import retrofit2.http.Query

interface SyncApi {
    @POST("dashboard/eco/sync")
    suspend fun sync(
        @Query("last") last: String?,
        @Body request: SyncRequestDto
    ): Response<SyncResponseDto>
}
