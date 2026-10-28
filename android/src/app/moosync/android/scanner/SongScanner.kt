package app.moosync.android.scanner

import android.content.Context
import android.media.MediaMetadataRetriever
import android.os.Build
import android.provider.MediaStore
import com.google.protobuf.Duration
import java.io.File
import java.util.UUID
import moosync.types.Songs.Album
import moosync.types.Songs.Artist
import moosync.types.Songs.Genre
import moosync.types.Songs.InnerSong
import moosync.types.Songs.Song
import moosync.types.Songs.SongType

object SongScanner {
    private val AUDIO_EXTENSIONS = setOf(
        "flac", "mp3", "ogg", "m4a", "webm", "wav", "wv", "aac", "opus"
    )

    fun scan(
        context: Context,
        filter: PathFilter,
        thumbnailDir: String,
        artistSplit: String
    ): List<Song> {
        val songsByPath = linkedMapOf<String, Song>()

        for (song in queryMediaStore(context, filter, thumbnailDir, artistSplit)) {
            val path = song.song.path
            if (path.isNotEmpty()) {
                songsByPath[path] = song
            }
        }

        for (song in scanDirectories(filter, thumbnailDir, artistSplit, songsByPath.keys)) {
            val path = song.song.path
            if (path.isNotEmpty() && !songsByPath.containsKey(path)) {
                songsByPath[path] = song
            }
        }

        return songsByPath.values.toList()
    }

    fun scanDirectories(
        filter: PathFilter,
        thumbnailDir: String,
        artistSplit: String,
        alreadyScannedPaths: Set<String> = emptySet()
    ): List<Song> {
        val audioFiles = mutableSetOf<File>()
        for (dirPath in filter.normalizedScanDirs) {
            collectAudioFiles(File(dirPath), filter, audioFiles)
        }
        return audioFiles.mapNotNull { file ->
            val canonical = canonicalOrAbsolute(file)
            if (canonical in alreadyScannedPaths) {
                null
            } else {
                buildSongFromFile(file, canonical, thumbnailDir, artistSplit)
            }
        }
    }

