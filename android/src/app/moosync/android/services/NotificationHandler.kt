package app.moosync.android.services

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
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
        var keyCode = KeyEvent.KEYCODE_UNKNOWN
        if (action == PlaybackState.ACTION_SKIP_TO_PREVIOUS) {
            keyCode = KeyEvent.KEYCODE_MEDIA_PREVIOUS
        } else if (action == PlaybackState.ACTION_PAUSE || action == PlaybackState.ACTION_PLAY_PAUSE) {
            keyCode = KeyEvent.KEYCODE_MEDIA_PLAY_PAUSE
        } else if (action == PlaybackState.ACTION_SKIP_TO_NEXT) {
            keyCode = KeyEvent.KEYCODE_MEDIA_NEXT
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

        val prevIntent = buildMediaButtonPendingIntent(PlaybackState.ACTION_SKIP_TO_PREVIOUS)
        val pauseIntent = buildMediaButtonPendingIntent(PlaybackState.ACTION_PAUSE)
        val nextIntent = buildMediaButtonPendingIntent(PlaybackState.ACTION_SKIP_TO_NEXT)

        val builder = Notification.Builder(mContext, NOTIFICATION_CHANNEL_ID)
            .setStyle(mediaStyle)
            .addAction(
                Notification.Action.Builder(
                    android.R.drawable.ic_media_previous, "Previous", prevIntent
                ).build()
            )
            .addAction(
                Notification.Action.Builder(
                    android.R.drawable.ic_media_pause, "Pause", pauseIntent
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
            .setShowWhen(false)

        notification = builder.build()
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
