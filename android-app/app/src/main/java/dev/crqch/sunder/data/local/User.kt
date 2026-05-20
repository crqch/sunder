package dev.crqch.sunder.data.local

data class User(
    val username: String,
    val email: String,
    val flags: List<String>
)
