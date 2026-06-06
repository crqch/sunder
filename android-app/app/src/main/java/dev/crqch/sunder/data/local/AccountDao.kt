package dev.crqch.sunder.data.local

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.Query
import kotlinx.coroutines.flow.Flow

@Dao
interface AccountDao {
    @Query("SELECT * from accounts where deletedAt is null")
    fun getAccounts(): Flow<List<AccountEntity>>

    @Query(
        """
        SELECT accounts.*, TOTAL(entries.amount) as balance
        FROM accounts
        LEFT JOIN entries ON accounts.id = entries.accountId AND entries.deletedAt IS NULL
        where accounts.deletedAt is null
        GROUP BY accounts.id
    """
    )
    fun getAccountsWithBalance(): Flow<List<AccountWithBalance>>

    @Insert
    suspend fun insertAccount(accountEntity: AccountEntity)
}