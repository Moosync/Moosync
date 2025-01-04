package app.moosync.audioplayer.services.interfaces

import app.moosync.audioplayer.models.PlaybackState

interface MediaServiceWrapper {
    val controls: MediaControls

    fun decideQuit()

    fun setMainActivityStatus(isRunning: Boolean)

    fun addMediaPlayerCallbacks(callbacks: MediaPlayerCallbacks)
}