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

use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    time::Duration,
};

use jni::{
    AttachGuard, JavaVM,
    objects::{GlobalRef, JClass, JObject, JString, JValue},
};
use tracing::{debug, warn};
use types::errors::{MoosyncError, Result};

use crate::{
    MediaControlEvent, MediaPosition, MprisPlayerDetails,
    context::MprisContext,
};
use extensions_proto::moosync::types::PlayerState;

// ─────────────────────────────────────────────────────────────────────── //
//  AndroidMprisContext                                                     //
// ─────────────────────────────────────────────────────────────────────── //

/// Rust-side bridge to the Kotlin `MoosyncService`.
///
/// Wraps a `JavaVM` obtained from `slint::android::AndroidApp` and calls
/// `MoosyncService` static methods via JNI to update the notification and
/// MediaSession. Media button events come back through the `nativeOnXxx`
/// `#[no_mangle]` extern functions below.
pub struct AndroidMprisContext {
    jvm: Arc<JavaVM>,
    activity: GlobalRef,
    service_class: GlobalRef,
}

impl AndroidMprisContext {
    /// Create a new context.
    ///
    /// * `jvm`           — the JVM.
    /// * `activity`      — a `GlobalRef` to the Android `Activity` object.
    /// * `service_class` — a `GlobalRef` to the `MoosyncService` JClass.
    pub fn new(jvm: Arc<JavaVM>, activity: GlobalRef, service_class: GlobalRef) -> Self {
        Self {
            jvm,
            activity,
            service_class,
        }
    }

    /// Attach the current Rust thread to the JVM.
    fn jni_attach(&self) -> Result<AttachGuard<'_>> {
        self.jvm
            .attach_current_thread()
            .map_err(|e| MoosyncError::String(format!("JNI attach failed: {e:?}")))
    }

    /// Start the foreground `MoosyncService` from the stored Activity context.
    pub fn start_service(&self) -> Result<()> {
        let mut guard = self.jni_attach()?;
        let env = &mut *guard;

        let activity = self.activity.as_obj();

        let intent_class = env
            .find_class("android/content/Intent")
            .map_err(|e| MoosyncError::String(format!("{e:?}")))?;
        let service_class = unsafe { JClass::from_raw(self.service_class.as_obj().as_raw()) };

        let intent = env
            .new_object(
                &intent_class,
                "(Landroid/content/Context;Ljava/lang/Class;)V",
                &[
                    JValue::Object(activity),
                    JValue::Object(&service_class),
                ],
            )
            .map_err(|e| MoosyncError::String(format!("new Intent: {e:?}")))?;

        env.call_method(
            activity,
            "startForegroundService",
            "(Landroid/content/Intent;)Landroid/content/ComponentName;",
            &[JValue::Object(&intent)],
        )
        .map_err(|e| MoosyncError::String(format!("startForegroundService: {e:?}")))?;

        debug!("MoosyncService started");
        Ok(())
    }
}

impl MprisContext for AndroidMprisContext {
    /// Called by `MprisHolder::new_with_context`.
    ///
    /// Leaks a `Box<Sender<MediaControlEvent>>` to get a stable pointer that
    /// Kotlin/Java stores as a `Long` and passes back into the native callbacks.
    fn attach(&mut self, sender: Sender<MediaControlEvent>) -> Result<()> {
        // Leak the sender — it lives for the app lifetime.
        let ptr = Box::into_raw(Box::new(sender)) as i64;

        let mut guard = self.jni_attach()?;
        let env = &mut *guard;
        let service_class = unsafe { JClass::from_raw(self.service_class.as_obj().as_raw()) };

        env.call_static_method(
            &service_class,
            "registerNativeCallback",
            "(J)V",
            &[JValue::Long(ptr)],
        )
        .map_err(|e| MoosyncError::String(format!("registerNativeCallback: {e:?}")))?;

        // Start the service (creates the notification).
        drop(guard);
        self.start_service()
    }

