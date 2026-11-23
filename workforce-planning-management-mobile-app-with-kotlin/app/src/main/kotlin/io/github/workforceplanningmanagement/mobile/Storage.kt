// What the app keeps on the phone: the deployment's address and the token, and nothing else (WPM-D227).
package io.github.workforceplanningmanagement.mobile

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import androidx.core.content.edit
import android.security.keystore.KeyProperties
import java.io.File
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/** Where the token is kept. */
interface TokenStore {
    fun load(): String?
    fun save(token: String)
    fun clear()
}

/** Where the deployment's address is kept. Not secret, but still not backed up (allowBackup is false). */
interface AddressStore {
    fun load(): String?
    fun save(baseUrl: String)
    fun clear()
}

/**
 * The token, encrypted with an AES-GCM key that never leaves the Android Keystore, in a file under `noBackupFilesDir`. The file
 * holds the IV and the ciphertext only; without this device's Keystore key it is unreadable. Not in preferences or a log.
 */
class KeystoreTokenStore(context: Context) : TokenStore {
    private val file = File(context.noBackupFilesDir, "token.bin")

    private fun key(): SecretKey {
        val keyStore = KeyStore.getInstance(KEYSTORE).apply { load(null) }
        (keyStore.getKey(ALIAS, null) as? SecretKey)?.let { return it }
        val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, KEYSTORE)
        generator.init(
            KeyGenParameterSpec.Builder(ALIAS, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setKeySize(256)
                .build(),
        )
        return generator.generateKey()
    }

    override fun load(): String? {
        if (!file.isFile) return null
        return runCatching {
            val bytes = file.readBytes()
            val iv = bytes.copyOfRange(0, IV_BYTES)
            val cipher = Cipher.getInstance(TRANSFORMATION)
            cipher.init(Cipher.DECRYPT_MODE, key(), GCMParameterSpec(TAG_BITS, iv))
            String(cipher.doFinal(bytes, IV_BYTES, bytes.size - IV_BYTES), Charsets.UTF_8)
        }.getOrElse {
            // A key that was removed (a new lock screen, a restore) leaves an unreadable file: sign in again.
            clear()
            null
        }
    }

    override fun save(token: String) {
        val cipher = Cipher.getInstance(TRANSFORMATION)
        cipher.init(Cipher.ENCRYPT_MODE, key())
        val sealed = cipher.doFinal(token.toByteArray(Charsets.UTF_8))
        file.writeBytes(cipher.iv + sealed)
    }

    override fun clear() {
        file.delete()
    }

    private companion object {
        const val KEYSTORE = "AndroidKeyStore"
        const val ALIAS = "wpm-mobile-token"
        const val TRANSFORMATION = "AES/GCM/NoPadding"
        const val IV_BYTES = 12
        const val TAG_BITS = 128
    }
}

/** The address, in the app's private preferences. */
class PreferencesAddressStore(context: Context) : AddressStore {
    private val prefs = context.getSharedPreferences("connection", Context.MODE_PRIVATE)
    override fun load(): String? = prefs.getString(KEY, null)
    override fun save(baseUrl: String) = prefs.edit { putString(KEY, baseUrl) }
    override fun clear() = prefs.edit { remove(KEY) }
    private companion object { const val KEY = "base_url" }
}
