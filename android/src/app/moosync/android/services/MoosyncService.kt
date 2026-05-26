package app.moosync.android.services

import android.app.Service
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import android.media.session.MediaSession
import android.util.Log
import app.moosync.android.services.Constants.NOTIFICATION_ID

class MoosyncService : Service() {

    private lateinit var mediaSessionHandler: MediaSessionHandler
    private lateinit var notificationHandler: NotificationHandler

    @Volatile
    var nativeCallbackPtr: Long = 0L

    override fun onCreate() {
        super.onCreate()
        Log.d("MoosyncAndroid", "MoosyncService onCreate")

        mediaSessionHandler = MediaSessionHandler(this)
        notificationHandler = NotificationHandler(this, mediaSessionHandler.sessionToken)

        mediaSessionHandler.setCommunicatorCallback(object : MediaSession.Callback() {
            override fun onPlay() {
                Log.d("MoosyncAndroid", "MediaSession: onPlay")
                val ptr = instance?.nativeCallbackPtr ?: 0L
                if (ptr != 0L) {
                    nativeOnPlay(ptr)
                }
            }

            override fun onPause() {
                Log.d("MoosyncAndroid", "MediaSession: onPause")
                val ptr = instance?.nativeCallbackPtr ?: 0L
                if (ptr != 0L) {
                    nativeOnPause(ptr)
                }
            }

            override fun onStop() {
                Log.d("MoosyncAndroid", "MediaSession: onStop")
                val ptr = instance?.nativeCallbackPtr ?: 0L
                if (ptr != 0L) {
                    nativeOnStop(ptr)
                }
            }

            override fun onSeekTo(pos: Long) {
                Log.d("MoosyncAndroid", "MediaSession: onSeekTo $pos")
                val ptr = instance?.nativeCallbackPtr ?: 0L
                if (ptr != 0L) {
                    nativeOnSeekTo(ptr, pos)
                }
            }

            override fun onSkipToNext() {
                Log.d("MoosyncAndroid", "MediaSession: onSkipToNext")
                val ptr = instance?.nativeCallbackPtr ?: 0L
                if (ptr != 0L) {
                    nativeOnSkipToNext(ptr)
                }
            }

            override fun onSkipToPrevious() {
                Log.d("MoosyncAndroid", "MediaSession: onSkipToPrevious")
                val ptr = instance?.nativeCallbackPtr ?: 0L
                if (ptr != 0L) {
                    nativeOnSkipToPrevious(ptr)
                }
            }
        })

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
        return START_NOT_STICKY
    }

    override fun onDestroy() {
        Log.d("MoosyncAndroid", "MoosyncService onDestroy")
        instance = null
        mediaSessionHandler.release()
        notificationHandler.release()
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null

    // Native methods — implemented in Rust (mpris_android.rs)
    private external fun nativeOnPlay(callbackPtr: Long)
    private external fun nativeOnPause(callbackPtr: Long)
    private external fun nativeOnStop(callbackPtr: Long)
    private external fun nativeOnSeekTo(callbackPtr: Long, posMs: Long)
    private external fun nativeOnSkipToNext(callbackPtr: Long)
    private external fun nativeOnSkipToPrevious(callbackPtr: Long)

    companion object {
        @Volatile
        private var instance: MoosyncService? = null

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
            svc.mediaSessionHandler.updateMetadata(title, artistName, albumName, durationMs, thumbnailUri)
            svc.notificationHandler.updateNotification()
        }

        @JvmStatic
        fun updatePlayerState(isPlaying: Boolean, positionMs: Long) {
            val svc = instance ?: run {
                Log.w("MoosyncAndroid", "updatePlayerState: service not running")
                return
            }
            svc.mediaSessionHandler.updatePlayerState(isPlaying, positionMs)
            svc.notificationHandler.updateNotification()
        }

        @JvmStatic
        fun clearNotification() {
            val svc = instance
            if (svc != null) {
                svc.notificationHandler.clearNotification()
            }
        }

        @JvmStatic
        fun registerNativeCallback(callbackPtr: Long) {
            val svc = instance
            if (svc != null) {
                svc.nativeCallbackPtr = callbackPtr
            }
        }
    }
}
