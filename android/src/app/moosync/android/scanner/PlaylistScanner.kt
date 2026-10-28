package app.moosync.android.scanner

import java.io.File
import java.util.UUID
import moosync.types.AndroidScanner.ScannedPlaylist
import moosync.types.Songs.Playlist

object PlaylistScanner {
    private val PLAYLIST_EXTENSIONS = setOf("m3u", "m3u8")

    fun scan(filter: PathFilter): List<ScannedPlaylist> {
        val playlistFiles = mutableSetOf<File>()
        for (dirPath in filter.normalizedScanDirs) {
            collectPlaylistFiles(File(dirPath), filter, playlistFiles)
        }
        return playlistFiles.mapNotNull { parsePlaylistFile(it) }
    }

    private fun collectPlaylistFiles(dir: File, filter: PathFilter, out: MutableSet<File>) {
        if (!dir.exists()) return
        if (dir.isFile) {
            if (isPlaylistFile(dir) && filter.isAllowed(dir.absolutePath)) {
                out.add(dir)
            }
            return
        }
        val children = dir.listFiles() ?: return
        for (child in children) {
            if (!filter.isAllowed(child.absolutePath)) continue
            if (child.isDirectory) {
                collectPlaylistFiles(child, filter, out)
            } else if (isPlaylistFile(child)) {
                out.add(child)
            }
        }
    }

    private fun isPlaylistFile(file: File): Boolean {
        return file.extension.lowercase() in PLAYLIST_EXTENSIONS
    }

    fun parsePlaylistFile(file: File): ScannedPlaylist? {
        if (!file.isFile || !file.canRead()) return null
        val parentDir = file.parentFile ?: File(".")
        val songPaths = mutableListOf<String>()

        val lines = try {
            file.readLines()
        } catch (_: Exception) {
            return null
        }

        for (rawLine in lines) {
            val line = rawLine.trim()
            if (line.isEmpty() || line.startsWith("#")) continue
            if (line.startsWith("http://") || line.startsWith("https://")) {
                songPaths.add(line)
            } else {
                val clean = line.removePrefix("file://")
                val candidate = File(clean)
                val resolved = if (candidate.isAbsolute) {
                    candidate
                } else {
                    File(parentDir, clean)
                }
                val normalizedPath = try {
                    resolved.canonicalPath
                } catch (_: Exception) {
                    resolved.absolutePath
                }
                songPaths.add(normalizedPath)
            }
        }

        val canonicalPlaylistPath = try {
            file.canonicalPath
        } catch (_: Exception) {
            file.absolutePath
        }

        val playlist = Playlist.newBuilder()
            .setPlaylistId(UUID.randomUUID().toString())
            .setPlaylistName(file.nameWithoutExtension)
            .setPlaylistPath(canonicalPlaylistPath)
            .setPlaylistSongCount(songPaths.size.toDouble())
            .build()

        return ScannedPlaylist.newBuilder()
            .setPlaylist(playlist)
            .addAllSongPaths(songPaths)
            .build()
    }
}
