package dev.crqch.sunder.notifications

import android.Manifest
import android.app.NotificationManager
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.provider.Settings
import androidx.core.app.NotificationManagerCompat
import androidx.core.content.ContextCompat

/**
 * Checks and launches Android system permission screens for reading
 * notifications ([PaymentNotificationListenerService]) and posting prompts.
 */
object PaymentNotificationAccess {

    fun listenerComponentName(context: Context): ComponentName =
        ComponentName(context, PaymentNotificationListenerService::class.java)

    fun isNotificationListenerEnabled(context: Context): Boolean {
        val component = listenerComponentName(context)
        val notificationManager = context.getSystemService(NotificationManager::class.java)
        val grantedViaManager = runCatching {
            notificationManager?.isNotificationListenerAccessGranted(component) == true
        }.getOrDefault(false)
        if (grantedViaManager) return true

        val enabledPackages = NotificationManagerCompat.getEnabledListenerPackages(context)
        if (context.packageName in enabledPackages) return true

        val flat = Settings.Secure.getString(
            context.contentResolver,
            "enabled_notification_listeners"
        ).orEmpty()
        return flat.split(':').any { entry ->
            ComponentName.unflattenFromString(entry) == component
        }
    }

    fun canPostNotifications(context: Context): Boolean {
        val permissionGranted = ContextCompat.checkSelfPermission(
            context,
            Manifest.permission.POST_NOTIFICATIONS
        ) == PackageManager.PERMISSION_GRANTED
        return permissionGranted && NotificationManagerCompat.from(context).areNotificationsEnabled()
    }

    fun openNotificationListenerSettings(context: Context) {
        val component = listenerComponentName(context)
        val detailIntent = Intent(Settings.ACTION_NOTIFICATION_LISTENER_DETAIL_SETTINGS).apply {
            putExtra(
                Settings.EXTRA_NOTIFICATION_LISTENER_COMPONENT_NAME,
                component.flattenToString()
            )
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        }
        val launchedDetail = runCatching {
            context.startActivity(detailIntent)
        }.isSuccess
        if (!launchedDetail) {
            val fallback = Intent(Settings.ACTION_NOTIFICATION_LISTENER_SETTINGS).apply {
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            }
            runCatching { context.startActivity(fallback) }
        }
    }

    fun openAppNotificationSettings(context: Context) {
        val intent = Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS).apply {
            putExtra(Settings.EXTRA_APP_PACKAGE, context.packageName)
            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        }
        runCatching { context.startActivity(intent) }
    }
}
