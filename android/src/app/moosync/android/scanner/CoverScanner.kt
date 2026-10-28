package app.moosync.android.scanner

import android.content.ContentUris
import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.media.MediaMetadataRetriever
import android.net.Uri
import java.io.File
import java.io.FileOutputStream
import java.security.MessageDigest

data class CoverPaths(
    val highPath: String,
    val lowPath: String
)

object CoverScanner {
    private const val ALBUM_ART_URI_STR = "content://media/external/audio/albumart"
    private val COVER_EXTENSIONS = setOf("jpg", "jpeg", "png", "webp", "bmp")

    fun extractAndCache(
        context: Context?,
        songPath: String,
        albumId: Long?,
        thumbnailDir: String,
        embeddedBytes: ByteArray? = null
    ): CoverPaths? {
        val bytes = embeddedBytes
            ?: extractEmbeddedPicture(songPath)
            ?: loadMediaStoreAlbumArt(context, albumId)
        if (bytes != null && bytes.isNotEmpty()) {
            return saveThumbnails(bytes, thumbnailDir)
        }

        val siblingCover = findSiblingCover(File(songPath)) ?: return null
        val siblingBytes = try {
            siblingCover.readBytes()
        } catch (_: Exception) {
            null
        }
        if (siblingBytes != null && siblingBytes.isNotEmpty()) {
            saveThumbnails(siblingBytes, thumbnailDir)?.let { return it }
        }
        return CoverPaths(siblingCover.absolutePath, siblingCover.absolutePath)
    }

    fun findSiblingCover(songFile: File): File? {
        val dir = songFile.parentFile ?: return null
        val files = dir.listFiles() ?: return null
        return files.firstOrNull { candidate ->
            candidate.isFile &&
                candidate.extension.lowercase() in COVER_EXTENSIONS &&
                candidate.nameWithoutExtension.lowercase().startsWith("cover")
        }
    }

    fun saveThumbnails(bytes: ByteArray, thumbnailDir: String): CoverPaths? {
        if (thumbnailDir.isBlank() || bytes.isEmpty()) return null
        val outDir = File(thumbnailDir)
        if (!outDir.exists() && !outDir.mkdirs()) return null

        val hash = sha256Hex(bytes)
        val highFile = File(outDir, "$hash.png")
        val lowFile = File(outDir, "$hash-low.png")

        if (!highFile.exists() || !lowFile.exists()) {
            writeScaledOrRaw(bytes, highFile, 400)
            writeScaledOrRaw(bytes, lowFile, 80)
        }

        val highCanonical = try {
            highFile.canonicalPath
        } catch (_: Exception) {
            highFile.absolutePath
        }
        val lowCanonical = try {
            lowFile.canonicalPath
        } catch (_: Exception) {
            lowFile.absolutePath
        }
        return CoverPaths(highCanonical, lowCanonical)
    }

    fun sha256Hex(bytes: ByteArray): String {
        val digest = MessageDigest.getInstance("SHA-256").digest(bytes)
        return digest.joinToString("") { "%02x".format(it) }
    }

    private fun writeScaledOrRaw(bytes: ByteArray, target: File, maxSize: Int) {
        val scaledWritten = try {
            val bounds = BitmapFactory.Options().apply { inJustDecodeBounds = true }
            BitmapFactory.decodeByteArray(bytes, 0, bytes.size, bounds)
            val sampleSize = calculateInSampleSize(bounds.outWidth, bounds.outHeight, maxSize)
            val decodeOptions = BitmapFactory.Options().apply { inSampleSize = sampleSize }
            val bitmap = BitmapFactory.decodeByteArray(bytes, 0, bytes.size, decodeOptions)
            if (bitmap != null) {
                val scaled = scaleBitmap(bitmap, maxSize)
                try {
                    FileOutputStream(target).use { out ->
                        scaled.compress(Bitmap.CompressFormat.PNG, 90, out)
                    }
                    true
                } finally {
                    if (scaled !== bitmap) {
                        scaled.recycle()
                    }
                    bitmap.recycle()
                }
            } else {
                false
            }
        } catch (_: Throwable) {
            false
        }

        if (!scaledWritten) {
            try {
                target.writeBytes(bytes)
            } catch (_: Exception) {
            }
        }
    }

    private fun calculateInSampleSize(width: Int, height: Int, reqSize: Int): Int {
        var inSampleSize = 1
        if (width > reqSize || height > reqSize) {
            val halfHeight = height / 2
            val halfWidth = width / 2
            while ((halfHeight / inSampleSize) >= reqSize && (halfWidth / inSampleSize) >= reqSize) {
                inSampleSize *= 2
            }
        }
        return inSampleSize
    }

    private fun scaleBitmap(source: Bitmap, maxSize: Int): Bitmap {
        val width = source.width
        val height = source.height
        if (width <= 0 || height <= 0 || (width <= maxSize && height <= maxSize)) {
            return source
        }
        val ratio = width.toFloat() / height.toFloat()
        val (targetW, targetH) = if (ratio >= 1f) {
            maxSize to (maxSize / ratio).toInt().coerceAtLeast(1)
        } else {
            (maxSize * ratio).toInt().coerceAtLeast(1) to maxSize
        }
        return Bitmap.createScaledBitmap(source, targetW, targetH, true)
    }

    private fun extractEmbeddedPicture(songPath: String): ByteArray? {
        return try {
            val retriever = MediaMetadataRetriever()
            try {
                retriever.setDataSource(songPath)
                retriever.embeddedPicture
            } finally {
                retriever.release()
            }
        } catch (_: Throwable) {
            null
        }
    }

    private fun loadMediaStoreAlbumArt(context: Context?, albumId: Long?): ByteArray? {
        if (context == null || albumId == null || albumId <= 0L) return null
        return try {
            val uri = ContentUris.withAppendedId(Uri.parse(ALBUM_ART_URI_STR), albumId)
            context.contentResolver.openInputStream(uri)?.use { it.readBytes() }
        } catch (_: Throwable) {
            null
        }
    }
}
