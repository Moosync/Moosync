package app.moosync.filescanner

import android.Manifest
import android.app.Activity
import android.util.Log
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.Permission
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Channel
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSArray
import com.google.gson.Gson
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import org.json.JSONArray
import org.json.JSONObject

@InvokeArg
class ScanArgs {
    lateinit var channel: Channel
}

@TauriPlugin(
    permissions = [
        Permission(strings = [Manifest.permission.READ_MEDIA_AUDIO, Manifest.permission.READ_EXTERNAL_STORAGE, Manifest.permission.READ_MEDIA_IMAGES]),
    ]
)
class FileScannerPlugin(private val activity: Activity): Plugin(activity) {
    @Command
    fun android_scan_music(invoke: Invoke) {
        val args = invoke.parseArgs(ScanArgs::class.java)
        CoroutineScope(Dispatchers.IO).launch {
            Log.d("file-scanner", "scanning audio files")
            val songs = AudioScanner().readDirectory(activity)
            val ret = Gson().toJson(songs)
            val obj = JSObject()
            obj.put("songs", ret);
            Log.d("file-scanner", "android_scan_music: resolving $ret")
            args.channel.send(obj)
        }

        val obj = JSObject()
        invoke.resolve(obj)
    }
}
