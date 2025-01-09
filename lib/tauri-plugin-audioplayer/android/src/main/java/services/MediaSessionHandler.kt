package app.moosync.audioplayer.services

import android.R.attr.bitmap
import android.app.PendingIntent
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.drawable.Drawable
import android.net.Uri
import android.support.v4.media.MediaMetadataCompat
import android.support.v4.media.session.MediaSessionCompat
import android.support.v4.media.session.PlaybackStateCompat
import android.util.Log
import androidx.core.net.UriCompat
import androidx.core.net.toUri
import app.moosync.audioplayer.models.MetadataArgs
import app.moosync.audioplayer.models.Song
import app.moosync.audioplayer.models.toArtistString
import com.bumptech.glide.Glide
import com.bumptech.glide.request.target.CustomTarget
import com.bumptech.glide.request.target.Target
import com.bumptech.glide.request.transition.Transition
import com.bumptech.glide.signature.MediaStoreSignature
import java.io.File
import java.io.FileInputStream


class MediaSessionHandler(private val mContext: Context) {

    private val mMediaSession: MediaSessionCompat

    val sessionToken: MediaSessionCompat.Token
        get() = mMediaSession.sessionToken

    init {
        mMediaSession = createMediaSession(mContext)
    }

    fun setCommunicatorCallback(callback: MediaSessionCompat.Callback) {
        mMediaSession.setCallback(callback)
    }

    private fun createMediaSession(mContext: Context): MediaSessionCompat {
        val mediaButtonReceiverComponentName = ComponentName(
            mContext.applicationContext,
            MediaButtonIntentReceiver::class.java
        )

        val mediaButtonIntent = Intent(Intent.ACTION_MEDIA_BUTTON)
        mediaButtonIntent.component = mediaButtonReceiverComponentName
        val mediaButtonReceiverPendingIntent = PendingIntent.getBroadcast(
            mContext.applicationContext, 0, mediaButtonIntent,
            PendingIntent.FLAG_IMMUTABLE
        )

        val mediaSession = MediaSessionCompat(
            mContext,
            "app.moosync.moosync",
            mediaButtonReceiverComponentName,
            mediaButtonReceiverPendingIntent
        )

        mediaSession.isActive = true
        mediaSession.setMediaButtonReceiver(mediaButtonReceiverPendingIntent)

        return mediaSession
    }

    fun updatePlayerState(isPlaying: Boolean, position: Int = 0) {
        mMediaSession.setPlaybackState(
            PlaybackStateCompat.Builder()
                .setState(
                    if (isPlaying) PlaybackStateCompat.STATE_PLAYING else PlaybackStateCompat.STATE_PAUSED,
                    position.toLong(),
                    1F
                )
                .setActions(Actions.PLAYBACK_STATE_ACTIONS)
                .build()
        )
    }

    fun updateMetadata(song: MetadataArgs?) {
        if (song != null) {
            val builder = MediaMetadataCompat.Builder()
                .putString(MediaMetadataCompat.METADATA_KEY_TITLE, song.title)
                .putString(
                    MediaMetadataCompat.METADATA_KEY_ARTIST,
                    song.artistName ?: ""
                )
                .putLong(MediaMetadataCompat.METADATA_KEY_DURATION, song.duration)

            val thumbnail = song.thumbnail
            if (thumbnail != null) {
                var thumbnail_uri = thumbnail
                if (thumbnail.startsWith("/data/data")) {
                    thumbnail_uri = File(mContext.filesDir, thumbnail.split("/").last()).toUri().toString()
                }
                Glide.with(mContext)
                    .asBitmap()
                    .load(thumbnail_uri)
                    .into(object :
                        CustomTarget<Bitmap>(Target.SIZE_ORIGINAL, Target.SIZE_ORIGINAL) {
                        override fun onResourceReady(
                            resource: Bitmap,
                            transition: Transition<in Bitmap>?
                        ) {
                            builder.putBitmap(MediaMetadataCompat.METADATA_KEY_ART, resource)
                            mMediaSession.setMetadata(builder.build())
                        }

                        override fun onLoadCleared(placeholder: Drawable?) {}
                    })
            } else {
                mMediaSession.setMetadata(builder.build())
            }
        } else {
            mMediaSession.setMetadata(null)
        }
    }
}