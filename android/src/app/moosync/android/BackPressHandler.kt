package app.moosync.android

import android.util.Log

class BackPressHandler(private val onUnhandled: () -> Unit) {

    fun onBackPressed() {
        val handled = try {
            NativeBridge.nativeOnBackPressed()
        } catch (e: UnsatisfiedLinkError) {
            Log.e("MoosyncAndroid", "Failed to invoke nativeOnBackPressed", e)
            false
        }
        Log.d("MoosyncAndroid", "BackPressHandler onBackPressed: handled=$handled")
        if (!handled) {
            onUnhandled()
        }
    }
}
