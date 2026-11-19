package dev.crqch.sunder.ui.theme

import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.tween
import androidx.compose.ui.unit.IntOffset

object SunderMotion {
    val EmphasizedDecelerate = CubicBezierEasing(0.05f, 0.7f, 0.1f, 1.0f)
    val EmphasizedAccelerate = CubicBezierEasing(0.3f, 0.0f, 0.8f, 0.15f)
    val StandardEasing = CubicBezierEasing(0.2f, 0.0f, 0.0f, 1.0f)

    const val DURATION_SHORT = 180
    const val DURATION_MEDIUM = 280
    const val DURATION_LONG = 380

    fun <T> enterTween(durationMillis: Int = DURATION_MEDIUM) = tween<T>(
        durationMillis = durationMillis,
        easing = EmphasizedDecelerate
    )

    fun <T> exitTween(durationMillis: Int = DURATION_SHORT) = tween<T>(
        durationMillis = durationMillis,
        easing = EmphasizedAccelerate
    )

    fun offsetTween(durationMillis: Int = DURATION_MEDIUM) = tween<IntOffset>(
        durationMillis = durationMillis,
        easing = EmphasizedDecelerate
    )
}
