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

package app.moosync.filescanner.utils

import java.io.Serializable

data class Artist( val artist_name: String, val artist_coverpath: Any?) : Serializable

data class Album(val album_name: String, val album_coverpath_high: String?, val album_coverpath_low: String?) : Serializable

data class Genre(val genre_name: String) : Serializable

data class Song(
        val title: String,
        val duration: Long,
        val path: String?,
        val artist: List<Artist>?,
        val album: Album?,
        val genre: List<Genre>?,
        val playbackUrl: String?,
        val song_coverPath_low: String?,
        val song_coverPath_high: String?,
        val type: String
) : Serializable
