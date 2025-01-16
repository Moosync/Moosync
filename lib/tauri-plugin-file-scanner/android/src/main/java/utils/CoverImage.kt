// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

// Moosync
// Copyright (C) 2025 Moosync
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

import android.content.ContentUris
import android.content.Context
import android.database.Cursor
import android.graphics.Bitmap.CompressFormat
import android.media.MediaMetadataRetriever
import android.net.Uri
import android.os.Build
import android.os.Environment
import android.provider.DocumentsContract
import android.provider.MediaStore
import android.util.Size
import androidx.core.net.toUri
import java.io.ByteArrayInputStream
import java.io.ByteArrayOutputStream
import java.io.File
import java.io.FileInputStream
import java.io.FileOutputStream
import java.io.InputStream

public fun getUriFromID(context: Context, id: Long): String? {
    return writeCoverImage(context, id)
}

private fun writeCoverImage(context: Context, id: Long): String? {
    val fileName = File(context.filesDir.canonicalFile, "cover-${id.toString()}-high.jpg")
    if (fileName.exists()) {
        return fileName.toString()
    }

    val uri = ContentUris.withAppendedId(MediaStore.Audio.Media.EXTERNAL_CONTENT_URI, id)

    val cover = extractCoverImage(context, uri)
    if (cover != null) {
        val file = FileOutputStream(fileName)

        val buffer = ByteArray(4 * 1024)
        while (true) {
            val bytesRead = cover.read(buffer)
            if (bytesRead < 0) break
            file.write(buffer, 0, bytesRead)
        }

        return fileName.toString()
    }
    return null
}

val FALLBACKS = arrayOf("cover.jpg", "album.jpg", "folder.jpg")

private fun fallbackCoverImage(uri: Uri): InputStream? {
    // Method 2: look for album art in external files

    val parent: File = uri.path?.let { File(it).parentFile } as File
    for (fallback in FALLBACKS) {
        val cover = File(parent, fallback)
        if (cover.exists()) {
            return FileInputStream(cover)
        }
    }
    return null
}

private fun extractCoverImage(context: Context, uri: Uri): InputStream? {
    try {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
            val thumb = context.contentResolver.loadThumbnail(uri, Size(512, 512), null)
            val stream = ByteArrayOutputStream()
            thumb.compress(CompressFormat.JPEG, 100, stream)
            return ByteArrayInputStream(stream.toByteArray())
        }
    } catch (e: Exception) {
        // Nothing to do
    }

    val retriever = MediaMetadataRetriever()
    try {
        retriever.setDataSource(context, uri)
        val picture = retriever.embeddedPicture
        return picture?.let { ByteArrayInputStream(it) } ?: fallbackCoverImage(uri)
    } finally {
        retriever.release()
    }
}

fun getPathFromURI(context: Context, uri: Uri): String? {
    if (DocumentsContract.isDocumentUri(context, uri)) {
        if (isExternalStorageDocument(uri)) {
            val docId = DocumentsContract.getDocumentId(uri)
            val split = docId.split(":")
            return if ("primary".equals(split[0], ignoreCase = true)) {
                "${Environment.getExternalStorageDirectory()}/${split[1]}"
            } else {
                null
            }
        } else if (isDownloadsDocument(uri)) {
            val id = DocumentsContract.getDocumentId(uri)
            val contentUri =
                    ContentUris.withAppendedId(
                            Uri.parse("content://downloads/public_downloads"),
                            java.lang.Long.valueOf(id)
                    )
            return getDataColumn(context, contentUri, null, null)
        } else if (isMediaDocument(uri)) {
            val docId = DocumentsContract.getDocumentId(uri)
            val split = docId.split(":")
            val contentUri: Uri? =
                    when (split[0]) {
                        "image" -> MediaStore.Images.Media.EXTERNAL_CONTENT_URI
                        "video" -> MediaStore.Video.Media.EXTERNAL_CONTENT_URI
                        "audio" -> MediaStore.Audio.Media.EXTERNAL_CONTENT_URI
                        else -> null
                    }
            val selection = "_id=?"
            val selectionArgs = arrayOf(split[1])
            return getDataColumn(context, contentUri, selection, selectionArgs)
        }
    } else if ("content".equals(uri.scheme, ignoreCase = true)) {
        return getDataColumn(context, uri, null, null)
    } else if ("file".equals(uri.scheme, ignoreCase = true)) {
        return uri.path
    }
    return null
}

private fun getDataColumn(
        context: Context,
        uri: Uri?,
        selection: String?,
        selectionArgs: Array<String>?
): String? {
    var cursor: Cursor? = null
    val column = "_data"
    val projection = arrayOf(column)
    try {
        cursor = context.contentResolver.query(uri!!, projection, selection, selectionArgs, null)
        if (cursor != null && cursor.moveToFirst()) {
            val columnIndex = cursor.getColumnIndexOrThrow(column)
            return cursor.getString(columnIndex)
        }
    } finally {
        cursor?.close()
    }
    return null
}

private fun isExternalStorageDocument(uri: Uri): Boolean {
    return "com.android.externalstorage.documents" == uri.authority
}

private fun isDownloadsDocument(uri: Uri): Boolean {
    return "com.android.providers.downloads.documents" == uri.authority
}

private fun isMediaDocument(uri: Uri): Boolean {
    return "com.android.providers.media.documents" == uri.authority
}
