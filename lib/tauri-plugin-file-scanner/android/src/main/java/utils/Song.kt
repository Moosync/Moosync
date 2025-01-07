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
