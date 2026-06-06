package dev.crqch.sunder.ui.categories

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.CenterAlignedTopAppBar
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import dev.crqch.sunder.R
import dev.crqch.sunder.ui.components.EntriesList

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CategoryScreen(
    onNavigateBack: () -> Unit,
    onEntryClick: (String) -> Unit,
    viewModel: CategoryViewModel = hiltViewModel(),
) {
    val category by viewModel.category.collectAsState()
    val entries by viewModel.entries.collectAsState()

    category?.let { category ->

        Scaffold(
            Modifier,
            topBar = {
                CenterAlignedTopAppBar({ Text(category.title) }, Modifier, navigationIcon = {
                    IconButton(onClick = onNavigateBack) {
                        Icon(
                            Icons.AutoMirrored.Filled.ArrowBack,
                            contentDescription = stringResource(
                                R.string.back
                            )
                        )
                    }
                })
            }
        ) { padding ->
            Column(
                Modifier
                    .padding(padding)
            ) {
                Text(
                    "Entries",
                    Modifier.padding(horizontal = 24.dp),
                    style = MaterialTheme.typography.titleLarge
                )
                EntriesList(entries = entries, onEntrySelect = { entryId ->
                    onEntryClick(entryId)
                }, Modifier, null, category.id)
            }
        }
    }
}
