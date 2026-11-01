package app.moosync.android.security

import android.content.Context
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import android.util.Log
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

object KeyStoreHelper {
    private const val TAG = "MoosyncKeyStore"
    private const val ANDROID_KEYSTORE = "AndroidKeyStore"
    private const val KEY_ALIAS = "moosync_pref_master_key"
    private const val PREFS_NAME = "moosync_secure_storage"
    private const val ENCRYPTED_SECRET_KEY = "encrypted_secret"
    private const val IV_KEY = "iv_secret"
    private const val GCM_TAG_LENGTH = 128

    private fun getOrCreateMasterKey(): SecretKey {
        val keyStore = KeyStore.getInstance(ANDROID_KEYSTORE).apply { load(null) }
        if (!keyStore.containsAlias(KEY_ALIAS)) {
            val keyGenerator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, ANDROID_KEYSTORE)
            val spec = KeyGenParameterSpec.Builder(
                KEY_ALIAS,
                KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT
            )
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setKeySize(256)
                .build()
            keyGenerator.init(spec)
            keyGenerator.generateKey()
        }
        val entry = keyStore.getEntry(KEY_ALIAS, null) as KeyStore.SecretKeyEntry
        return entry.secretKey
    }

    @JvmStatic
    fun getSecret(context: Context): ByteArray? {
        return try {
            val prefs = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            val encryptedBase64 = prefs.getString(ENCRYPTED_SECRET_KEY, null) ?: return null
            val ivBase64 = prefs.getString(IV_KEY, null) ?: return null

            val masterKey = getOrCreateMasterKey()
            val cipher = Cipher.getInstance("AES/GCM/NoPadding")
            val iv = Base64.decode(ivBase64, Base64.DEFAULT)
            cipher.init(Cipher.DECRYPT_MODE, masterKey, GCMParameterSpec(GCM_TAG_LENGTH, iv))
            val encrypted = Base64.decode(encryptedBase64, Base64.DEFAULT)
            cipher.doFinal(encrypted)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to get/decrypt secret from AndroidKeyStore", e)
            null
        }
    }

    @JvmStatic
    fun setSecret(context: Context, secret: ByteArray): Boolean {
        return try {
            val masterKey = getOrCreateMasterKey()
            val cipher = Cipher.getInstance("AES/GCM/NoPadding")
            cipher.init(Cipher.ENCRYPT_MODE, masterKey)
            val iv = cipher.iv
            val encrypted = cipher.doFinal(secret)

            val prefs = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            prefs.edit()
                .putString(ENCRYPTED_SECRET_KEY, Base64.encodeToString(encrypted, Base64.NO_WRAP))
                .putString(IV_KEY, Base64.encodeToString(iv, Base64.NO_WRAP))
                .commit()
        } catch (e: Exception) {
            Log.e(TAG, "Failed to encrypt/save secret with AndroidKeyStore", e)
            false
        }
    }
}
