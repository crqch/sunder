package dev.crqch.sunder.ui.categories

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.navigation.toRoute
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.crqch.sunder.data.local.CategoryEntity
import dev.crqch.sunder.data.repositories.CategoryRepository
import dev.crqch.sunder.data.repositories.EntryRepository
import dev.crqch.sunder.ui.SubRoute
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

data class CategoryFormState(
    val title: String = "",
    val description: String = "",
    val color: String = "#6366f1"
) {
    companion object {
        fun ofCategory(category: CategoryEntity): CategoryFormState = CategoryFormState(
            title = category.title,
            description = category.description,
            color = category.color.ifBlank { "#6366f1" }
        )
    }

    fun isFilled(): Boolean {
        return title.isNotBlank()
    }

    fun toCategory(id: String, createdAt: Long, updatedAt: Long): CategoryEntity = CategoryEntity(
        id = id,
        title = title.trim(),
        description = description.trim(),
        color = color.ifBlank { "#6366f1" },
        createdAt = createdAt,
        updatedAt = updatedAt
    )
}

@HiltViewModel
class CategoryViewModel @Inject constructor(
    savedStateHandle: SavedStateHandle,
    private val categoryRepository: CategoryRepository,
    private val entryRepository: EntryRepository
) : ViewModel() {

    var isEditMode by mutableStateOf(false)
        private set

    private val categoryRoute = try {
        savedStateHandle.toRoute<SubRoute.Category>()
    } catch (_: Exception) {
        null
    }

    val categoryId = categoryRoute?.categoryId

    var categoryFormState by mutableStateOf(CategoryFormState())
        private set

    fun updateForm(state: CategoryFormState) {
        categoryFormState = state
    }

    init {
        categoryId?.let { id ->
            viewModelScope.launch {
                categoryRepository.getCategory(id).collect { entity ->
                    entity?.let {
                        if (!isEditMode) {
                            categoryFormState = CategoryFormState.ofCategory(it)
                        }
                    }
                }
            }
        }
    }

    @OptIn(ExperimentalCoroutinesApi::class)
    val category: StateFlow<CategoryEntity?> = flowOf(categoryId).flatMapLatest { id ->
        if (id != null) categoryRepository.getCategory(id) else flowOf(null)
    }.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = null
    )

    @OptIn(ExperimentalCoroutinesApi::class)
    val entries = flowOf(categoryId).flatMapLatest { id ->
        entryRepository.getEntries(null, id)
    }.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = listOf()
    )

    fun startEdit() {
        category.value?.let {
            categoryFormState = CategoryFormState.ofCategory(it)
        }
        isEditMode = true
    }

    fun cancelEdit() {
        category.value?.let {
            categoryFormState = CategoryFormState.ofCategory(it)
        }
        isEditMode = false
    }

    fun saveEdit() {
        if (!categoryFormState.isFilled()) return
        viewModelScope.launch {
            categoryRepository.saveCategory(categoryFormState, categoryId)
            isEditMode = false
        }
    }

    fun delete(onComplete: () -> Unit) {
        categoryId?.let { id ->
            viewModelScope.launch {
                categoryRepository.deleteCategory(id)
                onComplete()
            }
        }
    }
}