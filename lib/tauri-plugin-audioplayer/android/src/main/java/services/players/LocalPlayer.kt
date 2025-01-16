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

package app.moosync.audioplayer.services.players

import android.content.ContentUris
import android.content.Context
import android.media.MediaPlayer
import android.net.Uri
import android.os.Handler
import android.os.Looper
import android.provider.MediaStore
import android.util.Log
import androidx.core.net.toUri
import app.moosync.audioplayer.models.Song
import java.util.Timer


class LocalPlayer : GenericPlayer() {
    private val playerInstance = MediaPlayer()

    private val key = "LOCAL"

    override var progress: Int
        get() = playerInstance.currentPosition
        set(value) { playerInstance.seekTo(value) }

    private var ignoreSongEnded = false

    override val isPlaying: Boolean
        get() {
            try {
                return playerInstance.isPlaying
            } catch (e: Exception) {
                Log.e("TAG", "Error getting isPlaying:", e)
            }
            return false
        }

    private var isPlayerPrepared = false
    private val afterPreparedMethodCalls: MutableList<() -> Unit> = mutableListOf()


    override fun canPlay(song: Song): Boolean {
        return !song.path.isNullOrEmpty()
    }

    private fun buildUri(id: String): Uri {
        if (!id.startsWith("http")) {
            return ContentUris.withAppendedId(
                MediaStore.Audio.Media.EXTERNAL_CONTENT_URI,
                id.toLong()
            )
        }
        return id.toUri()
    }

    private fun runAfterPlayerPrepared(method: () -> Unit) {
        if (isPlayerPrepared) {
            method.invoke()
            return
        }
        afterPreparedMethodCalls.add(method)
    }

    private fun runQueuedMethods() {
        for (method in afterPreparedMethodCalls) {
            method.invoke()
        }
    }

    override fun load(mContext: Context, src: String, autoPlay: Boolean) {
        ignoreSongEnded = true
        isPlayerPrepared = false

        playerInstance.reset()

        val uri = buildUri(src)
        Log.d("TAG", "load: got uri $uri")
        playerInstance.setDataSource(mContext, uri)
        playerInstance.setVolume(1.0F, 1.0F)

        playerInstance.setOnPreparedListener {
            if (autoPlay) {
                Log.d("TAG", "load: autoplaying")
                it.start()
            }

            isPlayerPrepared = true
            ignoreSongEnded = false
            runQueuedMethods()
        }
        playerInstance.prepareAsync()
    }

    private var progressTimer: Timer? = null

    private fun cancelProgressTimer() {
        progressTimer?.cancel()
        progressTimer?.purge()
        progressTimer = null
    }

    override fun setPlayerListeners(playerListeners: PlayerListeners) {
        playerInstance.setOnCompletionListener {
            if (!ignoreSongEnded)  {
                playerListeners.onSongEnded(key)
                ignoreSongEnded = false
            }
        }

        val handler = Handler(Looper.getMainLooper())
        val runnable = object: Runnable {
            override fun run() {
                if (isPlaying) {
                    Log.d("TAG", "run: sending time change event")
                    playerListeners.onTimeChange(key, progress)
                }
                handler.postDelayed(this, 1000)
            }
        }

        runnable.run()
    }

    override fun removePlayerListeners() {
        playerInstance.setOnCompletionListener(null)
        cancelProgressTimer()
    }

    override fun play() {
        Log.d("TAG", "play: playing local player")
        runAfterPlayerPrepared {
            Log.d("TAG", "play: local player is prepared, starting")
            playerInstance.start()
        }
    }

    override fun pause() {
        runAfterPlayerPrepared {
            if (playerInstance.isPlaying) {
                playerInstance.pause()
            }
        }
    }

    override fun stop() {
        playerInstance.stop()
    }

    override fun release() {
        playerInstance.release()
    }
}
