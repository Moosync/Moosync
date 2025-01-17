// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

package app.moosync.audioplayer

import android.app.Activity
import android.content.ComponentName
import android.content.Context
import android.content.ContextWrapper
import android.content.Intent
import android.content.ServiceConnection
import android.os.IBinder
import android.support.v4.media.session.MediaSessionCompat.Callback
import android.util.Log
import app.moosync.audioplayer.models.Song
import app.moosync.audioplayer.services.Constants.ACTION_FROM_MAIN_ACTIVITY
import app.moosync.audioplayer.services.MediaPlayerService
import app.moosync.audioplayer.services.NotificationHandler
import app.moosync.audioplayer.services.interfaces.MediaControls
import app.moosync.audioplayer.services.interfaces.MediaPlayerCallbacks
import app.moosync.audioplayer.services.interfaces.MediaServiceWrapper
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.consume
import kotlinx.coroutines.launch
import java.lang.reflect.InvocationHandler
import java.lang.reflect.Method
import java.lang.reflect.Proxy
import java.lang.reflect.UndeclaredThrowableException

data class MethodRequirements(val scope: CoroutineScope?, val channel: Channel<Any?>?, val method: (mediaService: MediaServiceWrapper) -> Any?) {
    constructor(method: (mediaService: MediaServiceWrapper) -> Any?) : this(null, null, method)
}

class AudioPlayerRemote private constructor(activity: Activity) {

    private var mediaService: MediaServiceWrapper? = null
    private val mContextWrapper: ContextWrapper = ContextWrapper(activity)
    private val serviceConnection: ServiceConnection

    // methodQueue containing methods which are to be executed after connection to service is created
    private val methodQueue: MutableList<MethodRequirements> = mutableListOf()

    // Return an instance of proxies media controls
    private var _controls: MediaControls? = null
    val controls: MediaControls?
        get() {
            if (_controls != null) {
                return _controls
            }

            val controls = mediaService?.controls
            if (controls != null) {
                _controls = Proxy.newProxyInstance(
                    controls.javaClass.classLoader,
                    controls.javaClass.interfaces,
                    TransportControlInvocationHandler(::runOrAddToQueue)
                ) as MediaControls
                return _controls
            }

            return null
        }

    init {
        serviceConnection = object : ServiceConnection {
            override fun onServiceConnected(p0: ComponentName?, p1: IBinder?) {
                val binder = p1 as MediaPlayerService.MediaPlayerBinder?
                mediaService = binder?.service
                mediaService?.let { runFromMethodQueue(it) }
            }

            override fun onServiceDisconnected(p0: ComponentName?) {
                mediaService = null
            }
        }

        bindService()
    }

    private fun runFromMethodQueue(mediaService: MediaServiceWrapper) {
        for (method in methodQueue) {
            val retValue = method.method.invoke(mediaService)
            val channel = method.channel
            val scope = method.scope

            if (channel != null && scope != null) {
                scope.launch {
                    channel.send(retValue)
                }
            }
        }
    }

    private fun runOrAddToQueue(method: (mediaService: MediaServiceWrapper) -> Unit) {
        if (mediaService == null) {
            methodQueue.add(MethodRequirements {
                method.invoke(it)
            })
            return
        }

        method.invoke(mediaService!!)
    }

    /**
     * This returns a deferred with method result
     * The result is returned after mediaService is created
     */
    @Suppress("UNCHECKED_CAST")
    private inline fun <reified T> runOrAddToQueueAsync(
        scope: CoroutineScope = CoroutineScope(
            Dispatchers.Default
        ), crossinline method: (mediaService: MediaServiceWrapper) -> T
    ): Deferred<T> = scope.async {
        if (mediaService == null) {

            val channel: Channel<T?> = Channel()

            methodQueue.add(MethodRequirements(scope, channel as Channel<Any?>) {
                return@MethodRequirements method.invoke(it)
            })

            channel.consume {
                val value = this.receive()
                channel.close()
                return@async value as T
            }

        } else {
            return@async method.invoke(mediaService!!)
        }
    }

    // Binds to the media service
    private fun bindService() {
        if (mediaService == null) {
            val intent = Intent(mContextWrapper, MediaPlayerService::class.java)
            intent.putExtra(ACTION_FROM_MAIN_ACTIVITY, true)

            // Start service as foreground
            mContextWrapper.startForegroundService(intent)

            mContextWrapper.bindService(
                Intent().setClass(
                    mContextWrapper,
                    MediaPlayerService::class.java
                ), serviceConnection, Context.BIND_AUTO_CREATE
            )
        }
    }

    fun addMediaCallbacks(callbacks: MediaPlayerCallbacks) {
        return runOrAddToQueue {
            it.addMediaPlayerCallbacks(callbacks)
        }
    }

    fun addMediaSessionCallbacks(callback: Callback) {
        return runOrAddToQueue {
            it.addMediaSessionCallbacks(callback)
        }
    }

    fun release() {
        mediaService?.decideQuit()
        if (mediaService != null) {
            mediaService?.setMainActivityStatus(false)
            mContextWrapper.unbindService(serviceConnection)
            mediaService = null
        }
    }

    // Maintain only single connection to service
    companion object {
        private var isInitialized = false
        operator fun invoke(activity: Activity): AudioPlayerRemote {
            if (!isInitialized) {
                return AudioPlayerRemote(activity)
            }
            throw Error("Remote is already initialized")
        }
    }

    // Proxy media controls to queue all method calls
    private class TransportControlInvocationHandler(private val addToQueueHandler: (method: (mediaService: MediaServiceWrapper) -> Unit) -> Unit) :
        InvocationHandler {
        override fun invoke(proxy: Any?, method: Method?, args: Array<out Any>?) {
            try {
                addToQueueHandler {
                    if (args != null) {
                        method?.invoke(it.controls, *args)
                    } else {
                        method?.invoke(it.controls)
                    }
                }
            } catch (e: UndeclaredThrowableException) {
                Log.e("InvocationHandler", "invoke: Failed to run method ${method?.name} $e", )
            }
        }
    }
}
