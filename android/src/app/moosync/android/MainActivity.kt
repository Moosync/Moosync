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
        super.onDestroy()
        Log.d("MoosyncAndroid", "MainActivity onDestroy")
    }

    companion object {
        init {
            NativeBridge.init()
        }
    }
}
