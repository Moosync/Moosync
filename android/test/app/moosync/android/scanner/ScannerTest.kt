package app.moosync.android.scanner

import java.io.File
import java.nio.file.Files
import moosync.types.AndroidScanner.ScanRequest
import moosync.types.AndroidScanner.ScanResult
import org.junit.Test

private fun checkEqual(expected: Any?, actual: Any?, message: String) {
    if (expected != actual) {
        throw AssertionError("$message: expected <$expected> but was <$actual>")
    }
}

private fun checkTrue(condition: Boolean, message: String) {
    if (!condition) {
        throw AssertionError(message)
    }
}

class ScannerTest {
    @Test
    fun testPathFilterIncludesAndExcludes() {
    val root = Files.createTempDirectory("moosync_filter_test").toFile()
    try {
        val musicDir = File(root, "Music").apply { mkdirs() }
        val excludedDir = File(musicDir, "Trash").apply { mkdirs() }
        val otherDir = File(root, "Downloads").apply { mkdirs() }

        val filter = PathFilter(
            scanDirs = listOf(musicDir.absolutePath),
            excludeDirs = listOf(excludedDir.absolutePath)
        )

        checkTrue(
            filter.isAllowed(File(musicDir, "track.mp3").absolutePath),
            "Track inside Music should be allowed"
        )
        checkTrue(
            !filter.isAllowed(File(excludedDir, "deleted.mp3").absolutePath),
            "Track inside excluded dir should be rejected"
        )
        checkTrue(
            !filter.isAllowed(File(otherDir, "other.mp3").absolutePath),
            "Track outside scanDirs should be rejected"
        )

        val unrestrictedFilter = PathFilter(
            scanDirs = emptyList(),
            excludeDirs = listOf(excludedDir.absolutePath)
        )
        checkTrue(
            unrestrictedFilter.isAllowed(File(otherDir, "other.mp3").absolutePath),
            "Empty scanDirs should allow any non-excluded path"
        )
    } finally {
        root.deleteRecursively()
    }
}

    @Test
    fun testLyricsScannerParsesSyncedAndUnsyncedLrc() {
    val syncedContent = """
        [ti:Test Song]
        [offset:100]
        [00:01.50]First synced line
        [00:05.00]Second synced line
    """.trimIndent()

    val synced = LyricsScanner.parseLyricsContent(syncedContent)
    checkTrue(synced != null, "Synced lyrics should not be null")
    checkTrue(synced!!.isSynced, "Lyrics should be marked synced")
    checkEqual(2, synced.linesCount, "Synced lines count")
    checkEqual("First synced line", synced.getLines(0).text, "First line text")
    checkEqual(1400L, synced.getLines(0).timeMs, "First line timestamp with offset")
    checkEqual("Second synced line", synced.getLines(1).text, "Second line text")
    checkEqual(4900L, synced.getLines(1).timeMs, "Second line timestamp with offset")

    val unsyncedContent = """
        Plain line one
        Plain line two
    """.trimIndent()

    val unsynced = LyricsScanner.parseLyricsContent(unsyncedContent)
    checkTrue(unsynced != null, "Unsynced lyrics should not be null")
    checkTrue(!unsynced!!.isSynced, "Unsynced lyrics should have isSynced=false")
    checkEqual(2, unsynced.linesCount, "Unsynced lines count")
    checkEqual("Plain line one", unsynced.getLines(0).text, "First unsynced line")
}

    @Test
    fun testPlaylistScannerParsesM3u() {
    val root = Files.createTempDirectory("moosync_playlist_test").toFile()
    try {
        val songFile = File(root, "song1.mp3").apply { writeBytes(byteArrayOf(1, 2, 3)) }
        val m3uFile = File(root, "favorites.m3u8").apply {
            writeText(
                """
                #EXTM3U
                #EXTINF:123,Song 1
                song1.mp3
                https://stream.example.com/live.ogg
                """.trimIndent()
            )
        }

        val filter = PathFilter(listOf(root.absolutePath), emptyList())
        val playlists = PlaylistScanner.scan(filter)

        checkEqual(1, playlists.size, "Should scan 1 playlist")
        val scanned = playlists[0]
        checkEqual("favorites", scanned.playlist.playlistName, "Playlist name")
        checkEqual(2, scanned.songPathsCount, "Playlist song count")
        checkEqual(songFile.canonicalPath, scanned.getSongPaths(0), "Resolved relative song path")
        checkEqual("https://stream.example.com/live.ogg", scanned.getSongPaths(1), "Stream URL")
    } finally {
        root.deleteRecursively()
    }
}

