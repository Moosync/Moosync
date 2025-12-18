package app.moosync.moosync

import android.os.Bundle
import android.content.Intent
import android.webkit.WebView
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.contract.ActivityResultContracts

class MainActivity : TauriActivity() {
    private lateinit var unknownSourcesLauncher: ActivityResultLauncher<Intent>
    private lateinit var apkInstallLauncher: ActivityResultLauncher<Intent>

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Register for unknown sources permission result
        unknownSourcesLauncher = registerForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
            app.moosync.selfupdate.SelfUpdatePlugin.handleUnknownSourcesResult(result.resultCode)
        }

        // Register for APK install result (optional, for tracking install completion)
        apkInstallLauncher = registerForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
            app.moosync.selfupdate.SelfUpdatePlugin.handleApkInstallResult(result.resultCode)
        }

        // Pass launchers to plugin
        app.moosync.selfupdate.SelfUpdatePlugin.unknownSourcesLauncher = unknownSourcesLauncher
        app.moosync.selfupdate.SelfUpdatePlugin.apkInstallLauncher = apkInstallLauncher
    }
}