    fn set_metadata(&mut self, metadata: MprisPlayerDetails) -> Result<()> {
        let mut guard = self.jni_attach()?;
        let env = &mut *guard;
        let service_class = unsafe { JClass::from_raw(self.service_class.as_obj().as_raw()) };

        let title = new_nullable_jstring(env, metadata.title.as_deref())?;
        let artist = new_nullable_jstring(env, metadata.artist_name.as_deref())?;
        let album = new_nullable_jstring(env, metadata.album_name.as_deref())?;
        let thumb = new_nullable_jstring(env, metadata.thumbnail.as_deref())?;
        let duration_ms = metadata.duration.map(|d| (d * 1000.0) as i64).unwrap_or(0);

        let null_obj = JObject::null();
        let title_obj = title.as_ref().map(|s| s.as_ref()).unwrap_or(&null_obj);
        let artist_obj = artist.as_ref().map(|s| s.as_ref()).unwrap_or(&null_obj);
        let album_obj = album.as_ref().map(|s| s.as_ref()).unwrap_or(&null_obj);
        let thumb_obj = thumb.as_ref().map(|s| s.as_ref()).unwrap_or(&null_obj);

        env.call_static_method(
            &service_class,
            "updateMetadata",
            "(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;JLjava/lang/String;)V",
            &[
                JValue::Object(title_obj),
                JValue::Object(artist_obj),
                JValue::Object(album_obj),
                JValue::Long(duration_ms),
                JValue::Object(thumb_obj),
            ],
        )
        .map_err(|e| MoosyncError::String(format!("updateMetadata JNI: {e:?}")))?;

        Ok(())
    }

    fn set_playback_state(&mut self, state: PlayerState, duration: u64) -> Result<()> {
        let is_playing = state == PlayerState::Playing;

        let mut guard = self.jni_attach()?;
        let env = &mut *guard;
        let service_class = unsafe { JClass::from_raw(self.service_class.as_obj().as_raw()) };

        env.call_static_method(
            &service_class,
            "updatePlayerState",
            "(ZJ)V",
            &[
                JValue::Bool(is_playing as u8),
                JValue::Long(duration as i64),
            ],
        )
        .map_err(|e| MoosyncError::String(format!("updatePlayerState JNI: {e:?}")))?;

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────── //
//  MprisHolder — owns the AndroidMprisContext.                            //
// ─────────────────────────────────────────────────────────────────────── //

pub struct MprisHolder {
    context: Mutex<Box<dyn MprisContext>>,
    pub event_rx: Arc<Mutex<Receiver<MediaControlEvent>>>,
    last_duration: Mutex<u64>,
    last_state: Mutex<PlayerState>,
}

impl MprisHolder {
    /// Construct with an `AndroidMprisContext` (the only option on Android).
    pub fn new_with_context(mut context: Box<dyn MprisContext>) -> Result<MprisHolder> {
        let (event_tx, event_rx) = mpsc::channel();
        context.attach(event_tx)?;

        Ok(MprisHolder {
            context: Mutex::new(context),
            event_rx: Arc::new(Mutex::new(event_rx)),
            last_duration: Mutex::new(0),
            last_state: Mutex::new(PlayerState::Stopped),
        })
    }

    #[tracing::instrument(level = "debug", skip(self, metadata))]
    pub fn set_metadata(&self, metadata: MprisPlayerDetails) -> Result<()> {
        let mut context = self.context.lock().unwrap();
        context.set_metadata(metadata)
    }

    #[tracing::instrument(level = "debug", skip(self, state))]
    pub fn set_playback_state(&self, state: PlayerState) -> Result<()> {
        let last_duration = self.last_duration.lock().unwrap();
        let duration = *last_duration;
        drop(last_duration);

        let mut context = self.context.lock().unwrap();
        context.set_playback_state(state, duration)?;

        let mut last_state = self.last_state.lock().unwrap();
        *last_state = state;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, duration))]
    pub fn set_position(&self, duration: f64) -> Result<()> {
        let mut last_duration = self.last_duration.lock().unwrap();
        *last_duration = (duration * 1000.0) as u64;
        drop(last_duration);

        #[allow(clippy::clone_on_copy)]
        let last_state = self.last_state.lock().unwrap().clone();
        self.set_playback_state(last_state)
    }
}