    @Test
    fun testCoverScannerFindsSiblingCoverAndCaches() {
    val root = Files.createTempDirectory("moosync_cover_test").toFile()
    val thumbDir = Files.createTempDirectory("moosync_thumb_test").toFile()
    try {
        val songFile = File(root, "track.flac").apply { writeBytes(byteArrayOf(0x66, 0x4C, 0x61, 0x43)) }
        val coverBytes = byteArrayOf(0x89.toByte(), 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A)
        File(root, "cover.png").writeBytes(coverBytes)

        val covers = CoverScanner.extractAndCache(
            context = null,
            songPath = songFile.absolutePath,
            albumId = null,
            thumbnailDir = thumbDir.absolutePath
        )

        checkTrue(covers != null, "Covers should be extracted from sibling cover.png")
        val expectedHash = CoverScanner.sha256Hex(coverBytes)
        checkTrue(covers!!.highPath.endsWith("$expectedHash.png"), "High cover path hash")
        checkTrue(covers.lowPath.endsWith("$expectedHash-low.png"), "Low cover path hash")
        checkTrue(File(covers.highPath).exists(), "High cover file exists")
        checkTrue(File(covers.lowPath).exists(), "Low cover file exists")
    } finally {
        root.deleteRecursively()
        thumbDir.deleteRecursively()
    }
}

    @Test
    fun testMediaScannerDirectoriesProtobufRoundTrip() {
    val root = Files.createTempDirectory("moosync_e2e_kt").toFile()
    val thumbDir = Files.createTempDirectory("moosync_e2e_thumbs").toFile()
    try {
        val albumDir = File(root, "AlbumA").apply { mkdirs() }
        val excludedDir = File(root, "Excluded").apply { mkdirs() }

        val songFile = File(albumDir, "My Song.mp3").apply {
            writeBytes(ByteArray(256) { it.toByte() })
        }
        File(albumDir, "My Song.lrc").writeText("[00:02.00]Hello from LRC")
        File(albumDir, "cover.jpg").writeBytes(byteArrayOf(10, 20, 30, 40))
        File(excludedDir, "Ignored.mp3").writeBytes(ByteArray(64))

        File(albumDir, "mix.m3u").writeText("My Song.mp3\nhttps://example.com/stream.mp3\n")

        val request = ScanRequest.newBuilder()
            .addScanDirs(root.absolutePath)
            .addExcludeDirs(excludedDir.absolutePath)
            .setThumbnailDir(thumbDir.absolutePath)
            .setArtistSplit(";")
            .build()

        val resultBytes = MediaScanner.scanDirectories(request.toByteArray())
        val result = ScanResult.parseFrom(resultBytes)

        checkEqual(1, result.songsCount, "Should find 1 non-excluded song")
        val song = result.getSongs(0)
        checkEqual("My Song", song.song.title, "Scanned song title")
        checkEqual(songFile.canonicalPath, song.song.path, "Scanned song path")
        checkEqual(256.0, song.song.size, "Scanned song size")
        checkTrue(song.song.hasLyrics(), "Scanned song should have lyrics")
        checkTrue(song.song.lyrics.isSynced, "Scanned song lyrics should be synced")
        checkEqual("Hello from LRC", song.song.lyrics.getLines(0).text, "Lyric line text")
        checkEqual(2000L, song.song.lyrics.getLines(0).timeMs, "Lyric line timeMs")
        checkTrue(song.song.songCoverPathHigh.isNotEmpty(), "High cover path populated")
        checkTrue(song.song.songCoverPathLow.isNotEmpty(), "Low cover path populated")

        checkEqual(1, result.playlistsCount, "Should find 1 playlist")
        val playlist = result.getPlaylists(0)
        checkEqual("mix", playlist.playlist.playlistName, "Scanned playlist name")
        checkEqual(2, playlist.songPathsCount, "Scanned playlist song paths count")
    } finally {
        root.deleteRecursively()
        thumbDir.deleteRecursively()
    }
}
}
