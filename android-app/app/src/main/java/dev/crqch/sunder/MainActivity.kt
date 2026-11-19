package dev.crqch.sunder

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.animation.Crossfade
import androidx.compose.foundation.background
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AccountBalanceWallet
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import dagger.hilt.android.AndroidEntryPoint
import dev.crqch.sunder.data.repositories.ThemeMode
import dev.crqch.sunder.notifications.PaymentEntryDraft
import dev.crqch.sunder.notifications.PaymentNotificationIntent
import dev.crqch.sunder.notifications.PaymentPromptNotifier
import dev.crqch.sunder.ui.SunderApp
import dev.crqch.sunder.ui.auth.AuthViewModel
import dev.crqch.sunder.ui.theme.SunderMotion
import dev.crqch.sunder.ui.theme.SunderTheme
import dev.crqch.sunder.utils.CurrencyFormatConfig
import dev.crqch.sunder.utils.LocalCurrencyConfig
import javax.inject.Inject

@AndroidEntryPoint
class MainActivity : ComponentActivity() {

    @Inject
    lateinit var paymentPromptNotifier: PaymentPromptNotifier

    private var pendingPaymentDraft by mutableStateOf<PaymentEntryDraft?>(null)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        handleIncomingIntent(intent)
        enableEdgeToEdge()
        setContent {
            RootContainer(
                pendingPaymentDraft = pendingPaymentDraft,
                onConsumePaymentDraft = { draft ->
                    draft.notificationId?.let { paymentPromptNotifier.cancelPrompt(it) }
                    if (pendingPaymentDraft?.draftId == draft.draftId) {
                        pendingPaymentDraft = null
                    }
                }
            )
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        handleIncomingIntent(intent)
    }

    private fun handleIncomingIntent(intent: Intent?) {
        val draft = PaymentNotificationIntent.extractDraft(intent) ?: return
        draft.notificationId?.let { paymentPromptNotifier.cancelPrompt(it) }
        pendingPaymentDraft = draft
        PaymentNotificationIntent.clearDraftExtras(intent)
    }
}

@Composable
fun RootContainer(
    pendingPaymentDraft: PaymentEntryDraft? = null,
    onConsumePaymentDraft: (PaymentEntryDraft) -> Unit = {},
    viewModel: AuthViewModel = hiltViewModel()
) {
    val isInitializing by viewModel.isInitializing.collectAsState()
    val user by viewModel.currentUser.collectAsState()
    val settings by viewModel.appSettings.collectAsState()

    val darkTheme = when (settings.themeMode) {
        ThemeMode.SYSTEM -> isSystemInDarkTheme()
        ThemeMode.LIGHT -> false
        ThemeMode.DARK -> true
    }

    val currencyConfig = CurrencyFormatConfig(
        symbol = settings.currencySymbol,
        after = settings.currencyAfter
    )

    SunderTheme(
        darkTheme = darkTheme,
        dynamicColor = settings.dynamicColor
    ) {
        CompositionLocalProvider(LocalCurrencyConfig provides currencyConfig) {
            Surface(
                modifier = Modifier.fillMaxSize(),
                color = MaterialTheme.colorScheme.background
            ) {
                Crossfade(
                    targetState = isInitializing,
                    animationSpec = SunderMotion.enterTween(),
                    label = "root_init_crossfade"
                ) { loading ->
                    if (loading) {
                        Column(
                            modifier = Modifier.fillMaxSize(),
                            verticalArrangement = Arrangement.Center,
                            horizontalAlignment = Alignment.CenterHorizontally
                        ) {
                            Box(
                                modifier = Modifier
                                    .size(72.dp)
                                    .clip(RoundedCornerShape(22.dp))
                                    .background(
                                        Brush.linearGradient(
                                            colors = listOf(
                                                MaterialTheme.colorScheme.primary,
                                                MaterialTheme.colorScheme.tertiary
                                            )
                                        )
                                    ),
                                contentAlignment = Alignment.Center
                            ) {
                                Icon(
                                    imageVector = Icons.Default.AccountBalanceWallet,
                                    contentDescription = null,
                                    tint = MaterialTheme.colorScheme.onPrimary,
                                    modifier = Modifier.size(36.dp)
                                )
                            }
                            Spacer(modifier = Modifier.height(16.dp))
                            Text(
                                text = stringResource(R.string.app_name),
                                style = MaterialTheme.typography.headlineMedium,
                                fontWeight = FontWeight.Bold,
                                color = MaterialTheme.colorScheme.onBackground
                            )
                            Spacer(modifier = Modifier.height(24.dp))
                            CircularProgressIndicator(
                                modifier = Modifier.size(28.dp),
                                strokeWidth = 3.dp,
                                color = MaterialTheme.colorScheme.primary
                            )
                        }
                    } else {
                        SunderApp(
                            authViewModel = viewModel,
                            user = user,
                            pendingPaymentDraft = pendingPaymentDraft,
                            onConsumePaymentDraft = onConsumePaymentDraft
                        )
                    }
                }
            }
        }
    }
}
