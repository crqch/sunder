package dev.crqch.sunder.data.local

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy.Companion.REPLACE
import androidx.room.Query
import kotlinx.coroutines.flow.Flow

@Dao
interface CurrencyDao {

    @Query("select * from currencies where is_active = 1 order by code")
    fun observeActiveCurrencies(): Flow<List<CurrencyEntity>>

    @Query("select * from currencies order by code")
    fun observeAllCurrencies(): Flow<List<CurrencyEntity>>

    @Query("select * from currencies where is_active = 1 order by code")
    suspend fun getActiveCurrencies(): List<CurrencyEntity>

    @Query("select * from currencies order by code")
    suspend fun getAllCurrencies(): List<CurrencyEntity>

    @Query("select * from currencies where code = :code limit 1")
    suspend fun getCurrency(code: String): CurrencyEntity?

    @Insert(onConflict = REPLACE)
    suspend fun upsert(currency: CurrencyEntity)

    @Insert(onConflict = REPLACE)
    suspend fun upsertAll(currencies: List<CurrencyEntity>)

    @Query("delete from currencies")
    suspend fun clear()

    @Query("select count(*) from currencies")
    suspend fun count(): Int
}
