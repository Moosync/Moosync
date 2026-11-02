package app.moosync.android

import android.util.Log

object NativeBridge {
    init {
        try {
            System.loadLibrary("slint_app")
            Log.d("MoosyncAndroid", "NativeBridge: initialized and loaded slint_app")
        } catch (e: UnsatisfiedLinkError) {
            Log.e("MoosyncAndroid", "NativeBridge: Failed to load slint_app", e)
        }
    }

    fun init() {
        // Triggers object initialization
    }

    external fun nativeOnPermissionResult(requestCode: Int, granted: Boolean)
    external fun nativeOnBackPressed(): Boolean
    external fun nativeOnDeepLink(url: String)
    external fun nativeOnDirectoryPicked(path: String)
    external fun nativeOnFilePicked(path: String)

    external fun nativeOnPlay()
    external fun nativeOnPause()
    external fun nativeOnStop()
    external fun nativeOnSeekTo(posMs: Long)
    external fun nativeOnSkipToNext()
    external fun nativeOnSkipToPrevious()
}
