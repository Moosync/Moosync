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
import android.net.Uri
import android.util.Log
import app.moosync.audioplayer.models.PlaybackState
import app.moosync.audioplayer.models.Song
import com.pierfrancescosoffritti.androidyoutubeplayer.core.player.PlayerConstants
import com.pierfrancescosoffritti.androidyoutubeplayer.core.player.YouTubePlayer
import com.pierfrancescosoffritti.androidyoutubeplayer.core.player.listeners.YouTubePlayerCallback
import com.pierfrancescosoffritti.androidyoutubeplayer.core.player.listeners.YouTubePlayerListener
import com.pierfrancescosoffritti.androidyoutubeplayer.core.player.views.YouTubePlayerView

class YoutubePlayer(mContext: Context) : GenericPlayer() {
    private val _playerView = YouTubePlayerView(mContext)
    private var playerInstance: YouTubePlayer? = null
    private var isInitialized = false

    private var shouldPlayOnVideoLoad = false
    private var isLoadingVideo = false
    private val playerStateActionQueue = arrayListOf<PlaybackState>()

    private val key = "YOUTUBE"

    init {
        _playerView.getYouTubePlayerWhenReady(object : YouTubePlayerCallback {
            override fun onYouTubePlayer(youTubePlayer: YouTubePlayer) {
                playerInstance = youTubePlayer
                isInitialized = true
                playerInstance!!.addListener(object : YouTubePlayerListener {
                    override fun onApiChange(youTubePlayer: YouTubePlayer) {}

                    override fun onCurrentSecond(youTubePlayer: YouTubePlayer, second: Float) {
                        _progress = second
                        emitInListeners { it.onTimeChange(key,  (second * 1000).toInt()) }
                    }

                    override fun onError(
                        youTubePlayer: YouTubePlayer,
                        error: PlayerConstants.PlayerError
                    ) {
                    }

                    override fun onPlaybackQualityChange(
                        youTubePlayer: YouTubePlayer,
                        playbackQuality: PlayerConstants.PlaybackQuality
                    ) {
                    }

                    override fun onPlaybackRateChange(
                        youTubePlayer: YouTubePlayer,
                        playbackRate: PlayerConstants.PlaybackRate
                    ) {
                    }

                    override fun onReady(youTubePlayer: YouTubePlayer) {}

                    override fun onStateChange(
                        youTubePlayer: YouTubePlayer,
                        state: PlayerConstants.PlayerState
                    ) {
                        _isPlaying = when (state) {
                            PlayerConstants.PlayerState.UNKNOWN -> false
                            PlayerConstants.PlayerState.UNSTARTED -> false
                            PlayerConstants.PlayerState.ENDED -> {
                                emitInListeners { it.onSongEnded(key) }
                                false
                            }

                            PlayerConstants.PlayerState.PLAYING -> true
                            PlayerConstants.PlayerState.PAUSED -> false
                            PlayerConstants.PlayerState.BUFFERING -> false
                            PlayerConstants.PlayerState.VIDEO_CUED -> handleVideoLoad()
                        }

                        Log.d("TAG", "onStateChange: $state")
                    }

                    override fun onVideoDuration(youTubePlayer: YouTubePlayer, duration: Float) {
                    }

                    override fun onVideoId(youTubePlayer: YouTubePlayer, videoId: String) {
                        youTubePlayer.play()
                    }

                    override fun onVideoLoadedFraction(
                        youTubePlayer: YouTubePlayer,
                        loadedFraction: Float
                    ) {
                    }
                })
                Log.d("TAG", "onYouTubePlayer: Initialized youtube player")
            }
        })
    }

    private var _progress = 0f
    override var progress: Int
        get() = (_progress * 1000).toInt()
        set(value) {
            playerInstance?.seekTo(value.toFloat() / 1000)
            _progress = value.toFloat() / 1000
        }

    private var _isPlaying: Boolean = false
    override val isPlaying: Boolean
        get() = _isPlaying

    private fun isVideoId(videoId: Any): Boolean {
        if (videoId is String && videoId.length == 11) return true
        return false
    }

    override fun canPlay(song: Song): Boolean {
        if (song.playbackUrl != null) {
            return isVideoId(song.playbackUrl)
        }
        return false
    }

    private fun getVideoIdFromURL(url: String): String? {
        return Uri.parse(url).getQueryParameter("v")
    }

    override fun load(mContext: Context, src: String, autoPlay: Boolean) {
        val videoId = if (isVideoId(src)) src else getVideoIdFromURL(src)
        Log.d("TAG", "load: loading video $videoId")
        if (isInitialized && videoId != null) {
            playerInstance!!.cueVideo(videoId, 0F)
            playerInstance!!.setVolume(100)
            isLoadingVideo = true
            shouldPlayOnVideoLoad = autoPlay
        }
    }

    override fun play() {
        Log.d("TAG", "play: Youtube player initialized $isInitialized")
        if (isInitialized) {
            if (isLoadingVideo) {
                playerStateActionQueue.add(PlaybackState.PLAYING)
            } else {
                playerInstance!!.play()
            }
        }
    }

    override fun pause() {
        if (isInitialized) {
            if (isLoadingVideo) {
                playerStateActionQueue.add(PlaybackState.PAUSED)
            } else {
                playerInstance!!.pause()
            }
        }
    }

    override fun stop() {
        if (isInitialized) {
            if (isLoadingVideo) {
                playerStateActionQueue.add(PlaybackState.PAUSED)
            } else {
                playerInstance!!.pause()
            }
        }
    }

    override fun release() {
        Log.d("TAG", "release: releasing view")
        _playerView.release()
    }

    private var youtubePlayerListener: MutableList<PlayerListeners> = mutableListOf()

    override fun setPlayerListeners(playerListeners: PlayerListeners) {
        youtubePlayerListener.add(playerListeners)
    }

    fun emitInListeners(callback: (c: PlayerListeners) -> Unit) {
        for (listener in youtubePlayerListener) {
            callback.invoke(listener)
        }
    }

    override fun removePlayerListeners() {
        youtubePlayerListener.clear()
    }

    private fun handleVideoLoad(): Boolean {
        val tmpList = playerStateActionQueue

        var isPlaying = false
        Log.d("TAG", "handleVideoLoad: got video load")
        if (shouldPlayOnVideoLoad) {
            if (!(tmpList.contains(PlaybackState.PAUSED) || tmpList.contains(PlaybackState.STOPPED))) {
                playerInstance!!.play()
                isPlaying = true
            }
        }

        for (action in tmpList) {
            when (action) {
                PlaybackState.PLAYING -> {
                    playerInstance!!.play()
                    isPlaying = true
                }
                PlaybackState.PAUSED, PlaybackState.STOPPED -> playerInstance!!.pause()
            }
        }

        isLoadingVideo = false
        return isPlaying
    }

}
