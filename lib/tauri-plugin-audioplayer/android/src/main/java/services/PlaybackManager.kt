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

package app.moosync.audioplayer.services

import android.content.Context
import android.util.Log
import app.moosync.audioplayer.models.Song
import app.moosync.audioplayer.services.players.GenericPlayer
import app.moosync.audioplayer.services.players.LibrespotPlayer
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
        players[key]?.pause()
    }

    fun play(key: String) {
        players[key]?.play()
    }

    fun seek(key: String, pos: Int) {
        players[key]?.progress = pos
    }

    fun canPlay(key: String, song: Song): Boolean {
        return players[key]?.canPlay(song) == true
    }

    fun load(key: String, context: Context, src: String, autoPlay: Boolean) {
        players[key]?.load(context, src, autoPlay)
    }

    fun initializeLibrespot(token: String) {
        val librespot = players["LIBRESPOT"]
        if (librespot is LibrespotPlayer) {
            librespot.initialize(token)
        }
    }
}
