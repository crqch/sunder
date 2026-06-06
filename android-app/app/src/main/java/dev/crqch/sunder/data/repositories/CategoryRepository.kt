package dev.crqch.sunder.data.repositories

import dev.crqch.sunder.data.local.CategoryDao
import dev.crqch.sunder.data.local.CategoryEntity
import dev.crqch.sunder.utils.Cuid2
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject
import javax.inject.Singleton


@Singleton
class CategoryRepository @Inject constructor(private val categoryDao: CategoryDao) {

    val allCategories: Flow<List<CategoryEntity>> = categoryDao.getCategories()

    suspend fun createCategory(title: String, description: String, color: String) {
        val now = System.currentTimeMillis()
        val category = CategoryEntity(
            id = Cuid2.generate(),
            title = title,
            description = description,
            color = color,
            createdAt = now,
            updatedAt = now
        )
        categoryDao.insert(category)
    }

    suspend fun getCategory(id: String): Flow<CategoryEntity?> = categoryDao.getCategory(id)
}