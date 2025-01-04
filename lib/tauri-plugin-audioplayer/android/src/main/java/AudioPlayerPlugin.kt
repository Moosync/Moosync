package app.moosync.audioplayer

import android.app.Activity
import android.util.Log
import app.moosync.audioplayer.services.interfaces.MediaPlayerCallbacks
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke

internal class LoadArgs {
    lateinit var key: String
    lateinit var src: String
    var autoplay: Boolean = false
}

internal class KeyArgs {
    lateinit var key: String
}

internal class SeekArgs {
    lateinit var key: String
    var seek = 0f
}

@TauriPlugin
class AudioPlayerPlugin(private val activity: Activity): Plugin(activity) {
    private val implementation = AudioPlayerRemote(activity)

    init {
        implementation.addMediaCallbacks(
            callbacks = object : MediaPlayerCallbacks {
                override fun onPlay(key: String) {
                    super.onPlay(key)
                    val ret = JSObject()
                    ret.put("key", key);
                    trigger("onPlay", ret)
                }

                override fun onPause(key: String) {
                    super.onPause(key)
                    val ret = JSObject()
                    ret.put("key", key);
                    trigger("onPause", ret)
                }

                override fun onSongEnded(key: String) {
                    super.onSongEnded(key)
                    val ret = JSObject()
                    ret.put("key", key);
                    trigger("onSongEnded", ret)
                }

                override fun onStop(key: String) {
                    super.onStop(key)
                    val ret = JSObject()
                    ret.put("key", key);
                    trigger("onStop", ret)
                }

                override fun onTimeChange(key: String, time: Int) {
                    super.onTimeChange(key, time)
                    Log.d("TAG", "onTimeChange: emitting timechange to tauri")
                    val ret = JSObject()
                    ret.put("key", key);
                    ret.put("pos", time)
                    trigger("onTimeChange", ret)
                }
            }
        )
    }

    @Command
    fun load(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(LoadArgs::class.java)
            Log.d("TAG", "load: loading ${args.src} ${args.key}")
            implementation.controls?.load(args.key, args.src, args.autoplay)
        } catch (e: Exception) {
            Log.d("TAG", "load: failed to load audio $e")
        }
        val ret = JSObject()
        invoke.resolve(ret)
    }

    @Command
    fun play(invoke: Invoke) {
        val args = invoke.parseArgs(KeyArgs::class.java)
        implementation.controls?.play(args.key)
        val ret = JSObject()
        invoke.resolve(ret)
    }

    @Command
    fun pause(invoke: Invoke) {
        val args = invoke.parseArgs(KeyArgs::class.java)
        implementation.controls?.pause(args.key)
        val ret = JSObject()
        invoke.resolve(ret)
    }

    @Command
    fun stop(invoke: Invoke) {
        val args = invoke.parseArgs(KeyArgs::class.java)
        implementation.controls?.stop(args.key)
        val ret = JSObject()
        invoke.resolve(ret)
    }

    @Command
    fun seek(invoke: Invoke) {
        val args = invoke.parseArgs(SeekArgs::class.java)
        implementation.controls?.seek(args.key, args.seek.toInt())
        val ret = JSObject()
        invoke.resolve(ret)
    }
}
