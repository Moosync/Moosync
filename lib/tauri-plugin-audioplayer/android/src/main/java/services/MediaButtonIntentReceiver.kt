package app.moosync.audioplayer.services

import android.content.Context
import android.content.Intent
import android.util.Log
import androidx.media.session.MediaButtonReceiver

class MediaButtonIntentReceiver : MediaButtonReceiver() {

    override fun onReceive(context: Context, intent: Intent) {
        Log.d("TAG", "Received intent: $intent")
    }
}