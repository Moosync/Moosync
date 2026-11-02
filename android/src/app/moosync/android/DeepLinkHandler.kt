package app.moosync.android

import android.content.Intent
import android.util.Log

class DeepLinkHandler {
    var pendingDeepLink: String? = null
        private set

    fun handleIntent(intent: Intent?) {
        if (intent == null) return
        val action = intent.action
        val data = intent.dataString
        Log.d("MoosyncAndroid", "DeepLinkHandler handleIntent: action=$action, data=$data")
        if (Intent.ACTION_VIEW == action && data != null) {
            try {
                NativeBridge.nativeOnDeepLink(data)
            } catch (e: UnsatisfiedLinkError) {
                Log.w("MoosyncAndroid", "nativeOnDeepLink not linked yet, caching pending deep link", e)
                pendingDeepLink = data
            }
        }
    }

    fun processPendingDeepLink() {
        val link = pendingDeepLink ?: return
        pendingDeepLink = null
        try {
            NativeBridge.nativeOnDeepLink(link)
        } catch (e: UnsatisfiedLinkError) {
            Log.e("MoosyncAndroid", "Failed to invoke nativeOnDeepLink for pending link", e)
        }
    }
}
