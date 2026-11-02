package app.moosync.android.services

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.util.Log

class NotificationDismissedReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        Log.d("MoosyncAndroid", "NotificationDismissedReceiver: notification dismissed")
        MoosyncService.onNotificationDismissed()
    }
}
