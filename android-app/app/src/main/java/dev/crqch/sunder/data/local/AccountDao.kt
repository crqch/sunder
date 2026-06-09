package dev.crqch.sunder.data.local

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.Query
import androidx.room.Upsert
import kotlinx.coroutines.flow.Flow

@Dao
interface AccountDao {
    @Query("select * from accounts where deletedAt is null")
    fun getAccounts(): Flow<List<AccountEntity>>

    @Query(
        """
        select accounts.*, TOTAL(entries.amount) as balance
        from accounts
        left join entries on accounts.id = entries.accountId and entries.deletedAt is null
        where accounts.deletedAt is null
        group by accounts.id
    """
    )
    fun getAccountsWithBalance(): Flow<List<AccountWithBalance>>

    @Insert
    suspend fun insert(accountEntity: AccountEntity)

    @Upsert
    suspend fun upsert(accountEntity: AccountEntity)

    @Query("select * from accounts where deletedAt is null and id = :id")
    suspend fun getAccountById(id: String): AccountEntity?

    @Query("select * from accounts where deletedAt is null and id = :id")
    fun getAccount(id: String): Flow<AccountEntity?>


    @Query("update accounts set deletedAt = :timestamp, updatedAt = :timestamp where id = :id")
    suspend fun softDelete(id: String, timestamp: Long = System.currentTimeMillis())
}