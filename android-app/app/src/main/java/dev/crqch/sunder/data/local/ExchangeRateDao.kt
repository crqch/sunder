package dev.crqch.sunder.data.local

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy.Companion.REPLACE
import androidx.room.Query
import kotlinx.coroutines.flow.Flow

@Dao
interface ExchangeRateDao {

    @Query("select * from exchange_rates order by rate_date asc, currency_code asc")
    fun observeAll(): Flow<List<ExchangeRateEntity>>

    @Query("select * from exchange_rates order by rate_date asc, currency_code asc")
    suspend fun getAll(): List<ExchangeRateEntity>

    @Query("select * from exchange_rates where currency_code = :code order by rate_date asc")
    suspend fun getForCurrency(code: String): List<ExchangeRateEntity>

    @Query("select * from exchange_rates where rate_date = :date order by currency_code asc")
    suspend fun getForDate(date: String): List<ExchangeRateEntity>

    @Query("select min(rate_date) from exchange_rates")
    suspend fun getEarliestRateDate(): String?

    @Insert(onConflict = REPLACE)
    suspend fun upsert(rate: ExchangeRateEntity)

    @Insert(onConflict = REPLACE)
    suspend fun upsertAll(rates: List<ExchangeRateEntity>)

    @Query("delete from exchange_rates")
    suspend fun clear()
}
