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

package app.moosync.audioplayer.services

import android.content.Context
import android.media.AudioAttributes
import android.media.AudioFocusRequest
import android.media.AudioManager
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

    val notificationManager = NotificationHandler(mContext, sessionToken, R.drawable.ic_launcher)

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

    fun addPlayerCallbacks(callbacks: MediaPlayerCallbacks) {
        mediaPlayerCallbacks.add(callbacks)
    }

    fun addMediaSessionCallbacks(callbacks: MediaSessionCompat.Callback) {
        mediaSessionCallbacks.add(callbacks)
    }

    private fun emitInAllCallbacks(c: (callback: MediaPlayerCallbacks) -> Unit) {
        for (callback in this.mediaPlayerCallbacks) {
            c.invoke(callback)
        }
    }

    private fun emitInAllMediaSessionCallbacks(c: (callback: MediaSessionCompat.Callback) -> Unit) {
        for (callback in this.mediaSessionCallbacks) {
            c.invoke(callback)
        }
    }

    init {
        val audioManager = mContext.applicationContext.getSystemService(Context.AUDIO_SERVICE) as AudioManager
        val audioFocusRequest = AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN).setAudioAttributes(
            AudioAttributes.Builder()
                .setUsage(AudioAttributes.USAGE_MEDIA)
                .setContentType(AudioAttributes.CONTENT_TYPE_MUSIC)
                .build()
        ).build()

        val result = audioManager.requestAudioFocus(audioFocusRequest)

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

            override fun onSkipToNext() {
                Log.d("TAG", "onStop: media session onSkipToNext")
                emitInAllMediaSessionCallbacks { it.onSkipToNext() }
            }

            override fun onSkipToPrevious() {
                Log.d("TAG", "onStop: media session onSkipToPrevious")
                emitInAllMediaSessionCallbacks { it.onSkipToPrevious() }
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
        }
    }

    fun release() {
        playbackManager.release()
    }
}
