package dev.crqch.sunder.ui.categories

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.navigation.toRoute
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.crqch.sunder.data.local.CategoryEntity
import dev.crqch.sunder.data.local.EntryEntity
import dev.crqch.sunder.data.repositories.CategoryRepository
import dev.crqch.sunder.data.repositories.EntryRepository
import dev.crqch.sunder.ui.SubRoute
import dev.crqch.sunder.ui.entries.EntryFormState
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject
import kotlin.math.abs


data class CategoryFormState(
    val title: String = "",
    val description: String = "",
    val color: String = "#f3f3f3"


) {
    companion object {
        fun ofCategory(category: CategoryEntity): CategoryFormState = CategoryFormState(
            category.title,
            category.description,
            category.color
        )

    }

    fun isFilled(): Boolean {
        if (title.isBlank()) return false
        return true
    }

    fun toCategory(id: String, createdAt: Long, updatedAt: Long): CategoryEntity = CategoryEntity(
        id = id,
        title = title,
        description = description,
        color = color,
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


    fun updateForm(state: CategoryFormState) {
        categoryFormState = state
    }

    var categoryFormState by mutableStateOf(CategoryFormState())
        private set

    init {
        categoryId?.let { id ->
            viewModelScope.launch {
                categoryRepository.getCategory(id).collect { entity ->
                    entity?.let {
                        categoryFormState = CategoryFormState.ofCategory(it)
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

    fun delete(onComplete: () -> Unit) {
        categoryId?.let { id ->
            viewModelScope.launch {
                categoryRepository.deleteCategory(id)
                onComplete()
            }
        }
    }

    fun toggleEditMode() {
        viewModelScope.launch {
            categoryRepository.saveCategory(categoryFormState, categoryId)
        }
        isEditMode = !isEditMode
    }
}