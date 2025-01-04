package app.moosync.audioplayer.services.interfaces

interface MediaControls {
    fun play(key: String)
    fun pause(key: String)
    fun stop(key: String)

    fun seek(key: String, time: Int)

    fun load(key: String, src: String, autoplay: Boolean)
}