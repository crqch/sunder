package dev.crqch.sunder.data.local

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy.Companion.REPLACE
import androidx.room.Query
import androidx.room.Upsert
import kotlinx.coroutines.flow.Flow

@Dao
interface CategoryDao {
    @Query("SELECT * from categories where deletedAt is null")
    fun getCategories(): Flow<List<CategoryEntity>>

    @Query("select * from categories where deletedAt is null and title like :title")
    fun findCategoryByTitle(title: String): Flow<List<CategoryEntity>>

    @Query("select * from categories where deletedAt is null and id = :id")
    fun getCategory(id: String): Flow<CategoryEntity?>

    @Query("select * from categories where deletedAt is null and id = :id")
    suspend fun getCategoryByIdDirect(id: String): CategoryEntity?

    @Insert
    suspend fun insert(categoryEntity: CategoryEntity)

    @Upsert
    suspend fun upsert(categoryEntity: CategoryEntity)

    @Query("update categories set deletedAt = :timestamp, updatedAt = :timestamp where id = :id")
    suspend fun softDelete(id: String, timestamp: Long = System.currentTimeMillis())

    @Query("select * from categories where updatedAt > :timestamp")
    suspend fun getModifiedSince(timestamp: Long): List<CategoryEntity>

    @Upsert
    suspend fun upsertAll(categories: List<CategoryEntity>)
}
