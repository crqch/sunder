package dev.crqch.sunder.data.local

import androidx.room.Embedded

data class AccountWithBalance(
    @Embedded val account: AccountEntity,
    val balance: Float
)
