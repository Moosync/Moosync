package app.moosync.audioplayer.services

import android.content.Context
import android.support.v4.media.session.MediaSessionCompat
import android.util.Log
import app.moosync.audioplayer.services.interfaces.MediaControls
import app.moosync.audioplayer.services.interfaces.MediaPlayerCallbacks
import app.moosync.audioplayer.services.players.PlayerListeners


class MediaController(private val mContext: Context) {

    // Manages media session
    private var mediaSessionHandler: MediaSessionHandler = MediaSessionHandler(mContext)

    // Session token required by service class
    val sessionToken: MediaSessionCompat.Token
        get() = mediaSessionHandler.sessionToken

    // Exposed controller abstraction for app to control media playback
    val controls: MediaControls

    private val playbackManager: PlaybackManager

    private val mediaPlayerCallbacks: MutableList<MediaPlayerCallbacks> = mutableListOf()

    private fun handleTimeChange(key: String, time: Int) {
        emitInAllCallbacks {it.onTimeChange(key, time)}
    }

    private fun handleSongEnded(key: String) {
        emitInAllCallbacks {it.onSongEnded(key)}
    }

    private fun seekToPos(key: String, pos: Int) {
        playbackManager.seek(key, pos)
        mediaSessionHandler.updatePlayerState(true, pos)
    }

    fun addPlayerCallbacks(callbacks: MediaPlayerCallbacks) {
        Log.d("TAG", "addPlayerCallbacks: registering callback")
        mediaPlayerCallbacks.add(callbacks)
    }

    private fun emitInAllCallbacks(c: (callback: MediaPlayerCallbacks) -> Unit) {
        for (callback in this.mediaPlayerCallbacks) {
            Log.d("TAG", "emitInAllCallbacks: emitting time change event in callback")
            c.invoke(callback)
        }
    }

    init {
//        mediaSessionHandler.setCommunicatorCallback(object : MediaSessionCompat.Callback() {
//            override fun onPlay() {
//                changePlaybackState(PlaybackState.PLAYING)
//            }
//
//            override fun onPause() {
//                changePlaybackState(PlaybackState.PAUSED)
//            }
//
//            override fun onStop() {
//                changePlaybackState(PlaybackState.STOPPED)
//            }
//
//            override fun onSeekTo(pos: Long) {
//                seekToPos(pos.toInt())
//            }
//        })

        playbackManager = PlaybackManager(mContext, object : PlayerListeners {
            override fun onSongEnded(key: String) {
                handleSongEnded(key)
            }

            override fun onTimeChange(key: String, time: Int) {
                handleTimeChange(key, time)
            }
        })


        controls = object : MediaControls {
            override fun play(key: String) {
                playbackManager.play(key)
            }

            override fun pause(key: String) {
                playbackManager.pause(key)
            }

            override fun stop(key: String) {
                playbackManager.stop(key)
            }

            override fun seek(key: String, time: Int) {
                playbackManager.seek(key, time)
            }

            override fun load(key: String, src: String, autoplay: Boolean) {
                playbackManager.load(key, mContext, src, autoplay)
            }

        }
    }

    fun release() {
        playbackManager.release()
    }

    interface ForegroundServiceCallbacks {
        fun shouldStartForeground()
        fun shouldStopForeground()
    }
}