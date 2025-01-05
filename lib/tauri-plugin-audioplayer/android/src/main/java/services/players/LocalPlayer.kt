package app.moosync.audioplayer.services.players

import android.content.ContentUris
import android.content.Context
import android.media.MediaPlayer
import android.net.Uri
import android.os.Handler
import android.os.Looper
import android.provider.MediaStore
import android.util.Log
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
        return ContentUris.withAppendedId(
            MediaStore.Audio.Media.EXTERNAL_CONTENT_URI,
            id.toLong()
        )
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
            playerInstance.pause()
        }
    }

    override fun stop() {
        playerInstance.stop()
    }

    override fun release() {
        playerInstance.release()
    }
}