    private fun queryMediaStore(
        context: Context,
        filter: PathFilter,
        thumbnailDir: String,
        artistSplit: String
    ): List<Song> {
        val songs = mutableListOf<Song>()
        val projection = mutableListOf(
            MediaStore.Audio.Media._ID,
            MediaStore.Audio.Media.DATA,
            MediaStore.Audio.Media.SIZE,
            MediaStore.Audio.Media.TITLE,
            MediaStore.Audio.Media.ARTIST,
            MediaStore.Audio.Media.ALBUM,
            MediaStore.Audio.Media.ALBUM_ID,
            MediaStore.Audio.Media.ALBUM_ARTIST,
            MediaStore.Audio.Media.DURATION,
            MediaStore.Audio.Media.YEAR,
            MediaStore.Audio.Media.TRACK,
            MediaStore.Audio.Media.DATE_ADDED
        )
        if (Build.VERSION.SDK_INT >= 30) {
            projection.add(MediaStore.Audio.Media.GENRE)
            projection.add(MediaStore.Audio.Media.BITRATE)
        }

        val selection = "${MediaStore.Audio.Media.IS_MUSIC} != 0"
        try {
            context.contentResolver.query(
                MediaStore.Audio.Media.EXTERNAL_CONTENT_URI,
                projection.toTypedArray(),
                selection,
                null,
                null
            )?.use { cursor ->
                val dataCol = cursor.getColumnIndex(MediaStore.Audio.Media.DATA)
                val sizeCol = cursor.getColumnIndex(MediaStore.Audio.Media.SIZE)
                val titleCol = cursor.getColumnIndex(MediaStore.Audio.Media.TITLE)
                val artistCol = cursor.getColumnIndex(MediaStore.Audio.Media.ARTIST)
                val albumCol = cursor.getColumnIndex(MediaStore.Audio.Media.ALBUM)
                val albumIdCol = cursor.getColumnIndex(MediaStore.Audio.Media.ALBUM_ID)
                val albumArtistCol = cursor.getColumnIndex(MediaStore.Audio.Media.ALBUM_ARTIST)
                val durationCol = cursor.getColumnIndex(MediaStore.Audio.Media.DURATION)
                val yearCol = cursor.getColumnIndex(MediaStore.Audio.Media.YEAR)
                val trackCol = cursor.getColumnIndex(MediaStore.Audio.Media.TRACK)
                val dateAddedCol = cursor.getColumnIndex(MediaStore.Audio.Media.DATE_ADDED)
                val genreCol = if (Build.VERSION.SDK_INT >= 30) {
                    cursor.getColumnIndex(MediaStore.Audio.Media.GENRE)
                } else -1
                val bitrateCol = if (Build.VERSION.SDK_INT >= 30) {
                    cursor.getColumnIndex(MediaStore.Audio.Media.BITRATE)
                } else -1

                while (cursor.moveToNext()) {
                    val rawPath = if (dataCol >= 0) cursor.getString(dataCol) else null
                    if (rawPath.isNullOrBlank()) continue
                    val canonicalPath = canonicalOrAbsolute(File(rawPath))
                    if (!filter.isAllowed(canonicalPath)) continue

                    val size = if (sizeCol >= 0) cursor.getLong(sizeCol).toDouble() else 0.0
                    val title = if (titleCol >= 0) cursor.getString(titleCol) else null
                    val artist = if (artistCol >= 0) cursor.getString(artistCol) else null
                    val albumName = if (albumCol >= 0) cursor.getString(albumCol) else null
                    val albumId = if (albumIdCol >= 0) cursor.getLong(albumIdCol) else null
                    val albumArtist = if (albumArtistCol >= 0) cursor.getString(albumArtistCol) else null
                    val durationMs = if (durationCol >= 0) cursor.getLong(durationCol) else 0L
                    val year = if (yearCol >= 0) cursor.getInt(yearCol).takeIf { it > 0 }?.toString() else null
                    val trackRaw = if (trackCol >= 0) cursor.getInt(trackCol) else 0
                    val trackNo = (trackRaw % 1000).takeIf { it > 0 }?.toDouble()
                    val dateAdded = if (dateAddedCol >= 0) cursor.getLong(dateAddedCol) else 0L
                    val genreName = if (genreCol >= 0) cursor.getString(genreCol) else null
                    val bitrate = if (bitrateCol >= 0) cursor.getLong(bitrateCol).takeIf { it > 0 }?.toDouble() else null

                    val covers = CoverScanner.extractAndCache(
                        context = context,
                        songPath = canonicalPath,
                        albumId = albumId,
                        thumbnailDir = thumbnailDir
                    )
                    val lyrics = LyricsScanner.extractFromSongPath(canonicalPath)

                    songs.add(
                        assembleSong(
                            path = canonicalPath,
                            size = size,
                            title = title?.takeIf { it.isNotBlank() } ?: File(canonicalPath).name,
                            artistStr = artist?.takeIf { it != "<unknown>" },
                            artistSplit = artistSplit,
                            albumName = albumName?.takeIf { it != "<unknown>" },
                            albumArtist = albumArtist?.takeIf { it != "<unknown>" },
                            genreName = genreName?.takeIf { it.isNotBlank() },
                            durationMs = durationMs,
                            year = year,
                            trackNo = trackNo,
                            bitrate = bitrate,
                            sampleRate = null,
                            dateAdded = dateAdded,
                            covers = covers,
                            lyrics = lyrics
                        )
                    )
                }
            }
        } catch (_: Throwable) {
        }
        return songs
    }

