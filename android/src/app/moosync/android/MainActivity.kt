package app.moosync.android

import android.app.NativeActivity
import android.content.pm.PackageManager
import android.os.Bundle
import android.util.Log

class MainActivity : NativeActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Log.d("MoosyncAndroid", "MainActivity onCreate - Slint app starting via NativeActivity")
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        val granted = grantResults.isNotEmpty() && grantResults[0] == PackageManager.PERMISSION_GRANTED
        Log.d("MoosyncAndroid", "onRequestPermissionsResult: req=$requestCode, granted=$granted")
        try {
            nativeOnPermissionResult(requestCode, granted)
        } catch (e: UnsatisfiedLinkError) {
            Log.e("MoosyncAndroid", "Failed to invoke nativeOnPermissionResult", e)
        }
    }

    override fun onBackPressed() {
        val handled = try {
            nativeOnBackPressed()
        } catch (e: UnsatisfiedLinkError) {
            Log.e("MoosyncAndroid", "Failed to invoke nativeOnBackPressed", e)
            false
        }
        Log.d("MoosyncAndroid", "MainActivity onBackPressed: handled=$handled")
        if (!handled) {
            super.onBackPressed()
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        Log.d("MoosyncAndroid", "MainActivity onDestroy")
    }

    private external fun nativeOnPermissionResult(requestCode: Int, granted: Boolean)
    private external fun nativeOnBackPressed(): Boolean

    companion object {
        init {
            try {
                System.loadLibrary("slint_app")
                Log.d("MoosyncAndroid", "Successfully loaded slint_app in MainActivity")
            } catch (e: UnsatisfiedLinkError) {
                Log.e("MoosyncAndroid", "Failed to load slint_app in MainActivity", e)
            }
        }
    }
}
