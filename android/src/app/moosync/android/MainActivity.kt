package app.moosync.android

import android.app.NativeActivity
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Bundle
import android.os.Environment
import android.provider.DocumentsContract
import android.provider.OpenableColumns
import android.util.Log
import java.io.File

class MainActivity : NativeActivity() {
    private var pendingDeepLink: String? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Log.d("MoosyncAndroid", "MainActivity onCreate - Slint app starting via NativeActivity")
        handleIntent(intent)
    }

    override fun onNewIntent(intent: Intent?) {
        super.onNewIntent(intent)
        setIntent(intent)
        handleIntent(intent)
    }

    private fun handleIntent(intent: Intent?) {
        if (intent == null) return
        val action = intent.action
        val data = intent.dataString
        Log.d("MoosyncAndroid", "handleIntent: action=$action, data=$data")
        if (Intent.ACTION_VIEW == action && data != null) {
            try {
                nativeOnDeepLink(data)
            } catch (e: UnsatisfiedLinkError) {
                Log.w("MoosyncAndroid", "nativeOnDeepLink not linked yet, caching pending deep link", e)
                pendingDeepLink = data
            }
        }
    }

    fun processPendingDeepLink() {
        val link = pendingDeepLink ?: return
        pendingDeepLink = null
        try {
            nativeOnDeepLink(link)
        } catch (e: UnsatisfiedLinkError) {
            Log.e("MoosyncAndroid", "Failed to invoke nativeOnDeepLink for pending link", e)
        }
    }

    fun openDirectoryPicker() {
        runOnUiThread {
            try {
                val intent = Intent(Intent.ACTION_OPEN_DOCUMENT_TREE).apply {
                    addFlags(
                        Intent.FLAG_GRANT_READ_URI_PERMISSION or
                        Intent.FLAG_GRANT_WRITE_URI_PERMISSION or
                        Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION
                    )
                }
                startActivityForResult(intent, REQUEST_CODE_PICK_DIRECTORY)
            } catch (e: Exception) {
                Log.e("MoosyncAndroid", "Failed to launch directory picker", e)
                try {
                    nativeOnDirectoryPicked("")
                } catch (ue: UnsatisfiedLinkError) {
                    Log.e("MoosyncAndroid", "nativeOnDirectoryPicked not linked", ue)
                }
            }
        }
    }

    fun openFilePicker(filter: String) {
        runOnUiThread {
            try {
                val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
                    addCategory(Intent.CATEGORY_OPENABLE)
                    type = "*/*"
                    addFlags(
                        Intent.FLAG_GRANT_READ_URI_PERMISSION or
                        Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION
                    )
                }
                startActivityForResult(intent, REQUEST_CODE_PICK_FILE)
            } catch (e: Exception) {
                Log.e("MoosyncAndroid", "Failed to launch file picker", e)
                try {
                    nativeOnFilePicked("")
                } catch (ue: UnsatisfiedLinkError) {
                    Log.e("MoosyncAndroid", "nativeOnFilePicked not linked", ue)
                }
            }
        }
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        when (requestCode) {
            REQUEST_CODE_PICK_DIRECTORY -> {
                val uri = if (resultCode == RESULT_OK) data?.data else null
                var resolvedPath = ""
                if (uri != null) {
                    try {
                        contentResolver.takePersistableUriPermission(
                            uri,
                            Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION
                        )
                    } catch (e: Exception) {
                        Log.w("MoosyncAndroid", "Failed to take persistable URI permission for directory", e)
                    }
                    resolvedPath = resolveTreeUriToPath(uri)
                }
                Log.d("MoosyncAndroid", "onActivityResult: directory picked=$resolvedPath")
                try {
                    nativeOnDirectoryPicked(resolvedPath)
                } catch (e: UnsatisfiedLinkError) {
                    Log.e("MoosyncAndroid", "Failed to invoke nativeOnDirectoryPicked", e)
                }
            }
            REQUEST_CODE_PICK_FILE -> {
                val uri = if (resultCode == RESULT_OK) data?.data else null
                var cachedPath = ""
                if (uri != null) {
                    try {
                        contentResolver.takePersistableUriPermission(
                            uri,
                            Intent.FLAG_GRANT_READ_URI_PERMISSION
                        )
                    } catch (e: Exception) {
                        Log.w("MoosyncAndroid", "Failed to take persistable URI permission for file", e)
                    }
                    cachedPath = copyUriToCache(uri)
                }
                Log.d("MoosyncAndroid", "onActivityResult: file picked=$cachedPath")
                try {
                    nativeOnFilePicked(cachedPath)
                } catch (e: UnsatisfiedLinkError) {
                    Log.e("MoosyncAndroid", "Failed to invoke nativeOnFilePicked", e)
                }
            }
        }
    }

    private fun resolveTreeUriToPath(treeUri: Uri): String {
        try {
            val docId = DocumentsContract.getTreeDocumentId(treeUri)
            val parts = docId.split(":")
            if (parts.isNotEmpty()) {
                val type = parts[0]
                val subPath = if (parts.size > 1) parts[1] else ""
                if (type.equals("primary", ignoreCase = true)) {
                    val root = Environment.getExternalStorageDirectory().absolutePath
                    return if (subPath.isNotEmpty()) "$root/$subPath" else root
                }
                val extDirs = getExternalFilesDirs(null)
                for (dir in extDirs) {
                    if (dir != null) {
                        val path = dir.absolutePath
                        val idx = path.indexOf("/Android/data")
                        if (idx > 0) {
                            val storageRoot = path.substring(0, idx)
                            if (storageRoot.contains(type)) {
                                return if (subPath.isNotEmpty()) "$storageRoot/$subPath" else storageRoot
                            }
                        }
                    }
                }
            }
        } catch (e: Exception) {
            Log.w("MoosyncAndroid", "Error resolving tree URI to path, falling back to uri string", e)
        }
        return treeUri.toString()
    }

    private fun copyUriToCache(fileUri: Uri): String {
        return try {
            val displayName = queryFileName(fileUri) ?: "picked_${System.currentTimeMillis()}.msxt"
            val cacheFile = File(cacheDir, displayName)
            contentResolver.openInputStream(fileUri)?.use { input ->
                cacheFile.outputStream().use { output ->
                    input.copyTo(output)
                }
            }
            cacheFile.absolutePath
        } catch (e: Exception) {
            Log.e("MoosyncAndroid", "Failed to copy picked file to cache", e)
            ""
        }
    }

    private fun queryFileName(uri: Uri): String? {
        return try {
            contentResolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)?.use { cursor ->
                if (cursor.moveToFirst()) {
                    val idx = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                    if (idx >= 0) cursor.getString(idx) else null
                } else null
            }
        } catch (_: Exception) {
            null
        }
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        val granted = grantResults.isNotEmpty() && grantResults[0] == PackageManager.PERMISSION_GRANTED
        Log.d("MoosyncAndroid", "onRequestPermissionsResult: req=$requestCode, granted=$granted")
        try {
            nativeOnPermissionResult(requestCode, granted)
        } catch (e: UnsatisfiedLinkError) {
            Log.e("MoosyncAndroid", "Failed to invoke nativeOnPermissionResult", e)
        }
    }

    override fun onBackPressed() {
        val handled = try {
            nativeOnBackPressed()
        } catch (e: UnsatisfiedLinkError) {
            Log.e("MoosyncAndroid", "Failed to invoke nativeOnBackPressed", e)
            false
        }
        Log.d("MoosyncAndroid", "MainActivity onBackPressed: handled=$handled")
        if (!handled) {
            super.onBackPressed()
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        Log.d("MoosyncAndroid", "MainActivity onDestroy")
    }

    private external fun nativeOnPermissionResult(requestCode: Int, granted: Boolean)
    private external fun nativeOnBackPressed(): Boolean
    private external fun nativeOnDeepLink(url: String)
    private external fun nativeOnDirectoryPicked(path: String)
    private external fun nativeOnFilePicked(path: String)

    companion object {
        const val REQUEST_CODE_PICK_DIRECTORY = 2001
        const val REQUEST_CODE_PICK_FILE = 2002

        init {
            try {
                System.loadLibrary("slint_app")
                Log.d("MoosyncAndroid", "Successfully loaded slint_app in MainActivity")
            } catch (e: UnsatisfiedLinkError) {
                Log.e("MoosyncAndroid", "Failed to load slint_app in MainActivity", e)
            }
        }
    }
}
