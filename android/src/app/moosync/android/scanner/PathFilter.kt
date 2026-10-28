package app.moosync.android.scanner

import java.io.File

class PathFilter(
    scanDirs: List<String>,
    excludeDirs: List<String>
) {
    val normalizedScanDirs: List<String> = scanDirs
        .filter { it.isNotBlank() }
        .map { normalizePath(it) }

    val normalizedExcludeDirs: List<String> = excludeDirs
        .filter { it.isNotBlank() }
        .map { normalizePath(it) }

    fun isAllowed(path: String): Boolean {
        if (path.isBlank()) return false
        val normalized = normalizePath(path)

        if (normalizedExcludeDirs.any { isUnderDir(normalized, it) }) {
            return false
        }

        if (normalizedScanDirs.isEmpty()) {
            return true
        }

        return normalizedScanDirs.any { isUnderDir(normalized, it) }
    }

    private fun isUnderDir(filePath: String, dirPath: String): Boolean {
        return filePath == dirPath || filePath.startsWith("$dirPath/")
    }

    private fun normalizePath(raw: String): String {
        return try {
            File(raw).canonicalPath.trimEnd('/')
        } catch (_: Exception) {
            File(raw).absolutePath.trimEnd('/')
        }
    }
}
