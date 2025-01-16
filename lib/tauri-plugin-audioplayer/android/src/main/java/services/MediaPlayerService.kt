// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

// Moosync
// Copyright (C) 2025 Moosync
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

package app.moosync.audioplayer.services

import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.net.ConnectivityManager
import android.net.Network
import android.net.NetworkCapabilities
import android.net.NetworkRequest
import android.net.wifi.WifiManager
import android.net.wifi.WifiManager.WifiLock
import android.os.Binder
import android.os.Build
import android.os.Bundle
import android.os.IBinder
import android.os.PowerManager
import android.support.v4.media.MediaBrowserCompat
import android.support.v4.media.session.MediaSessionCompat
import android.util.Log
import androidx.media.MediaBrowserServiceCompat
import app.moosync.audioplayer.services.Constants.ACTION_FROM_MAIN_ACTIVITY
import app.moosync.audioplayer.services.Constants.NOTIFICATION_ID
import app.moosync.audioplayer.services.interfaces.MediaControls
import app.moosync.audioplayer.services.interfaces.MediaPlayerCallbacks
import app.moosync.audioplayer.services.interfaces.MediaServiceWrapper


class MediaPlayerService : MediaBrowserServiceCompat() {
    // Manages everything related to music playback
    private lateinit var mediaController: MediaController

    // Binder used to connect to activity
    private val binder: IBinder = MediaPlayerBinder()

    private var isMainActivityRunning = false
    private var wl: WifiLock? = null



    override fun onCreate() {
        super.onCreate()

        mediaController = MediaController(this)
        sessionToken = mediaController.sessionToken

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                startForeground(
                    NOTIFICATION_ID,
                    mediaController.notificationManager.notification!!,
                    ServiceInfo.FOREGROUND_SERVICE_TYPE_MANIFEST
                )

        } else {
            startForeground(NOTIFICATION_ID, mediaController.notificationManager.notification)
        }
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
