package app.moosync.android.services

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.util.Log

class MediaButtonIntentReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        Log.d("MoosyncAndroid", "MediaButtonIntentReceiver: received intent: $intent")
        val serviceIntent = Intent(context, MoosyncService::class.java).apply {
            action = intent.action
            intent.extras?.let {
                putExtras(it)
            }
        }
        try {
            context.startService(serviceIntent)
        } catch (e: Exception) {
            Log.e("MoosyncAndroid", "Failed to forward media button to MoosyncService", e)
        }
    }
}
