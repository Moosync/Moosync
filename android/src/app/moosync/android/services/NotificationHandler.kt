package app.moosync.android.services

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.media.session.MediaSession
import android.media.session.PlaybackState
import android.view.KeyEvent
import app.moosync.android.services.Constants.NOTIFICATION_CHANNEL_ID
import app.moosync.android.services.Constants.NOTIFICATION_ID

class NotificationHandler(
    private val mContext: Context,
    private val mToken: MediaSession.Token,
) {
    private val mNotificationManager: NotificationManager =
        mContext.getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager

    var notification: Notification? = null
        private set

    var title: String? = null
    var artistName: String? = null
    var albumName: String? = null
    var isPlaying: Boolean = false
    var coverBitmap: Bitmap? = null

    init {
        mNotificationManager.cancelAll()
        createNotificationChannel()
        createNotification()
    }

    private fun createNotificationChannel() {
        val existingChannel = mNotificationManager.getNotificationChannel(NOTIFICATION_CHANNEL_ID)
        if (existingChannel == null) {
            val channel = NotificationChannel(
                NOTIFICATION_CHANNEL_ID,
                "Now playing",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                enableLights(false)
                enableVibration(false)
                setShowBadge(false)
            }
            mNotificationManager.createNotificationChannel(channel)
        }
    }

    private fun buildMediaButtonPendingIntent(action: Long): PendingIntent {
        val keyCode = when (action) {
            PlaybackState.ACTION_SKIP_TO_PREVIOUS -> KeyEvent.KEYCODE_MEDIA_PREVIOUS
            PlaybackState.ACTION_PLAY -> KeyEvent.KEYCODE_MEDIA_PLAY
            PlaybackState.ACTION_PAUSE -> KeyEvent.KEYCODE_MEDIA_PAUSE
            PlaybackState.ACTION_PLAY_PAUSE -> KeyEvent.KEYCODE_MEDIA_PLAY_PAUSE
            PlaybackState.ACTION_SKIP_TO_NEXT -> KeyEvent.KEYCODE_MEDIA_NEXT
            else -> KeyEvent.KEYCODE_UNKNOWN
        }

        val intent = Intent(Intent.ACTION_MEDIA_BUTTON).apply {
            setClass(mContext, MediaButtonIntentReceiver::class.java)
            putExtra(Intent.EXTRA_KEY_EVENT, KeyEvent(KeyEvent.ACTION_DOWN, keyCode))
        }

        return PendingIntent.getBroadcast(
            mContext,
            action.toInt(),
            intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
    }

    private fun createNotification() {
        val mediaStyle = Notification.MediaStyle()
            .setMediaSession(mToken)
            .setShowActionsInCompactView(0, 1, 2)

        val launchIntent = mContext.packageManager.getLaunchIntentForPackage(mContext.packageName)?.apply {
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_RESET_TASK_IF_NEEDED
        }

        val clickIntent = PendingIntent.getActivity(
            mContext,
            0,
            launchIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )

        val dismissIntent = Intent(mContext, NotificationDismissedReceiver::class.java)
        val deleteIntent = PendingIntent.getBroadcast(
            mContext,
            0,
            dismissIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )

        val prevIntent = buildMediaButtonPendingIntent(PlaybackState.ACTION_SKIP_TO_PREVIOUS)
        val playPauseAction = if (isPlaying) PlaybackState.ACTION_PAUSE else PlaybackState.ACTION_PLAY
        val playPauseIntent = buildMediaButtonPendingIntent(playPauseAction)
        val playPauseIcon = if (isPlaying) android.R.drawable.ic_media_pause else android.R.drawable.ic_media_play
        val playPauseLabel = if (isPlaying) "Pause" else "Play"
        val nextIntent = buildMediaButtonPendingIntent(PlaybackState.ACTION_SKIP_TO_NEXT)

        val builder = Notification.Builder(mContext, NOTIFICATION_CHANNEL_ID)
            .setStyle(mediaStyle)
            .setContentTitle(if (!title.isNullOrEmpty()) title else "Moosync")
            .setContentText(artistName ?: "")
            .addAction(
                Notification.Action.Builder(
                    android.R.drawable.ic_media_previous, "Previous", prevIntent
                ).build()
            )
            .addAction(
                Notification.Action.Builder(
                    playPauseIcon, playPauseLabel, playPauseIntent
                ).build()
            )
            .addAction(
                Notification.Action.Builder(
                    android.R.drawable.ic_media_next, "Next", nextIntent
                ).build()
            )
            .setVisibility(Notification.VISIBILITY_PUBLIC)
            .setSmallIcon(android.R.drawable.ic_media_play)
            .setContentIntent(clickIntent)
            .setDeleteIntent(deleteIntent)
            .setShowWhen(false)

        if (!albumName.isNullOrEmpty()) {
            builder.setSubText(albumName)
        }

        if (coverBitmap != null) {
            builder.setLargeIcon(coverBitmap)
        }

        notification = builder.build()
    }

    fun updateMetadata(title: String?, artistName: String?, albumName: String?, coverBitmap: Bitmap? = null) {
        this.title = title
        this.artistName = artistName
        this.albumName = albumName
        if (coverBitmap != null) {
            this.coverBitmap = coverBitmap
        }
        updateNotification()
    }

    fun updateCover(bitmap: Bitmap?) {
        this.coverBitmap = bitmap
        updateNotification()
    }

    fun updatePlayerState(isPlaying: Boolean) {
        this.isPlaying = isPlaying
        updateNotification()
    }

    fun clearNotification() {
        mNotificationManager.cancel(NOTIFICATION_ID)
    }

    fun updateNotification() {
        createNotification()
        notification?.let {
            mNotificationManager.notify(NOTIFICATION_ID, it)
        }
    }

    fun release() {
        mNotificationManager.cancelAll()
    }
}
