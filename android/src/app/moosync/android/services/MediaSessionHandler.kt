package app.moosync.android.services

import android.app.PendingIntent
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.media.MediaMetadata
import android.media.session.MediaSession
import android.media.session.PlaybackState
import android.util.Log
import java.io.File
import java.net.URL
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors

class MediaSessionHandler(private val mContext: Context) {

    private val mMediaSession: MediaSession
    private val mExecutor: ExecutorService = Executors.newSingleThreadExecutor()

    val sessionToken: MediaSession.Token
        get() = mMediaSession.sessionToken

    init {
        mMediaSession = createMediaSession(mContext)
    }

    fun setCommunicatorCallback(callback: MediaSession.Callback) {
        mMediaSession.setCallback(callback)
    }

    private fun createMediaSession(context: Context): MediaSession {
        val mediaButtonReceiverComponentName = ComponentName(
            context.applicationContext,
            MediaButtonIntentReceiver::class.java
        )

        val mediaButtonIntent = Intent(Intent.ACTION_MEDIA_BUTTON).apply {
            component = mediaButtonReceiverComponentName
        }
        val mediaButtonReceiverPendingIntent = PendingIntent.getBroadcast(
            context.applicationContext, 0, mediaButtonIntent,
            PendingIntent.FLAG_IMMUTABLE
        )

        val mediaSession = MediaSession(
            context,
            "app.moosync.android"
        ).apply {
            isActive = true
            setMediaButtonReceiver(mediaButtonReceiverPendingIntent)
        }

        return mediaSession
    }

    fun updatePlayerState(isPlaying: Boolean, positionMs: Long) {
        val builder = PlaybackState.Builder()
            .setState(
                if (isPlaying) PlaybackState.STATE_PLAYING else PlaybackState.STATE_PAUSED,
                positionMs,
                1.0f
            )
            .setActions(Actions.PLAYBACK_STATE_ACTIONS)

        mMediaSession.setPlaybackState(builder.build())
    }

    fun updateMetadata(
        title: String?,
        artistName: String?,
        albumName: String?,
        durationMs: Long,
        thumbnailUri: String?
    ) {
        val builder = MediaMetadata.Builder()
            .putString(MediaMetadata.METADATA_KEY_TITLE, title)
            .putString(MediaMetadata.METADATA_KEY_ARTIST, artistName ?: "")
            .putString(MediaMetadata.METADATA_KEY_ALBUM, albumName ?: "")
            .putLong(MediaMetadata.METADATA_KEY_DURATION, durationMs)

        if (thumbnailUri != null) {
            val resolvedUri = if (thumbnailUri.startsWith("/data/data")) {
                val parts = thumbnailUri.split("/")
                File(mContext.filesDir, parts.last()).absolutePath
            } else {
                thumbnailUri
            }

            mExecutor.submit {
                try {
                    var bitmap: Bitmap? = null
                    if (resolvedUri.startsWith("http://") || resolvedUri.startsWith("https://")) {
                        val inputStream = URL(resolvedUri).openStream()
                        bitmap = BitmapFactory.decodeStream(inputStream)
                    } else {
                        bitmap = BitmapFactory.decodeFile(resolvedUri)
                    }
                    if (bitmap != null) {
                        builder.putBitmap(MediaMetadata.METADATA_KEY_ART, bitmap)
                    }
                } catch (e: Exception) {
                    Log.e("MoosyncAndroid", "Error loading thumbnail: ${e.message}")
                }
                mMediaSession.setMetadata(builder.build())
            }
        } else {
            mMediaSession.setMetadata(builder.build())
        }
    }

    fun release() {
        mMediaSession.isActive = false
        mMediaSession.release()
        mExecutor.shutdown()
    }
}
