package app.moosync.audioplayer.services

import android.app.PendingIntent
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.support.v4.media.session.MediaSessionCompat
import android.support.v4.media.session.PlaybackStateCompat
import app.moosync.audioplayer.R
import app.tauri.BuildConfig

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
}