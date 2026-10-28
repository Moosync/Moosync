package app.moosync.android.scanner

import java.io.File
import moosync.types.Songs.LyricLine
import moosync.types.Songs.Lyrics

object LyricsScanner {
    private val TIMESTAMP_REGEX = Regex("""\[(\d+):(\d+)(?:\.(\d+))?]""")
    private val OFFSET_REGEX = Regex("""(?i)\[offset:\s*([+-]?\d+)\s*]""")
    private val METADATA_TAG_REGEX = Regex("""^\[[a-zA-Z]+:.*]$""")

    fun extractFromSongPath(songPath: String, embeddedLyrics: String? = null): Lyrics? {
        if (!embeddedLyrics.isNullOrBlank()) {
            parseLyricsContent(embeddedLyrics)?.let { return it }
        }
        return scanLrcFile(File(songPath))
    }

    fun scanLrcFile(songFile: File): Lyrics? {
        val parent = songFile.parentFile ?: return null
        val lrcFile = File(parent, "${songFile.nameWithoutExtension}.lrc")
        if (!lrcFile.isFile || !lrcFile.canRead()) {
            return null
        }
        return try {
            parseLyricsContent(lrcFile.readText())
        } catch (_: Exception) {
            null
        }
    }

    fun parseLyricsContent(content: String): Lyrics? {
        val trimmed = content.trim()
        if (trimmed.isEmpty()) return null

        var offsetMs = 0L
        val syncedLines = mutableListOf<LyricLine>()
        val rawLines = mutableListOf<String>()
        var hasSyncedTag = false

        for (rawLine in content.lineSequence()) {
            val line = rawLine.trim()
            if (line.isEmpty()) continue

            val offsetMatch = OFFSET_REGEX.find(line)
            if (offsetMatch != null) {
                offsetMs = offsetMatch.groupValues[1].toLongOrNull() ?: 0L
                continue
            }

            val timestamps = TIMESTAMP_REGEX.findAll(line).toList()
            if (timestamps.isNotEmpty()) {
                hasSyncedTag = true
                val lyricText = line.substring(timestamps.last().range.last + 1).trim()
                for (match in timestamps) {
                    val minutes = match.groupValues[1].toLongOrNull() ?: 0L
                    val seconds = match.groupValues[2].toLongOrNull() ?: 0L
                    val fracStr = match.groupValues[3]
                    val millis = when (fracStr.length) {
                        0 -> 0L
                        1 -> (fracStr.toLongOrNull() ?: 0L) * 100L
                        2 -> (fracStr.toLongOrNull() ?: 0L) * 10L
                        else -> fracStr.take(3).toLongOrNull() ?: 0L
                    }
                    val totalMs = (minutes * 60_000L + seconds * 1_000L + millis - offsetMs)
                        .coerceAtLeast(0L)
                    syncedLines.add(
                        LyricLine.newBuilder()
                            .setText(lyricText)
                            .setTimeMs(totalMs)
                            .build()
                    )
                }
            } else if (!METADATA_TAG_REGEX.matches(line)) {
                rawLines.add(line)
            }
        }

        if (hasSyncedTag) {
            if (syncedLines.isEmpty()) return null
            val sorted = syncedLines.sortedBy { it.timeMs }
            return Lyrics.newBuilder()
                .addAllLines(sorted)
                .setIsSynced(true)
                .build()
        }

        if (rawLines.isEmpty()) return null
        val unsynced = rawLines.map { text ->
            LyricLine.newBuilder()
                .setText(text)
                .setTimeMs(0L)
                .build()
        }
        return Lyrics.newBuilder()
            .addAllLines(unsynced)
            .setIsSynced(false)
            .build()
    }
}
