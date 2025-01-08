package app.moosync.audioplayer.services.interfaces

import app.moosync.audioplayer.models.MetadataArgs
import app.moosync.audioplayer.models.PlaybackState
import app.moosync.audioplayer.models.Song

interface MediaControls {
    fun play(key: String)
    fun pause(key: String)
    fun stop(key: String)

    fun seek(key: String, time: Int)

    fun load(key: String, src: String, autoplay: Boolean)

    fun updateMetadata(metadata: MetadataArgs?)
    fun updatePlayerState(isPlaying: Boolean, pos: Int)

    fun initializeLibrespot(token: String)
}