package dev.crqch.sunder.data.sync

import com.squareup.moshi.Json
import dev.crqch.sunder.data.local.AccountEntity
import dev.crqch.sunder.data.local.CategoryEntity
import dev.crqch.sunder.data.local.EntryEntity
import java.time.Instant
import java.time.ZoneOffset
import java.time.format.DateTimeFormatter

private val formatter = DateTimeFormatter.ISO_INSTANT.withZone(ZoneOffset.UTC)

private fun safeParseDate(dateStr: String): Long {
    val withZone = if (dateStr.endsWith("Z")) dateStr else "${dateStr}Z"
    return Instant.parse(withZone).toEpochMilli()
}


data class SyncRequestDto(
    @Json(name = "accounts") val accounts: List<AccountSyncDto>,
    @Json(name = "categories") val categories: List<CategorySyncDto>,
    @Json(name = "entries") val entries: List<EntrySyncDto>
)

data class SyncResponseDto(
    @Json(name = "accounts") val accounts: List<AccountSyncDto>,
    @Json(name = "categories") val categories: List<CategorySyncDto>,
    @Json(name = "entries") val entries: List<EntrySyncDto>
)

data class AccountSyncDto(
    @Json(name = "id") val id: String,
    @Json(name = "name") val name: String,
    @Json(name = "updated_at") val updatedAt: String,
    @Json(name = "inserted_at") val createdAt: String,
    @Json(name = "deleted_at") val deletedAt: String? = null
)

data class CategorySyncDto(
    @Json(name = "id") val id: String,
    @Json(name = "title") val title: String,
    @Json(name = "description") val description: String? = null,
    @Json(name = "color") val color: String,
    @Json(name = "updated_at") val updatedAt: String,
    @Json(name = "inserted_at") val createdAt: String,
    @Json(name = "deleted_at") val deletedAt: String? = null
)

data class EntrySyncDto(
    @Json(name = "id") val id: String,
    @Json(name = "title") val title: String,
    @Json(name = "date") val date: String,
    @Json(name = "location") val location: String? = null,
    @Json(name = "amount") val amount: Float,
    @Json(name = "account_id") val accountId: String,
    @Json(name = "category_id") val categoryId: String,
    @Json(name = "description") val description: String? = null,
    @Json(name = "updated_at") val updatedAt: String,
    @Json(name = "inserted_at") val createdAt: String,
    @Json(name = "deleted_at") val deletedAt: String? = null
)

fun AccountEntity.toDto() = AccountSyncDto(
    id = id,
    name = name,
    updatedAt = formatter.format(Instant.ofEpochMilli(updatedAt)),
    createdAt = formatter.format(Instant.ofEpochMilli(createdAt)),
    deletedAt = deletedAt?.let { formatter.format(Instant.ofEpochMilli(it)) }
)

fun CategoryEntity.toDto() = CategorySyncDto(
    id = id,
    title = title,
    description = description,
    color = color,
    updatedAt = formatter.format(Instant.ofEpochMilli(updatedAt)),
    createdAt = formatter.format(Instant.ofEpochMilli(createdAt)),
    deletedAt = deletedAt?.let { formatter.format(Instant.ofEpochMilli(it)) }
)

fun EntryEntity.toDto() = EntrySyncDto(
    id = id,
    title = title,
    date = formatter.format(Instant.ofEpochMilli(date)),
    location = location,
    amount = amount,
    accountId = accountId,
    categoryId = categoryId,
    description = description,
    updatedAt = formatter.format(Instant.ofEpochMilli(updatedAt)),
    createdAt = formatter.format(Instant.ofEpochMilli(createdAt)),
    deletedAt = deletedAt?.let { formatter.format(Instant.ofEpochMilli(it)) }
)

fun AccountSyncDto.toEntity() = AccountEntity(
    id = id,
    name = name,
    updatedAt = safeParseDate(updatedAt),
    createdAt = safeParseDate(createdAt),
    deletedAt = deletedAt?.let { safeParseDate(it) }
)

fun CategorySyncDto.toEntity() = CategoryEntity(
    id = id,
    title = title,
    description = description ?: "",
    color = color,
    updatedAt = safeParseDate(updatedAt),
    createdAt = safeParseDate(createdAt),
    deletedAt = deletedAt?.let { safeParseDate(it) }
)

fun EntrySyncDto.toEntity() = EntryEntity(
    id = id,
    title = title,
    date = safeParseDate(date),
    location = location ?: "",
    amount = amount,
    accountId = accountId,
    categoryId = categoryId,
    description = description ?: "",
    updatedAt = safeParseDate(updatedAt),
    createdAt = safeParseDate(createdAt),
    deletedAt = deletedAt?.let { safeParseDate(it) }
)
