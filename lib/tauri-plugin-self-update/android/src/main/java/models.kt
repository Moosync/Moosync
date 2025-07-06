package app.moosync.selfupdate

import java.io.Serializable

data class PlatformInfo(val signature: String = "", val url: String = "") : Serializable
