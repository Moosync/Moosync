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
import app.moosync.audioplayer.models.Song

abstract class GenericPlayer {
    abstract fun canPlay(song: Song): Boolean
    abstract fun load(mContext: Context, src: String, autoPlay: Boolean)

    abstract fun play()
    abstract fun pause()
    abstract fun stop()
    abstract fun release()

    abstract var progress: Int
    abstract val isPlaying: Boolean

    abstract fun setPlayerListeners(playerListeners: PlayerListeners)
    abstract fun removePlayerListeners()
}

interface PlayerListeners {
    fun onSongEnded(key: String)
    fun onTimeChange(key: String, time: Int)
}
