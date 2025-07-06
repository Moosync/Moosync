package app.moosync.selfupdate

import android.annotation.SuppressLint
import android.app.Activity
import android.content.Intent
import android.provider.Settings
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.Plugin
import kotlinx.coroutines.launch
import androidx.core.net.toUri

@InvokeArg
class SelfUpdateArgs {
    var payload: PlatformInfo? = null
}

@TauriPlugin
class SelfUpdatePlugin(private val activity: Activity) : Plugin(activity) {
    companion object {
        // Static state for permission retry logic
        var pendingApkFile: java.io.File? = null
        var pendingInvoke: Invoke? = null
        @SuppressLint("StaticFieldLeak")
        var pendingActivity: Activity? = null

        // These will be set by MainActivity
        lateinit var unknownSourcesLauncher: androidx.activity.result.ActivityResultLauncher<Intent>
        lateinit var apkInstallLauncher: androidx.activity.result.ActivityResultLauncher<Intent>

        fun handleUnknownSourcesResult(resultCode: Int) {
            val activity = pendingActivity
            if (activity != null &&
                            android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.O
            ) {
                val canInstall = activity.packageManager.canRequestPackageInstalls()

                if (canInstall && pendingApkFile != null) {
                    promptInstallStatic(activity, pendingApkFile!!)
                    pendingApkFile = null
                    pendingActivity = null
                } else if (!canInstall) {
                    pendingInvoke?.reject("Permission to install unknown apps was not granted.")
                    pendingInvoke = null
                    pendingActivity = null
                }
            }
        }

        fun handleApkInstallResult(resultCode: Int) {
            // Optionally handle APK install completion if needed

        }

        fun promptInstallStatic(activity: Activity, apkFile: java.io.File) {
            val apkUri: android.net.Uri =
                    if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.N) {
                        androidx.core.content.FileProvider.getUriForFile(
                                activity,
                                activity.packageName + ".fileprovider",
                                apkFile
                        )
                    } else {
                        android.net.Uri.fromFile(apkFile)
                    }

            // Check and request permission to install unknown sources if needed
            if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.O) {
                val canInstall = activity.packageManager.canRequestPackageInstalls()
                if (!canInstall) {
                    val intent =
                            Intent(Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES).apply {
                                data = ("package:" + activity.packageName).toUri()
                            }
                    pendingApkFile = apkFile
                    // Do NOT store activity reference!
                    unknownSourcesLauncher.launch(intent)
                    return
                }
            }

            val intent =
                    Intent(Intent.ACTION_INSTALL_PACKAGE).apply {
                        setData(apkUri)
                        flags = Intent.FLAG_GRANT_READ_URI_PERMISSION
                    }
            apkInstallLauncher.launch(intent)
        }
    }

    @Command
    fun download_and_install(invoke: Invoke) {

        val args = invoke.parseArgs(SelfUpdateArgs::class.java)
        val info =
                args.payload
                        ?: run {
                            invoke.reject("Missing payload")
                            return
                        }

        kotlinx.coroutines.CoroutineScope(kotlinx.coroutines.Dispatchers.IO).launch {
            try {
                val apkFile = downloadApk(info.url)
                if (!verifySignature(apkFile, info.signature)) {
                    apkFile.delete()
                    throw Exception("Signature verification failed")
                }
                // Store invoke and activity for retry logic
                pendingInvoke = invoke
                pendingActivity = activity
                promptInstallStatic(activity, apkFile)
                // Only resolve if install prompt is shown immediately (not waiting for permission)
                // invoke.resolve() will be called after install prompt on retry
            } catch (e: Exception) {
                invoke.reject("Update failed: ${e.message}")
            }
        }
    }

    private fun downloadApk(url: String): java.io.File {
        val apkFile = java.io.File(activity.filesDir, "update.apk")
        if (apkFile.exists()) {
            apkFile.delete()
        }
        java.net.URL(url).openStream().use { input ->
            java.io.FileOutputStream(apkFile).use { output -> input.copyTo(output) }
        }
        return apkFile
    }

    private fun verifySignature(file: java.io.File, expectedSignature: String): Boolean {
        val digest = java.security.MessageDigest.getInstance("SHA-256")
        val fileBytes = file.readBytes()
        val hash = digest.digest(fileBytes).joinToString("") { "%02x".format(it) }
        val isValid = hash.equals(expectedSignature, ignoreCase = true)
        return isValid
    }
}
