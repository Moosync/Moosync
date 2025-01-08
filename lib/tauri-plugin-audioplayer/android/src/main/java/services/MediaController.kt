package app.moosync.audioplayer.services

import android.content.Context
import android.support.v4.media.session.MediaSessionCompat
import android.util.Log
import app.moosync.audioplayer.R
import app.moosync.audioplayer.models.MetadataArgs
import app.moosync.audioplayer.services.interfaces.MediaControls
import app.moosync.audioplayer.services.interfaces.MediaPlayerCallbacks
import app.moosync.audioplayer.services.players.PlayerListeners


class MediaController(private val mContext: Context) {

    // Manages media session
    private var mediaSessionHandler: MediaSessionHandler = MediaSessionHandler(mContext)

    // Session token required by service class
    val sessionToken: MediaSessionCompat.Token
        get() = mediaSessionHandler.sessionToken

    val notificationManager = NotificationHandler(mContext, sessionToken, R.drawable.ic_android_black_24dp)

    // Exposed controller abstraction for app to control media playback
    val controls: MediaControls

    private val playbackManager: PlaybackManager

    private val mediaPlayerCallbacks: MutableList<MediaPlayerCallbacks> = mutableListOf()
    private val mediaSessionCallbacks: MutableList<MediaSessionCompat.Callback> = mutableListOf()

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

    fun addMediaSessionCallbacks(callbacks: MediaSessionCompat.Callback) {
        Log.d("TAG", "addMediaSessionCallbacks: registering callback")
        mediaSessionCallbacks.add(callbacks)
    }

    private fun emitInAllCallbacks(c: (callback: MediaPlayerCallbacks) -> Unit) {
        for (callback in this.mediaPlayerCallbacks) {
            Log.d("TAG", "emitInAllCallbacks: emitting time change event in callback")
            c.invoke(callback)
        }
    }

    private fun emitInAllMediaSessionCallbacks(c: (callback: MediaSessionCompat.Callback) -> Unit) {
        for (callback in this.mediaSessionCallbacks) {
            Log.d("TAG", "emitInAllCallbacks: emitting time change event in callback")
            c.invoke(callback)
        }
    }

    init {
        mediaSessionHandler.setCommunicatorCallback(object : MediaSessionCompat.Callback() {
            override fun onPlay() {
                Log.d("TAG", "onPlay: media session play")
                emitInAllMediaSessionCallbacks { it.onPlay() }
            }

            override fun onPause() {
                Log.d("TAG", "onPause: media session pause")
                emitInAllMediaSessionCallbacks { it.onPause() }
            }

            override fun onStop() {
                Log.d("TAG", "onStop: media session stop")
                emitInAllMediaSessionCallbacks { it.onStop() }
            }

            override fun onSeekTo(pos: Long) {
                Log.d("TAG", "onStop: media session seek")
                emitInAllMediaSessionCallbacks { it.onSeekTo(pos) }
            }
        })

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
                Log.d("TAG", "play: got play command $key")
                playbackManager.play(key)
            }

            override fun pause(key: String) {
                Log.d("TAG", "pause: got pause command $key")
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

            override fun updateMetadata(metadata: MetadataArgs?) {
                mediaSessionHandler.updateMetadata(metadata)
                if (metadata == null) {
                    notificationManager.clearNotification()
                } else {
                    notificationManager.updateMetadata()
                }
            }

            override fun updatePlayerState(isPlaying: Boolean, pos: Int) {
                mediaSessionHandler.updatePlayerState(isPlaying, pos)
                notificationManager.updateMetadata()
            }

            override fun initializeLibrespot(token: String) {
                playbackManager.initializeLibrespot(token)
            }
        }
    }

    fun release() {
        playbackManager.release()
    }
}