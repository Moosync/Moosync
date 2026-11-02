package app.moosync.android

import android.Manifest
import android.app.Activity
import android.content.pm.PackageManager
import android.os.Build
import android.util.Log

class PermissionHandler(private val activity: Activity) {

    fun hasAudioPermission(): Boolean {
        val perm = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            Manifest.permission.READ_MEDIA_AUDIO
        } else {
            Manifest.permission.READ_EXTERNAL_STORAGE
        }
        return activity.checkSelfPermission(perm) == PackageManager.PERMISSION_GRANTED
    }

    fun requestAudioPermission() {
        val perm = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            Manifest.permission.READ_MEDIA_AUDIO
        } else {
            Manifest.permission.READ_EXTERNAL_STORAGE
        }
        activity.requestPermissions(arrayOf(perm), REQUEST_AUDIO_PERMISSION)
    }

    fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray
    ) {
        val granted = grantResults.isNotEmpty() && grantResults[0] == PackageManager.PERMISSION_GRANTED
        Log.d("MoosyncAndroid", "PermissionHandler onRequestPermissionsResult: req=$requestCode, granted=$granted")
        try {
            NativeBridge.nativeOnPermissionResult(requestCode, granted)
        } catch (e: UnsatisfiedLinkError) {
            Log.e("MoosyncAndroid", "Failed to invoke nativeOnPermissionResult", e)
        }
    }

    companion object {
        const val REQUEST_AUDIO_PERMISSION = 1001
    }
}