// ─────────────────────────────────────────────────────────────────────── //
//  Helpers                                                                 //
// ─────────────────────────────────────────────────────────────────────── //

fn new_nullable_jstring<'a>(
    env: &mut jni::JNIEnv<'a>,
    s: Option<&str>,
) -> Result<Option<JString<'a>>> {
    match s {
        Some(v) => env
            .new_string(v)
            .map(Some)
            .map_err(|e| MoosyncError::String(format!("new_string: {e:?}"))),
        None => Ok(None),
    }
}

// ─────────────────────────────────────────────────────────────────────── //
//  Native callback functions called FROM Kotlin via JNI.                  //
//                                                                         //
//  Kotlin MoosyncService stores the pointer returned by                   //
//  `registerNativeCallback` and passes it back on each media event.       //
//  The pointer is a `*const Sender<MediaControlEvent>` leaked in `attach`. //
// ─────────────────────────────────────────────────────────────────────── //

/// # Safety: `callback_ptr` must be the value stored by `registerNativeCallback` or 0.
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Java_app_moosync_android_services_MoosyncService_nativeOnPlay(
    _env: jni::JNIEnv,
    _obj: JObject,
    callback_ptr: i64,
) {
    unsafe {
        dispatch_event(callback_ptr, MediaControlEvent::Play);
    }
}

/// # Safety
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Java_app_moosync_android_services_MoosyncService_nativeOnPause(
    _env: jni::JNIEnv,
    _obj: JObject,
    callback_ptr: i64,
) {
    unsafe {
        dispatch_event(callback_ptr, MediaControlEvent::Pause);
    }
}

/// # Safety
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Java_app_moosync_android_services_MoosyncService_nativeOnStop(
    _env: jni::JNIEnv,
    _obj: JObject,
    callback_ptr: i64,
) {
    unsafe {
        dispatch_event(callback_ptr, MediaControlEvent::Stop);
    }
}

/// # Safety
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Java_app_moosync_android_services_MoosyncService_nativeOnSeekTo(
    _env: jni::JNIEnv,
    _obj: JObject,
    callback_ptr: i64,
    pos_ms: i64,
) {
    unsafe {
        dispatch_event(
            callback_ptr,
            MediaControlEvent::SetPosition(MediaPosition(Duration::from_millis(
                pos_ms.max(0) as u64,
            ))),
        );
    }
}

/// # Safety
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Java_app_moosync_android_services_MoosyncService_nativeOnSkipToNext(
    _env: jni::JNIEnv,
    _obj: JObject,
    callback_ptr: i64,
) {
    unsafe {
        dispatch_event(callback_ptr, MediaControlEvent::Next);
    }
}

/// # Safety
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Java_app_moosync_android_services_MoosyncService_nativeOnSkipToPrevious(
    _env: jni::JNIEnv,
    _obj: JObject,
    callback_ptr: i64,
) {
    unsafe {
        dispatch_event(callback_ptr, MediaControlEvent::Previous);
    }
}

/// # Safety: `ptr` must be a valid `*const Sender<MediaControlEvent>` or 0.
unsafe fn dispatch_event(ptr: i64, event: MediaControlEvent) {
    if ptr == 0 {
        warn!("dispatch_event: null callback pointer, dropping {:?}", event);
        return;
    }
    // SAFETY: created by Box::into_raw in `attach`, lives for app lifetime.
    let sender = unsafe { &*(ptr as *const Sender<MediaControlEvent>) };
    if let Err(e) = sender.send(event) {
        warn!("dispatch_event: channel closed: {e}");
    }
}
