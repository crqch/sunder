package dev.crqch.sunder.ui.categories

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.crqch.sunder.data.local.AccountWithBalance
import dev.crqch.sunder.data.local.CategoryEntity
import dev.crqch.sunder.data.repositories.CategoryRepository
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject


@HiltViewModel
class CategoriesViewModel @Inject constructor(private val categoryRepository: CategoryRepository) :
    ViewModel() {

    val categories: StateFlow<List<CategoryEntity>> =
        categoryRepository.allCategories.stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = emptyList()
        )


    fun createCategory(formState: CategoryFormState, onComplete: () -> Unit) {
        viewModelScope.launch {
            categoryRepository.saveCategory(
                formState, null
            )
            onComplete()
        }
    }
}