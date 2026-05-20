package dev.crqch.sunder.utils

import java.math.BigInteger
import java.security.MessageDigest
import java.security.SecureRandom
import java.util.concurrent.atomic.AtomicLong
import kotlin.random.asKotlinRandom

object Cuid2 {
    private val secureRandom = SecureRandom()
    private val kotlinRandom = secureRandom.asKotlinRandom()
    private val counter = AtomicLong(secureRandom.nextLong(0, 1_000_000_000L))

    private val fingerprint: String by lazy {
        val randomBytes = ByteArray(32)
        secureRandom.nextBytes(randomBytes)
        hash(randomBytes.joinToString("") { "%02x".format(it) }).take(16)
    }

    private fun hash(input: String): String {
        val algorithm = try {
            MessageDigest.getInstance("SHA3-256")
            "SHA3-256"
        } catch (e: Exception) {
            "SHA-256"
        }
        val digest = MessageDigest.getInstance(algorithm)
        val hashBytes = digest.digest(input.toByteArray())
        return BigInteger(1, hashBytes).toString(36)
    }

    fun generate(length: Int = 24): String {
        val timestamp = System.currentTimeMillis().toString(36)
        val count = counter.getAndIncrement().toString(36)

        val entropy = StringBuilder()
        while (entropy.length < length) {
            val randomBytes = ByteArray(32)
            secureRandom.nextBytes(randomBytes)
            entropy.append(BigInteger(1, randomBytes).toString(36))
        }

        val input = timestamp + entropy.toString() + count + fingerprint
        val hashed = hash(input)

        val firstLetter = ('a'..'z').random(kotlinRandom)
        return (firstLetter + hashed).take(length)
    }
}

