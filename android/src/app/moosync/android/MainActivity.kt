package app.moosync.android

import android.app.NativeActivity
import android.content.Intent
import android.os.Bundle
import android.util.Log

class MainActivity : NativeActivity() {
    val deepLinkHandler = DeepLinkHandler()
    val storagePickerHandler = StoragePickerHandler(this)
    val permissionHandler = PermissionHandler(this)
    private val backPressHandler = BackPressHandler {
        @Suppress("DEPRECATION")
        super.onBackPressed()
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        if (hasRunBefore) {
            Log.w("MoosyncAndroid", "MainActivity: Process was kept alive after previous activity destruction; restarting process for fresh NativeActivity")
            val restartIntent = packageManager.getLaunchIntentForPackage(packageName)?.apply {
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK)
            }
            startActivity(restartIntent)
            finish()
            android.os.Process.killProcess(android.os.Process.myPid())
            kotlin.system.exitProcess(0)
            return
        }
        hasRunBefore = true
        isActivityAlive = true
        super.onCreate(savedInstanceState)
        Log.d("MoosyncAndroid", "MainActivity onCreate - Slint app starting via NativeActivity")
        deepLinkHandler.handleIntent(intent)
    }

    override fun onNewIntent(intent: Intent?) {
        super.onNewIntent(intent)
        setIntent(intent)
        deepLinkHandler.handleIntent(intent)
    }

    fun processPendingDeepLink() = deepLinkHandler.processPendingDeepLink()

    fun openDirectoryPicker() = storagePickerHandler.openDirectoryPicker()

    fun openFilePicker(filter: String) = storagePickerHandler.openFilePicker(filter)

    fun hasAudioPermission(): Boolean = permissionHandler.hasAudioPermission()

    fun requestAudioPermission() = permissionHandler.requestAudioPermission()

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        storagePickerHandler.onActivityResult(requestCode, resultCode, data)
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        permissionHandler.onRequestPermissionsResult(requestCode, permissions, grantResults)
    }

    @Deprecated("Deprecated in Java")
    override fun onBackPressed() {
        backPressHandler.onBackPressed()
    }

    override fun onDestroy() {
        isActivityAlive = false
        super.onDestroy()
        Log.d("MoosyncAndroid", "MainActivity onDestroy")
    }

    companion object {
        @Volatile var isActivityAlive: Boolean = false
        @Volatile var hasRunBefore: Boolean = false

        init {
            NativeBridge.init()
        }
    }
}
