package app.moosync.android.services

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.os.Build
import android.util.Log
import android.view.KeyEvent

class MediaButtonIntentReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        Log.d("MoosyncAndroid", "MediaButtonIntentReceiver: received intent: $intent")

        val keyEvent = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            intent.getParcelableExtra(Intent.EXTRA_KEY_EVENT, KeyEvent::class.java)
        } else {
            @Suppress("DEPRECATION")
            intent.getParcelableExtra<KeyEvent>(Intent.EXTRA_KEY_EVENT)
        }

        if (keyEvent != null && keyEvent.action == KeyEvent.ACTION_DOWN) {
            val svc = MoosyncService.instance
            if (svc != null) {
                svc.handleMediaKeyEvent(keyEvent.keyCode)
                return
            }
        }

        val serviceIntent = Intent(context, MoosyncService::class.java).apply {
            action = intent.action
            intent.extras?.let {
                putExtras(it)
            }
        }
        try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                context.startForegroundService(serviceIntent)
            } else {
                context.startService(serviceIntent)
            }
        } catch (e: Exception) {
            Log.e("MoosyncAndroid", "Failed to forward media button to MoosyncService", e)
        }
    }
}
