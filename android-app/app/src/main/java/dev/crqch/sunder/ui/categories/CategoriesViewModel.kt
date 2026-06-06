package dev.crqch.sunder.ui.categories

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import dev.crqch.sunder.data.repositories.CategoryRepository
import kotlinx.coroutines.launch
import javax.inject.Inject


@HiltViewModel
class CategoriesViewModel @Inject constructor(private val categoryRepository: CategoryRepository) :
    ViewModel() {

    fun createCategory(formState: CategoryFormState, onComplete: () -> Unit) {
        viewModelScope.launch {
            categoryRepository.createCategory(
                formState.name,
                formState.description,
                formState.color
            )
            onComplete()
        }
    }
}