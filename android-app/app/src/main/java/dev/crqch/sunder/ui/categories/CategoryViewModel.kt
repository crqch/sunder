package dev.crqch.sunder.ui.categories

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.collectAsState
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
import javax.inject.Inject

@HiltViewModel
class CategoryViewModel @Inject constructor(
    savedStateHandle: SavedStateHandle,
    private val categoryRepository: CategoryRepository,
    private val entryRepository: EntryRepository
) : ViewModel() {

    private val categoryRoute = try {
        savedStateHandle.toRoute<SubRoute.Category>()
    } catch (_: Exception) {
        null
    }

    val categoryId = categoryRoute?.categoryId

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
}