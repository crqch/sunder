package dev.crqch.sunder.data.local

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy.Companion.REPLACE
import androidx.room.Query
import androidx.room.Transaction
import kotlinx.coroutines.flow.Flow

@Dao
interface EntryDao {

    @Transaction
    @Query(
        """
        select * from entries
        where deletedAt is null
        and id = :id
    """
    )
    fun getEntryWithDetailsById(id: String): Flow<EntryWithDetails?>

    @Transaction
    @Query(
        """
        select * from entries 
        where deletedAt is null 
        and (:query is null or title like '%' || :query || '%' or description like '%' || :query || '%' or location like '%' || :query || '%')
        order by date desc
    """
    )
    fun getAllEntries(query: String?): Flow<List<EntryWithDetails>>

    @Query("select * from entries where deletedAt is null and id = :id")
    fun getEntryById(id: String): Flow<EntryEntity?>

    @Query("select * from entries where deletedAt is null and id = :id")
    suspend fun getEntryByIdDirect(id: String): EntryEntity?

    @Transaction
    @Query(
        """
        select * from entries 
        where accountId = :accountId 
        and deletedAt is null
        and (:query is null or title like '%' || :query || '%' or description like '%' || :query || '%' or location like '%' || :query || '%')
        order by date desc
    """
    )
    fun getEntriesByAccountId(accountId: String, query: String?): Flow<List<EntryWithDetails>>

    @Transaction
    @Query(
        """
        select * from entries 
        where categoryId = :categoryId 
        and deletedAt is null
        and (:query is null or title like '%' || :query || '%' or description like '%' || :query || '%' or location like '%' || :query || '%')
        order by date desc
    """
    )
    fun getEntriesByCategoryId(categoryId: String, query: String?): Flow<List<EntryWithDetails>>

    @Transaction
    @Query(
        """
        select * from entries 
        where accountId = :accountId 
        and categoryId = :categoryId 
        and deletedAt is null
        and (:query is null or title like '%' || :query || '%' or description like '%' || :query || '%' or location like '%' || :query || '%')
        order by date desc
    """
    )
    fun getEntriesByAccountAndCategoryId(
        accountId: String,
        categoryId: String,
        query: String?
    ): Flow<List<EntryWithDetails>>

    @Insert(onConflict = REPLACE)
    suspend fun upsert(entryEntity: EntryEntity)

    @Insert(onConflict = REPLACE)
    suspend fun upsertAll(entries: List<EntryEntity>)

    @Query("update entries set deletedAt = :timestamp, updatedAt = :timestamp where id = :id")
    suspend fun softDelete(id: String, timestamp: Long = System.currentTimeMillis())

    @Query("update entries set deletedAt = :timestamp, updatedAt = :timestamp where id in (:ids)")
    suspend fun softDeleteAll(ids: List<String>, timestamp: Long = System.currentTimeMillis())

    @Query("select count(*) from entries where categoryId = :categoryId and deletedAt is null")
    suspend fun countActiveByCategoryId(categoryId: String): Int

    @Query("select * from entries where updatedAt > :timestamp")
    suspend fun getModifiedSince(timestamp: Long): List<EntryEntity>

    @Query("update entries set currency_code = :currencyCode, updatedAt = :timestamp where id in (:ids)")
    suspend fun updateCurrency(
        ids: List<String>,
        currencyCode: String,
        timestamp: Long = System.currentTimeMillis()
    )

    @Query("select min(date) from entries where deletedAt is null")
    suspend fun getEarliestActiveEntryDate(): Long?

    @Query("select distinct currency_code from entries where deletedAt is null")
    suspend fun getActiveCurrencyCodes(): List<String>
}
