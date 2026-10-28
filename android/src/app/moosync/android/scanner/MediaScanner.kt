package app.moosync.android.scanner

import android.content.Context
import android.os.Environment
import moosync.types.AndroidScanner.ScanRequest
import moosync.types.AndroidScanner.ScanResult

object MediaScanner {
    @JvmStatic
    fun scanMedia(context: Context, requestBytes: ByteArray): ByteArray {
        val request = ScanRequest.parseFrom(requestBytes)
        val mediaStoreFilter = PathFilter(request.scanDirsList, request.excludeDirsList)
        val dirScanDirs = request.scanDirsList.ifEmpty { defaultAndroidMusicDirs() }
        val dirFilter = PathFilter(dirScanDirs, request.excludeDirsList)
        val artistSplit = request.artistSplit.ifEmpty { ";" }

        val songsByPath = linkedMapOf<String, moosync.types.Songs.Song>()
        for (song in SongScanner.scan(context, mediaStoreFilter, request.thumbnailDir, artistSplit)) {
            songsByPath[song.song.path] = song
        }
        for (song in SongScanner.scanDirectories(dirFilter, request.thumbnailDir, artistSplit, songsByPath.keys)) {
            songsByPath.putIfAbsent(song.song.path, song)
        }

        val playlists = PlaylistScanner.scan(dirFilter)

        return ScanResult.newBuilder()
            .addAllSongs(songsByPath.values)
            .addAllPlaylists(playlists)
            .build()
            .toByteArray()
    }

    @JvmStatic
    fun scanDirectories(requestBytes: ByteArray): ByteArray {
        val request = ScanRequest.parseFrom(requestBytes)
        val filter = PathFilter(request.scanDirsList, request.excludeDirsList)
        val artistSplit = request.artistSplit.ifEmpty { ";" }

        val songs = SongScanner.scanDirectories(
            filter = filter,
            thumbnailDir = request.thumbnailDir,
            artistSplit = artistSplit
        )
        val playlists = PlaylistScanner.scan(filter)

        return ScanResult.newBuilder()
            .addAllSongs(songs)
            .addAllPlaylists(playlists)
            .build()
            .toByteArray()
    }

    private fun defaultAndroidMusicDirs(): List<String> {
        return try {
            val musicDir = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_MUSIC)
            if (musicDir != null) listOf(musicDir.absolutePath) else emptyList()
        } catch (_: Throwable) {
            emptyList()
        }
    }
}
