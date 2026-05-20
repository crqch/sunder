package dev.crqch.sunder.data.local

import androidx.room.Dao
import androidx.room.Query
import kotlinx.coroutines.flow.Flow

@Dao
interface CategoryDao {
    @Query("SELECT * from categories")
    fun getCategories(): Flow<List<CategoryEntity>>

    @Query("select * from categories where title like :title")
    fun findCategoryByTitle(title: String): Flow<List<CategoryEntity>>
}
