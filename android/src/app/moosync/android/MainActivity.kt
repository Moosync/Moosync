package app.moosync.android

import android.app.NativeActivity
import android.os.Bundle
import android.util.Log

class MainActivity : NativeActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Log.d("MoosyncAndroid", "MainActivity onCreate - Slint app starting via NativeActivity")
    }

    override fun onDestroy() {
        super.onDestroy()
        Log.d("MoosyncAndroid", "MainActivity onDestroy - terminating process")
        kotlin.system.exitProcess(0)
    }
}
