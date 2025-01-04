package app.moosync.audioplayer.services

import android.content.Context
import android.util.Log
import app.moosync.audioplayer.models.Song
import app.moosync.audioplayer.services.players.GenericPlayer
import app.moosync.audioplayer.services.players.LocalPlayer
import app.moosync.audioplayer.services.players.PlayerListeners
import app.moosync.audioplayer.services.players.YoutubePlayer

class PlaybackManager(mContext: Context, private val playerListeners: PlayerListeners) {
    private val players: HashMap<String, GenericPlayer> = hashMapOf(Pair("LOCAL", LocalPlayer()), Pair("YOUTUBE", YoutubePlayer(mContext)))

    init {
        for (player in players.values) {
            player.setPlayerListeners(playerListeners)
        }
    }

    fun stop(key: String) {
        players[key]?.stop()
    }

    fun release() {
        players.forEach {
            it.value.release()
        }
    }

    fun pause(key: String) {
        players[key]?.stop()
    }

    fun play(key: String) {
        players[key]?.stop()
    }

    fun seek(key: String, pos: Int) {
        players[key]?.progress = pos
    }

    fun canPlay(key: String, song: Song): Boolean {
        return players[key]?.canPlay(song) == true
    }

    fun load(key: String, context: Context, src: String, autoPlay: Boolean) {
        Log.d("TAG", "load: loading in service $key, $src, $autoPlay")
        players[key]?.load(context, src, autoPlay)
    }
}