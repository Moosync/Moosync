package app.moosync.android.services

import android.app.Service
import android.content.Intent
import android.content.pm.ServiceInfo
import android.media.session.MediaSession
import android.os.Build
import android.os.IBinder
import android.util.Log
import android.view.KeyEvent
import app.moosync.android.MainActivity
import app.moosync.android.NativeBridge
import app.moosync.android.services.Constants.NOTIFICATION_ID

open class MoosyncService : Service() {

    private lateinit var mediaSessionHandler: MediaSessionHandler
    private lateinit var notificationHandler: NotificationHandler

    override fun onCreate() {
        super.onCreate()
        Log.d("MoosyncAndroid", "MoosyncService onCreate")

        isNotificationActive = true
        isTaskRemoved = false
        instance = this

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

        val pendingMeta = pendingMetadata
        if (pendingMeta != null) {
            notificationHandler.updateMetadata(
                pendingMeta.title,
                pendingMeta.artistName,
                pendingMeta.albumName
            )
        }

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

        if (pendingMeta != null) {
            updateMetadata(
                pendingMeta.title,
                pendingMeta.artistName,
                pendingMeta.albumName,
                pendingMeta.durationMs,
                pendingMeta.thumbnailUri
            )
        }
        pendingPlayerState?.let {
            updatePlayerState(it.isPlaying, it.positionMs)
        }
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

    override fun onTaskRemoved(rootIntent: Intent?) {
        super.onTaskRemoved(rootIntent)
        Log.d("MoosyncAndroid", "MoosyncService onTaskRemoved")
        isTaskRemoved = true
        checkShouldKillApp()
    }

    open fun handleMediaKeyEvent(keyCode: Int) {
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

    open fun checkShouldKillApp() {
        val taskRemoved = isTaskRemoved || !MainActivity.isActivityAlive
        if (shouldKillApp(taskRemoved, isNotificationActive)) {
            Log.d("MoosyncAndroid", "MoosyncService: Task removed and notification dismissed -> killing app")
            killApp()
        }
    }

    open fun killApp() {
        try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
                stopForeground(STOP_FOREGROUND_REMOVE)
            } else {
                @Suppress("DEPRECATION")
                stopForeground(true)
            }
        } catch (_: Exception) {}
        stopSelf()
        android.os.Process.killProcess(android.os.Process.myPid())
        kotlin.system.exitProcess(0)
    }

    override fun onDestroy() {
        Log.d("MoosyncAndroid", "MoosyncService onDestroy")
        instance = null
        mediaSessionHandler.release()
        notificationHandler.release()
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null

    private data class MetadataArgs(
        val title: String?,
        val artistName: String?,
        val albumName: String?,
        val durationMs: Long,
        val thumbnailUri: String?
    )

    private data class PlayerStateArgs(
        val isPlaying: Boolean,
        val positionMs: Long
    )

    companion object {
        @Volatile var instance: MoosyncService? = null
        @Volatile var isTaskRemoved: Boolean = false
        @Volatile var isNotificationActive: Boolean = false
        @Volatile private var pendingMetadata: MetadataArgs? = null
        @Volatile private var pendingPlayerState: PlayerStateArgs? = null

        @JvmStatic
        fun shouldKillApp(taskRemoved: Boolean, notificationActive: Boolean): Boolean {
            return taskRemoved && !notificationActive
        }

        @JvmStatic
        fun onNotificationDismissed() {
            Log.d("MoosyncAndroid", "MoosyncService: onNotificationDismissed")
            isNotificationActive = false
            instance?.checkShouldKillApp()
        }

        @JvmStatic
        fun updateMetadata(
            title: String?,
            artistName: String?,
            albumName: String?,
            durationMs: Long,
            thumbnailUri: String?
        ) {
            val svc = instance
            if (svc == null) {
                Log.w("MoosyncAndroid", "updateMetadata: service not running, caching pending metadata")
                pendingMetadata = MetadataArgs(title, artistName, albumName, durationMs, thumbnailUri)
                return
            }
            pendingMetadata = null
            isNotificationActive = true
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
            val svc = instance
            if (svc == null) {
                Log.w("MoosyncAndroid", "updatePlayerState: service not running, caching pending state")
                pendingPlayerState = PlayerStateArgs(isPlaying, positionMs)
                return
            }
            pendingPlayerState = null
            svc.mediaSessionHandler.updatePlayerState(isPlaying, positionMs)
            svc.notificationHandler.updatePlayerState(isPlaying)

            if (isPlaying) {
                isNotificationActive = true
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                    svc.startForeground(
                        NOTIFICATION_ID,
                        svc.notificationHandler.notification!!,
                        ServiceInfo.FOREGROUND_SERVICE_TYPE_MEDIA_PLAYBACK
                    )
                } else {
                    svc.startForeground(NOTIFICATION_ID, svc.notificationHandler.notification!!)
                }
            } else {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
                    svc.stopForeground(STOP_FOREGROUND_DETACH)
                } else {
                    @Suppress("DEPRECATION")
                    svc.stopForeground(false)
                }
            }
        }

        @JvmStatic
        fun clearNotification() {
            isNotificationActive = false
            pendingMetadata = null
            pendingPlayerState = null
            val svc = instance
            if (svc != null) {
                svc.notificationHandler.clearNotification()
                svc.checkShouldKillApp()
            }
        }
    }
}