    private fun buildSongFromFile(
        file: File,
        canonicalPath: String,
        thumbnailDir: String,
        artistSplit: String
    ): Song {
        var title: String? = null
        var artist: String? = null
        var albumName: String? = null
        var albumArtist: String? = null
        var genreName: String? = null
        var year: String? = null
        var trackNo: Double? = null
        var durationMs = 0L
        var bitrate: Double? = null
        var sampleRate: Double? = null
        var embeddedCover: ByteArray? = null

        try {
            val retriever = MediaMetadataRetriever()
            try {
                retriever.setDataSource(canonicalPath)
                title = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_TITLE)
                artist = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_ARTIST)
                albumName = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_ALBUM)
                albumArtist = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_ALBUMARTIST)
                genreName = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_GENRE)
                year = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_YEAR)
                trackNo = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_CD_TRACK_NUMBER)
                    ?.substringBefore('/')
                    ?.trim()
                    ?.toDoubleOrNull()
                durationMs = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_DURATION)
                    ?.toLongOrNull() ?: 0L
                bitrate = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_BITRATE)
                    ?.toDoubleOrNull()
                if (Build.VERSION.SDK_INT >= 31) {
                    sampleRate = retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_SAMPLERATE)
                        ?.toDoubleOrNull()
                }
                embeddedCover = retriever.embeddedPicture
            } finally {
                retriever.release()
            }
        } catch (_: Throwable) {
        }

        val covers = CoverScanner.extractAndCache(
            context = null,
            songPath = canonicalPath,
            albumId = null,
            thumbnailDir = thumbnailDir,
            embeddedBytes = embeddedCover
        )
        val lyrics = LyricsScanner.extractFromSongPath(canonicalPath)

        return assembleSong(
            path = canonicalPath,
            size = file.length().toDouble(),
            title = title?.takeIf { it.isNotBlank() } ?: file.nameWithoutExtension,
            artistStr = artist,
            artistSplit = artistSplit,
            albumName = albumName,
            albumArtist = albumArtist,
            genreName = genreName,
            durationMs = durationMs,
            year = year,
            trackNo = trackNo,
            bitrate = bitrate,
            sampleRate = sampleRate,
            dateAdded = file.lastModified() / 1000L,
            covers = covers,
            lyrics = lyrics
        )
    }

    private fun assembleSong(
        path: String,
        size: Double,
        title: String,
        artistStr: String?,
        artistSplit: String,
        albumName: String?,
        albumArtist: String?,
        genreName: String?,
        durationMs: Long,
        year: String?,
        trackNo: Double?,
        bitrate: Double?,
        sampleRate: Double?,
        dateAdded: Long,
        covers: CoverPaths?,
        lyrics: moosync.types.Songs.Lyrics?
    ): Song {
        val durationProto = Duration.newBuilder()
            .setSeconds(durationMs / 1000L)
            .setNanos(((durationMs % 1000L) * 1_000_000L).toInt())
            .build()

        val innerBuilder = InnerSong.newBuilder()
            .setId(UUID.randomUUID().toString())
            .setPath(path)
            .setSize(size)
            .setTitle(title)
            .setDuration(durationProto)
            .setType(SongType.LOCAL)
            .setDateAdded(dateAdded)

        year?.takeIf { it.isNotBlank() }?.let { innerBuilder.setYear(it) }
        trackNo?.let { innerBuilder.setTrackNo(it) }
        bitrate?.let { innerBuilder.setBitrate(it) }
        sampleRate?.let { innerBuilder.setSampleRate(it) }
        covers?.let {
            innerBuilder.setSongCoverPathHigh(it.highPath)
            innerBuilder.setSongCoverPathLow(it.lowPath)
        }
        lyrics?.let { innerBuilder.setLyrics(it) }

        val songBuilder = Song.newBuilder()
            .setSong(innerBuilder.build())

        if (!albumName.isNullOrBlank()) {
            val albumBuilder = Album.newBuilder()
                .setAlbumId(UUID.randomUUID().toString())
                .setAlbumName(albumName)
            albumArtist?.takeIf { it.isNotBlank() }?.let { albumBuilder.setAlbumArtist(it) }
            covers?.let {
                albumBuilder.setAlbumCoverpathHigh(it.highPath)
                albumBuilder.setAlbumCoverpathLow(it.lowPath)
            }
            year?.takeIf { it.isNotBlank() }?.let { albumBuilder.setYear(it) }
            songBuilder.setAlbum(albumBuilder.build())
        }

        val delimiter = artistSplit.ifEmpty { ";" }
        if (!artistStr.isNullOrBlank()) {
            val artists = artistStr.split(delimiter)
                .map { it.trim() }
                .filter { it.isNotEmpty() }
                .map { name ->
                    Artist.newBuilder()
                        .setArtistId(UUID.randomUUID().toString())
                        .setArtistName(name)
                        .build()
                }
            songBuilder.addAllArtists(artists)
        }

        if (!genreName.isNullOrBlank()) {
            songBuilder.addGenre(
                Genre.newBuilder()
                    .setGenreName(genreName.trim())
                    .build()
            )
        }

        return songBuilder.build()
    }

    private fun collectAudioFiles(dir: File, filter: PathFilter, out: MutableSet<File>) {
        if (!dir.exists()) return
        if (dir.isFile) {
            if (isAudioFile(dir) && filter.isAllowed(dir.absolutePath)) {
                out.add(dir)
            }
            return
        }
        val children = dir.listFiles() ?: return
        for (child in children) {
            if (!filter.isAllowed(child.absolutePath)) continue
            if (child.isDirectory) {
                collectAudioFiles(child, filter, out)
            } else if (isAudioFile(child)) {
                out.add(child)
            }
        }
    }

    private fun isAudioFile(file: File): Boolean {
        return file.extension.lowercase() in AUDIO_EXTENSIONS
    }

    private fun canonicalOrAbsolute(file: File): String {
        return try {
            file.canonicalPath
        } catch (_: Exception) {
            file.absolutePath
        }
    }
}
