package app.moosync.filescanner

import android.Manifest
import android.app.Activity
import android.util.Log
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.Permission
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import com.google.gson.Gson
import org.json.JSONArray
import org.json.JSONObject

@InvokeArg
class PingArgs {
  var value: String? = null
}

@TauriPlugin(
    permissions = [
        Permission(strings = [Manifest.permission.READ_MEDIA_AUDIO, Manifest.permission.READ_EXTERNAL_STORAGE, Manifest.permission.READ_MEDIA_IMAGES]),
    ]
)
class FileScannerPlugin(private val activity: Activity): Plugin(activity) {
    @Command
    fun android_scan_music(invoke: Invoke) {
        Log.d("file-scanner", "scanning audio files")
        val songs = AudioScanner().readDirectory(activity)
        val ret = Gson().toJson(songs)

        Log.d("file-scanner", "android_scan_music: resolving $ret")
        val obj = JSObject()
        obj.put("songs", ret);
        invoke.resolve(obj)
    }
}
