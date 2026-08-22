package app.moosync.android

import android.app.NativeActivity
import android.os.Bundle
import android.util.Log

class MainActivity : NativeActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        Log.d("MoosyncAndroid", "MainActivity onCreate - Slint app starting via NativeActivity")
    }
}
