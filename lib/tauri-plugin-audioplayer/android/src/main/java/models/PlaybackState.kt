package app.moosync.audioplayer.models

enum class PlaybackState {
    PLAYING,
    PAUSED,
    STOPPED
}

class MetadataArgs {
    lateinit var id: String
    lateinit var title: String
    var artistName: String? = null
    var albumName: String? = null
    var albumArtist: String? = null
    var genres: List<String>? = null
    var duration: Long = 0
    var thumbnail: String? = null
}