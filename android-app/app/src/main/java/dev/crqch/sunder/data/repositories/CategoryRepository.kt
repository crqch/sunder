package dev.crqch.sunder.data.repositories

import androidx.room.Query
import dev.crqch.sunder.data.local.CategoryDao
import dev.crqch.sunder.data.local.CategoryEntity
import dev.crqch.sunder.data.local.EntryEntity
import dev.crqch.sunder.ui.categories.CategoryFormState
import dev.crqch.sunder.utils.Cuid2
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject
import javax.inject.Singleton


@Singleton
class CategoryRepository @Inject constructor(private val categoryDao: CategoryDao) {

    val allCategories: Flow<List<CategoryEntity>> = categoryDao.getCategories()


    suspend fun getCategory(id: String): Flow<CategoryEntity?> = categoryDao.getCategory(id)


    suspend fun saveCategory(state: CategoryFormState, id: String?) {
        val now = System.currentTimeMillis()

        if (id == null) {
            val entry = state.toCategory(
                id = Cuid2.generate(),
                createdAt = now,
                updatedAt = now
            )
            categoryDao.upsert(entry)
        } else {
            val existing = categoryDao.getCategoryByIdDirect(id)
            val updated = state.toCategory(
                id = id,
                createdAt = existing?.createdAt ?: now,
                updatedAt = now
            )
            categoryDao.upsert(updated)
        }
    }

    suspend fun deleteCategory(id: String) {
        categoryDao.softDelete(id)
    }
}