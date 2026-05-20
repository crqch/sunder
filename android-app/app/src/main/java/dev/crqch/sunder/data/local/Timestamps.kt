package dev.crqch.sunder.data.local

interface Timestamps {
    val updatedAt: Long
    val createdAt: Long
    val deletedAt: Long?
}
