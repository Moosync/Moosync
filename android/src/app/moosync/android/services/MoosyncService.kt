package app.moosync.android.services

import android.app.Service
import android.content.Intent
import android.content.pm.ServiceInfo
import android.media.session.MediaSession
import android.os.Build
import android.os.IBinder
import android.util.Log
import android.view.KeyEvent
import app.moosync.android.NativeBridge
import app.moosync.android.services.Constants.NOTIFICATION_ID

class MoosyncService : Service() {

    private lateinit var mediaSessionHandler: MediaSessionHandler
    private lateinit var notificationHandler: NotificationHandler

    override fun onCreate() {
        super.onCreate()
        Log.d("MoosyncAndroid", "MoosyncService onCreate")

        mediaSessionHandler = MediaSessionHandler(this)
        notificationHandler = NotificationHandler(this, mediaSessionHandler.sessionToken)

        mediaSessionHandler.setCommunicatorCallback(
            object : MediaSession.Callback() {
                override fun onPlay() {
                    Log.d("MoosyncAndroid", "MediaSession: onPlay")
                    NativeBridge.nativeOnPlay()
                }

                override fun onPause() {
                    Log.d("MoosyncAndroid", "MediaSession: onPause")
                    NativeBridge.nativeOnPause()
                }

                override fun onStop() {
                    Log.d("MoosyncAndroid", "MediaSession: onStop")
                    NativeBridge.nativeOnStop()
                }

                override fun onSeekTo(pos: Long) {
                    Log.d("MoosyncAndroid", "MediaSession: onSeekTo $pos")
                    NativeBridge.nativeOnSeekTo(pos)
                }

                override fun onSkipToNext() {
                    Log.d("MoosyncAndroid", "MediaSession: onSkipToNext")
                    NativeBridge.nativeOnSkipToNext()
                }

                override fun onSkipToPrevious() {
                    Log.d("MoosyncAndroid", "MediaSession: onSkipToPrevious")
                    NativeBridge.nativeOnSkipToPrevious()
                }
            }
        )

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            startForeground(
                NOTIFICATION_ID,
                notificationHandler.notification!!,
                ServiceInfo.FOREGROUND_SERVICE_TYPE_MEDIA_PLAYBACK
            )
        } else {
            startForeground(NOTIFICATION_ID, notificationHandler.notification!!)
        }

        instance = this
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent != null && Intent.ACTION_MEDIA_BUTTON == intent.action) {
            val keyEvent = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                intent.getParcelableExtra(Intent.EXTRA_KEY_EVENT, KeyEvent::class.java)
            } else {
                @Suppress("DEPRECATION")
                intent.getParcelableExtra<KeyEvent>(Intent.EXTRA_KEY_EVENT)
            }
            if (keyEvent != null && keyEvent.action == KeyEvent.ACTION_DOWN) {
                handleMediaKeyEvent(keyEvent.keyCode)
            }
        }
        return START_NOT_STICKY
    }

    private fun handleMediaKeyEvent(keyCode: Int) {
        Log.d("MoosyncAndroid", "handleMediaKeyEvent: keyCode=$keyCode")
        when (keyCode) {
            KeyEvent.KEYCODE_MEDIA_PLAY -> NativeBridge.nativeOnPlay()
            KeyEvent.KEYCODE_MEDIA_PAUSE -> NativeBridge.nativeOnPause()
            KeyEvent.KEYCODE_MEDIA_PLAY_PAUSE, KeyEvent.KEYCODE_HEADSETHOOK -> {
                if (notificationHandler.isPlaying) {
                    NativeBridge.nativeOnPause()
                } else {
                    NativeBridge.nativeOnPlay()
                }
            }
            KeyEvent.KEYCODE_MEDIA_STOP -> NativeBridge.nativeOnStop()
            KeyEvent.KEYCODE_MEDIA_NEXT -> NativeBridge.nativeOnSkipToNext()
            KeyEvent.KEYCODE_MEDIA_PREVIOUS -> NativeBridge.nativeOnSkipToPrevious()
        }
    }

    override fun onDestroy() {
        Log.d("MoosyncAndroid", "MoosyncService onDestroy")
        instance = null
        mediaSessionHandler.release()
        notificationHandler.release()
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null

    companion object {
        @Volatile private var instance: MoosyncService? = null

        @JvmStatic
        fun updateMetadata(
            title: String?,
            artistName: String?,
            albumName: String?,
            durationMs: Long,
            thumbnailUri: String?
        ) {
            val svc = instance ?: run {
                Log.w("MoosyncAndroid", "updateMetadata: service not running")
                return
            }
            svc.notificationHandler.updateMetadata(title, artistName, albumName)
            svc.mediaSessionHandler.updateMetadata(
                title,
                artistName,
                albumName,
                durationMs,
                thumbnailUri
            ) { bitmap ->
                svc.notificationHandler.updateCover(bitmap)
            }
        }

        @JvmStatic
        fun updatePlayerState(isPlaying: Boolean, positionMs: Long) {
            val svc = instance ?: run {
                Log.w("MoosyncAndroid", "updatePlayerState: service not running")
                return
            }
            svc.mediaSessionHandler.updatePlayerState(isPlaying, positionMs)
            svc.notificationHandler.updatePlayerState(isPlaying)
        }

        @JvmStatic
        fun clearNotification() {
            val svc = instance
            if (svc != null) {
                svc.notificationHandler.clearNotification()
            }
        }
    }
}
