package app.moosync.android

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.os.Environment
import android.provider.DocumentsContract
import android.provider.OpenableColumns
import android.util.Log
import java.io.File

class StoragePickerHandler(private val activity: Activity) {

    fun openDirectoryPicker() {
        launchPicker(
            Intent(Intent.ACTION_OPEN_DOCUMENT_TREE).apply {
                addFlags(
                    Intent.FLAG_GRANT_READ_URI_PERMISSION or
                    Intent.FLAG_GRANT_WRITE_URI_PERMISSION or
                    Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION
                )
            },
            REQUEST_CODE_PICK_DIRECTORY
        ) { NativeBridge.nativeOnDirectoryPicked("") }
    }

    fun openFilePicker(filter: String) {
        launchPicker(
            Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
                addCategory(Intent.CATEGORY_OPENABLE)
                type = "*/*"
                addFlags(
                    Intent.FLAG_GRANT_READ_URI_PERMISSION or
                    Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION
                )
            },
            REQUEST_CODE_PICK_FILE
        ) { NativeBridge.nativeOnFilePicked("") }
    }

    fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        when (requestCode) {
            REQUEST_CODE_PICK_DIRECTORY -> handleDirectoryResult(resultCode, data)
            REQUEST_CODE_PICK_FILE -> handleFileResult(resultCode, data)
        }
    }

    private fun launchPicker(intent: Intent, requestCode: Int, onError: () -> Unit) {
        activity.runOnUiThread {
            try {
                activity.startActivityForResult(intent, requestCode)
            } catch (e: Exception) {
                Log.e("MoosyncAndroid", "Failed to launch picker ($requestCode)", e)
                onError()
            }
        }
    }

    private fun persistUriPermission(uri: Uri, flags: Int) {
        try {
            activity.contentResolver.takePersistableUriPermission(uri, flags)
        } catch (e: Exception) {
            Log.w("MoosyncAndroid", "Failed to take persistable URI permission", e)
        }
    }

    private fun handleDirectoryResult(resultCode: Int, data: Intent?) {
        val uri = if (resultCode == Activity.RESULT_OK) data?.data else null
        val resolvedPath = uri?.let {
            persistUriPermission(
                it,
                Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION
            )
            resolveTreeUriToPath(it)
        } ?: ""
        NativeBridge.nativeOnDirectoryPicked(resolvedPath)
    }

    private fun handleFileResult(resultCode: Int, data: Intent?) {
        val uri = if (resultCode == Activity.RESULT_OK) data?.data else null
        val cachedPath = uri?.let {
            persistUriPermission(it, Intent.FLAG_GRANT_READ_URI_PERMISSION)
            copyUriToCache(it)
        } ?: ""
        NativeBridge.nativeOnFilePicked(cachedPath)
    }

    private fun resolveTreeUriToPath(treeUri: Uri): String {
        return try {
            val docId = DocumentsContract.getTreeDocumentId(treeUri)
            val parts = docId.split(":")
            val type = parts.getOrNull(0) ?: ""
            val subPath = if (parts.size > 1) parts[1] else ""

            if (type.equals("primary", ignoreCase = true)) {
                val root = Environment.getExternalStorageDirectory().absolutePath
                if (subPath.isNotEmpty()) "$root/$subPath" else root
            } else {
                activity.getExternalFilesDirs(null)
                    .filterNotNull()
                    .map { it.absolutePath }
                    .firstOrNull { it.contains(type) }
                    ?.substringBefore("/Android/data")
                    ?.let { root -> if (subPath.isNotEmpty()) "$root/$subPath" else root }
                    ?: treeUri.toString()
            }
        } catch (e: Exception) {
            Log.w("MoosyncAndroid", "Error resolving tree URI to path", e)
            treeUri.toString()
        }
    }

    private fun copyUriToCache(fileUri: Uri): String {
        return try {
            val displayName = queryFileName(fileUri) ?: "picked_${System.currentTimeMillis()}.msxt"
            val cacheFile = File(activity.cacheDir, displayName)
            activity.contentResolver.openInputStream(fileUri)?.use { input ->
                cacheFile.outputStream().use { input.copyTo(it) }
            }
            cacheFile.absolutePath
        } catch (e: Exception) {
            Log.e("MoosyncAndroid", "Failed to copy picked file to cache", e)
            ""
        }
    }

    private fun queryFileName(uri: Uri): String? {
        return try {
            activity.contentResolver.query(
                uri,
                arrayOf(OpenableColumns.DISPLAY_NAME),
                null,
                null,
                null
            )?.use { cursor ->
                if (cursor.moveToFirst()) {
                    val idx = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                    if (idx >= 0) cursor.getString(idx) else null
                } else null
            }
        } catch (_: Exception) {
            null
        }
    }

    companion object {
        const val REQUEST_CODE_PICK_DIRECTORY = 2001
        const val REQUEST_CODE_PICK_FILE = 2002
    }
}
