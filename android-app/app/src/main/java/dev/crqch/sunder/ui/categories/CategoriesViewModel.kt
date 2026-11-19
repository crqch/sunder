package dev.crqch.sunder.ui.categories

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.crqch.sunder.data.local.CategoryEntity
import dev.crqch.sunder.data.repositories.CategoryRepository
import dev.crqch.sunder.data.repositories.EntryRepository
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

data class CategoryWithStats(
    val category: CategoryEntity,
    val entryCount: Int,
    val totalAmount: Float
)

@HiltViewModel
class CategoriesViewModel @Inject constructor(
    private val categoryRepository: CategoryRepository,
    private val entryRepository: EntryRepository
) : ViewModel() {

    private val _searchQuery = MutableStateFlow("")
    val searchQuery: StateFlow<String> = _searchQuery.asStateFlow()

    val allCategoriesWithStats: StateFlow<List<CategoryWithStats>> = combine(
        categoryRepository.allCategories,
        entryRepository.allEntries
    ) { cats, entries ->
        val byCat = entries.groupBy { it.entry.categoryId }
        cats.map { cat ->
            val catEntries = byCat[cat.id].orEmpty()
            CategoryWithStats(
                category = cat,
                entryCount = catEntries.size,
                totalAmount = catEntries.sumOf { it.entry.amount.toDouble() }.toFloat()
            )
        }
    }.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = emptyList()
    )

    val categories: StateFlow<List<CategoryWithStats>> = combine(
        allCategoriesWithStats,
        _searchQuery
    ) { list, query ->
        if (query.isBlank()) list
        else list.filter {
            it.category.title.contains(query.trim(), ignoreCase = true) ||
                    it.category.description.contains(query.trim(), ignoreCase = true)
        }
    }.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = emptyList()
    )

    fun updateSearchQuery(query: String) {
        _searchQuery.value = query
    }

    fun createCategory(formState: CategoryFormState, onComplete: () -> Unit) {
        viewModelScope.launch {
            categoryRepository.saveCategory(formState, null)
            onComplete()
        }
    }

    fun deleteCategories(
        ids: List<String>,
        onBlockedByEntries: (blockedCount: Int) -> Unit = {},
        onComplete: () -> Unit = {}
    ) {
        viewModelScope.launch {
            val deletable = mutableListOf<String>()
            var blocked = 0
            for (id in ids) {
                val activeCount = entryRepository.countActiveByCategoryId(id)
                if (activeCount > 0) {
                    blocked++
                } else {
                    deletable.add(id)
                }
            }
            if (deletable.isNotEmpty()) {
                categoryRepository.deleteCategories(deletable)
            }
            if (blocked > 0) {
                onBlockedByEntries(blocked)
            }
            onComplete()
        }
    }
}