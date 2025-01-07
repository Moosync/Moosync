# Add project specific ProGuard rules here.
# You can control the set of applied configuration files using the
# proguardFiles setting in build.gradle.
#
# For more details, see
#   http://developer.android.com/guide/developing/tools/proguard.html

# If your project uses WebView with JS, uncomment the following
# and specify the fully qualified class name to the JavaScript interface
# class:
#-keepclassmembers class fqcn.of.javascript.interface.for.webview {
#   public *;
#}

# Uncomment this to preserve the line number information for
# debugging stack traces.
#-keepattributes SourceFile,LineNumberTable

# If you keep the line number information, uncomment this to
# hide the original source file name.
#-renamesourcefileattribute SourceFile

# Keep the MediaPlayerService class and its methods, fields, and inner classes
-keep class app.moosync.audioplayer.services.interfaces.** {
    *;
}

# Keep the Binder inner class (MediaPlayerBinder) and its methods/fields
-keep class app.moosync.audioplayer.services.MediaPlayerService$MediaPlayerBinder {
    *;
}

# Keep the parent class (MediaBrowserServiceCompat) as it is required for compatibility
-keep public class androidx.media.MediaBrowserServiceCompat {
    *;
}

# Keep the custom model classes used in the service
-keep class app.moosync.audioplayer.models.** {
    *;
}