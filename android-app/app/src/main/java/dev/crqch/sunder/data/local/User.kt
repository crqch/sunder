package dev.crqch.sunder.data.local

import kotlinx.serialization.Serializable

@Serializable
data class User(
    val username: String,
    val email: String,
    val flags: List<String>
)
