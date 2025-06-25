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

package app.moosync.audioplayer.services.interfaces

import app.moosync.audioplayer.models.MetadataArgs
import app.moosync.audioplayer.models.PlaybackState
import app.moosync.audioplayer.models.Song

interface MediaControls {
    fun play(key: String)
    fun pause(key: String)
    fun stop(key: String)

    fun seek(key: String, time: Int)

    fun load(key: String, src: String, autoplay: Boolean)

    fun updateMetadata(metadata: MetadataArgs?)
    fun updatePlayerState(isPlaying: Boolean, pos: Int)
}
