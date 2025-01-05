package app.moosync.audioplayer.services.interfaces

import android.support.v4.media.session.MediaSessionCompat
import app.moosync.audioplayer.models.PlaybackState
import app.moosync.audioplayer.services.NotificationHandler

interface MediaServiceWrapper {
    val controls: MediaControls

    fun decideQuit()

    fun setMainActivityStatus(isRunning: Boolean)

    fun addMediaPlayerCallbacks(callbacks: MediaPlayerCallbacks)
    fun addMediaSessionCallbacks(callbacks: MediaSessionCompat.Callback)
}