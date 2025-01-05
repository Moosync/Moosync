package app.moosync.audioplayer.services

import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Binder
import android.os.Build
import android.os.Bundle
import android.os.IBinder
import android.support.v4.media.MediaBrowserCompat
import android.support.v4.media.session.MediaSessionCompat
import android.util.Log
import androidx.media.MediaBrowserServiceCompat
import app.moosync.audioplayer.R
import app.moosync.audioplayer.models.PlaybackState
import app.moosync.audioplayer.models.Song
import app.moosync.audioplayer.services.Constants.ACTION_FROM_MAIN_ACTIVITY
import app.moosync.audioplayer.services.interfaces.MediaControls
import app.moosync.audioplayer.services.interfaces.MediaPlayerCallbacks
import app.moosync.audioplayer.services.interfaces.MediaServiceWrapper

class MediaPlayerService : MediaBrowserServiceCompat() {
    // Manages everything related to music playback
    private lateinit var mediaController: MediaController

    // Binder used to connect to activity
    private val binder: IBinder = MediaPlayerBinder()

    private var isMainActivityRunning = false

    override fun onCreate() {
        super.onCreate()

        mediaController = MediaController(this)
        sessionToken = mediaController.sessionToken
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val fromMainActivity = intent?.extras?.getBoolean(ACTION_FROM_MAIN_ACTIVITY) ?: false
        if (fromMainActivity) {
            isMainActivityRunning = true
        }
        return START_NOT_STICKY
    }

    private fun quit() {
        mediaController.release()
        stopSelf()
    }

    override fun onDestroy() {
        mediaController.release()
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? {
        return if ("android.media.browse.MediaBrowserService" == intent?.action) {
            super.onBind(intent)
        } else binder
    }

    override fun onGetRoot(
        clientPackageName: String,
        clientUid: Int,
        rootHints: Bundle?
    ): BrowserRoot {
        return BrowserRoot("Moosync", null)
    }

    override fun onLoadChildren(
        parentId: String,
        result: Result<MutableList<MediaBrowserCompat.MediaItem>>
    ) {
        result.sendResult(null)
    }

    fun decideQuit() {
        quit()
    }

    inner class MediaPlayerBinder : Binder() {
        val service = object: MediaServiceWrapper {
            override val controls: MediaControls
                get() = this@MediaPlayerService.mediaController.controls

            override fun decideQuit() {
                this@MediaPlayerService.decideQuit()
            }

            override fun setMainActivityStatus(isRunning: Boolean) {
                isMainActivityRunning = isRunning
            }

            override fun addMediaPlayerCallbacks(callbacks: MediaPlayerCallbacks) {
                this@MediaPlayerService.mediaController.addPlayerCallbacks(callbacks)
            }

            override fun addMediaSessionCallbacks(callbacks: MediaSessionCompat.Callback) {
                this@MediaPlayerService.mediaController.addMediaSessionCallbacks(callbacks)
            }
        }
    }
}