package app.moosync.audioplayer.services.players

import android.content.Context
import app.moosync.audioplayer.models.Song

abstract class GenericPlayer {
    abstract fun canPlay(song: Song): Boolean
    abstract fun load(mContext: Context, src: String, autoPlay: Boolean)

    abstract fun play()
    abstract fun pause()
    abstract fun stop()
    abstract fun release()

    abstract var progress: Int
    abstract val isPlaying: Boolean

    abstract fun setPlayerListeners(playerListeners: PlayerListeners)
    abstract fun removePlayerListeners()
}

interface PlayerListeners {
    fun onSongEnded(key: String)
    fun onTimeChange(key: String, time: Int)
}