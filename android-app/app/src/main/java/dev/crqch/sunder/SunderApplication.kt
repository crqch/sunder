package dev.crqch.sunder

import android.app.Application
import androidx.hilt.work.HiltWorkerFactory
import androidx.work.Configuration
import dagger.hilt.android.HiltAndroidApp
import dev.crqch.sunder.notifications.PaymentPromptNotifier
import javax.inject.Inject

@HiltAndroidApp
class SunderApplication : Application(), Configuration.Provider {
    @Inject
    lateinit var workerFactory: HiltWorkerFactory

    @Inject
    lateinit var paymentPromptNotifier: PaymentPromptNotifier

    override fun onCreate() {
        super.onCreate()
        instance = this
        paymentPromptNotifier.ensureChannel()
    }

    override val workManagerConfiguration: Configuration
        get() = Configuration.Builder()
            .setWorkerFactory(workerFactory)
            .build()

    companion object {
        lateinit var instance: SunderApplication
            private set
    }
}
