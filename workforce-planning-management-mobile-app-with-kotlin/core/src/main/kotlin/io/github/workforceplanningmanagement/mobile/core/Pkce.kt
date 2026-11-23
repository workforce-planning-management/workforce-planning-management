// PKCE (RFC 7636, S256) for the native sign-in (RFC 8252, WPM-R281). Randomness is passed in, so the core stays pure.
package io.github.workforceplanningmanagement.mobile.core

import java.security.MessageDigest
import java.util.Base64

/** Proof Key for Code Exchange. */
public object Pkce {
    /** The number of random bytes behind a verifier or a state: 32 bytes give a 43-character verifier. */
    public const val RANDOM_BYTES: Int = 32

    private val encoder = Base64.getUrlEncoder().withoutPadding()

    /** A verifier from [random] bytes (at least 32), base64url without padding. */
    public fun verifier(random: ByteArray): String {
        require(random.size >= RANDOM_BYTES) { "a verifier needs at least $RANDOM_BYTES random bytes" }
        return encoder.encodeToString(random)
    }

    /** The S256 challenge for [verifier]: base64url(SHA-256(ASCII(verifier))) without padding. */
    public fun challenge(verifier: String): String {
        require(isVerifier(verifier)) { "not a PKCE verifier" }
        val digest = MessageDigest.getInstance("SHA-256").digest(verifier.toByteArray(Charsets.US_ASCII))
        return encoder.encodeToString(digest)
    }

    /** Whether [value] is 43 to 128 characters of `[A-Z] [a-z] [0-9] - . _ ~` (RFC 7636 §4.1). */
    public fun isVerifier(value: String): Boolean =
        value.length in 43..128 && value.all { it in 'A'..'Z' || it in 'a'..'z' || it in '0'..'9' || it in "-._~" }
}
