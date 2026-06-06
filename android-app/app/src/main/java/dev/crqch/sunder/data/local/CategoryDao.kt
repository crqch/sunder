package dev.crqch.sunder.data.local

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy.Companion.REPLACE
import androidx.room.Query
import kotlinx.coroutines.flow.Flow

@Dao
interface CategoryDao {
    @Query("SELECT * from categories where deletedAt is null")
    fun getCategories(): Flow<List<CategoryEntity>>

    @Query("select * from categories where deletedAt is null and title like :title")
    fun findCategoryByTitle(title: String): Flow<List<CategoryEntity>>

    @Query("select * from categories where deletedAt is null and id = :id")
    fun getCategory(id: String): Flow<CategoryEntity?>

    @Insert
    suspend fun insert(categoryEntity: CategoryEntity)
}
