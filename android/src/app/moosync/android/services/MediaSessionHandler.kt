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
import java.net.URI
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

    fun resolveThumbnailFile(uri: String): File? = resolveThumbnailFile(mContext.filesDir, uri)

    companion object {
        fun resolveThumbnailFile(filesDir: File?, uri: String): File? {
            val cleanPath = if (uri.startsWith("file://")) uri.removePrefix("file://") else uri
            val directFile = File(cleanPath)
            if (directFile.exists()) return directFile

            val appDataDir = filesDir?.parentFile
            if (appDataDir != null) {
                if (cleanPath.startsWith("/data/data/")) {
                    val subPath = cleanPath.replaceFirst("^/data/data/[^/]+/?".toRegex(), "")
                    val altFile = File(appDataDir, subPath)
                    if (altFile.exists()) return altFile
                } else if (cleanPath.startsWith("/data/user/")) {
                    val subPath = cleanPath.replaceFirst("^/data/user/[0-9]+/[^/]+/?".toRegex(), "")
                    val altFile = File(appDataDir, subPath)
                    if (altFile.exists()) return altFile
                }
            }
            return null
        }
    }

    private fun loadThumbnailBitmap(uri: String): Bitmap? {
        return try {
            if (uri.startsWith("http://") || uri.startsWith("https://")) {
                URI.create(uri).toURL().openStream().use { inputStream ->
                    BitmapFactory.decodeStream(inputStream)
                }
            } else if (uri.startsWith("content://")) {
                mContext.contentResolver.openInputStream(android.net.Uri.parse(uri))?.use { inputStream ->
                    BitmapFactory.decodeStream(inputStream)
                }
            } else {
                val file = resolveThumbnailFile(uri)
                if (file != null && file.exists()) {
                    BitmapFactory.decodeFile(file.absolutePath)
                } else {
                    Log.w("MoosyncAndroid", "Thumbnail file does not exist: $uri")
                    null
                }
            }
        } catch (e: Exception) {
            Log.e("MoosyncAndroid", "Error loading thumbnail: ${e.message}")
            null
        }
    }

    fun updateMetadata(
        title: String?,
        artistName: String?,
        albumName: String?,
        durationMs: Long,
        thumbnailUri: String?,
        onBitmapLoaded: ((Bitmap?) -> Unit)? = null
    ) {
        val builder = MediaMetadata.Builder()
            .putString(MediaMetadata.METADATA_KEY_TITLE, title)
            .putString(MediaMetadata.METADATA_KEY_ARTIST, artistName ?: "")
            .putString(MediaMetadata.METADATA_KEY_ALBUM, albumName ?: "")
            .putLong(MediaMetadata.METADATA_KEY_DURATION, durationMs)

        if (thumbnailUri != null) {
            mExecutor.submit {
                val bitmap = loadThumbnailBitmap(thumbnailUri)
                if (bitmap != null) {
                    builder.putBitmap(MediaMetadata.METADATA_KEY_ART, bitmap)
                }
                mMediaSession.setMetadata(builder.build())
                onBitmapLoaded?.invoke(bitmap)
            }
        } else {
            mMediaSession.setMetadata(builder.build())
            onBitmapLoaded?.invoke(null)
        }
    }

    fun release() {
        mMediaSession.isActive = false
        mMediaSession.release()
        mExecutor.shutdown()
    }
}
