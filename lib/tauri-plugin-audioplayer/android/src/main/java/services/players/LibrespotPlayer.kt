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

package app.moosync.audioplayer.services.players

import android.content.Context
import android.os.Handler
import android.os.Looper
import android.util.Log
import app.moosync.audioplayer.models.Song
import services.players.librespot.LibrespotCallbacks
import services.players.librespot.LibrespotWrapper
import java.util.Timer

class LibrespotPlayer(private val context: Context): GenericPlayer() {

    private val TAG = "LibrespotPlayer"
    private val key = "LIBRESPOT"

    private var progressTimer: Timer? = null

    private fun cancelProgressTimer() {
        progressTimer?.cancel()
        progressTimer?.purge()
        progressTimer = null
    }

    val librespot: LibrespotWrapper = LibrespotWrapper(object: LibrespotCallbacks {
        override fun onPlay() {
            val handler = Handler(Looper.getMainLooper())
            val runnable = object: Runnable {
                override fun run() {
                    if (isPlaying) {
                        Log.d("TAG", "run: sending time change event")
                        emitInListeners { it.onTimeChange(key, 0) }
                    }
                    handler.postDelayed(this, 1000)
                }
            }

            runnable.run()
        }

        override fun onStop() {
            Log.d(TAG, "onStop: ")
        }

        override fun onPause() {
            Log.d(TAG, "onPause: ")
        }

        override fun onTimeChange(pos: Long) {
            Log.d(TAG, "onTimeChange: ")
        }

        override fun onEnded() {
            Log.d(TAG, "onEnded: ")
            emitInListeners { it.onSongEnded(key) }
        }

        override fun onSeek(pos: Long) {
            Log.d(TAG, "onSeek: ")
        }

        override fun onConnected() {
            Log.d(TAG, "onConnected: ")
        }

    })

    init {
        LibrespotWrapper.initializeAndroidContext()
    }

    companion object {
        init {
            System.loadLibrary("librespot_jni")
        }
    }

    override fun canPlay(song: Song): Boolean {
        return song.playbackUrl?.startsWith("spotify:") ?: false
    }

    override fun load(mContext: Context, src: String, autoPlay: Boolean) {
        librespot.load(src, autoPlay)
    }

    override fun play() {
        librespot.play()
    }

    override fun pause() {
        librespot.pause()
    }

    override fun stop() {

    }

    override fun release() {
        librespot.delete()
    }

    override var progress: Int
        get() = 0
        set(value) {}
    override val isPlaying: Boolean
        get() = false

    private var librespotPlayerListeners: MutableList<PlayerListeners> = mutableListOf()

    override fun setPlayerListeners(playerListeners: PlayerListeners) {
        librespotPlayerListeners.add(playerListeners)
    }

    fun emitInListeners(callback: (c: PlayerListeners) -> Unit) {
        for (listener in librespotPlayerListeners) {
            callback.invoke(listener)
        }
    }

    override fun removePlayerListeners() {
        librespotPlayerListeners.clear()
    }

    fun initialize(token: String) {
        val credentialsPath = context.dataDir.toString()
        val cachePath = context.cacheDir.toString()
        librespot.initializeLibrespot(credentialsPath, cachePath, token)
    }
}
