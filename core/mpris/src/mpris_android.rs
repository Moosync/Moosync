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

use extensions_proto::moosync::types::PlayerState;
use jni::{
    AttachGuard, JavaVM,
    objects::{GlobalRef, JClass, JObject, JString, JValue},
};
use tracing::{debug, info, warn};
use types::android::AndroidJNIContext;

use crate::{
    MediaControlEvent, MediaPosition, MprisPlayerDetails, context::MprisContext, error::MprisError,
};

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
    context: AndroidJNIContext,
}

impl AndroidMprisContext {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(android_context: AndroidJNIContext) -> Self {
        Self {
            context: android_context,
        }
    }

    /// Attach the current Rust thread to the JVM.
    #[tracing::instrument(level = "debug", skip_all)]
    fn jni_attach(&self) -> Result<AttachGuard<'_>, MprisError> {
        self.context
            .jvm
            .attach_current_thread()
            .map_err(|e| MprisError::InitFailed(format!("JNI attach failed: {e:?}")))
    }

    /// Start the foreground `MoosyncService` from the stored Activity context.
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn start_service(&self) -> Result<(), MprisError> {
        let mut guard = self.jni_attach()?;
        let env = &mut *guard;

        let activity = self.context.activity.as_obj();

        let intent_class = env
            .find_class("android/content/Intent")
            .map_err(|e| MprisError::InitFailed(format!("find_class: {e:?}")))?;
        let service_class =
            unsafe { JClass::from_raw(self.context.service_class.as_obj().as_raw()) };

        let intent = env
            .new_object(
                &intent_class,
                "(Landroid/content/Context;Ljava/lang/Class;)V",
                &[JValue::Object(activity), JValue::Object(&service_class)],
            )
            .map_err(|e| MprisError::InitFailed(format!("new Intent: {e:?}")))?;

        env.call_method(
            activity,
            "startForegroundService",
            "(Landroid/content/Intent;)Landroid/content/ComponentName;",
            &[JValue::Object(&intent)],
        )
        .map_err(|e| MprisError::InitFailed(format!("startForegroundService: {e:?}")))?;

        debug!("MoosyncService started");
        Ok(())
    }
}

impl MprisContext for AndroidMprisContext {
    /// Called by `MprisHolder::new_with_context`.
    ///
    /// Leaks a `Box<Sender<MediaControlEvent>>` to get a stable pointer that
    /// Kotlin/Java stores as a `Long` and passes back into the native
    /// callbacks.
    #[tracing::instrument(level = "debug", skip_all)]
    fn attach(&mut self, sender: Sender<MediaControlEvent>) -> Result<(), MprisError> {
        // Leak the sender — it lives for the app lifetime.
        let ptr = Box::into_raw(Box::new(sender)) as i64;

        let mut guard = self.jni_attach()?;
        let env = &mut *guard;
        let service_class =
            unsafe { JClass::from_raw(self.context.service_class.as_obj().as_raw()) };

        env.call_static_method(
            &service_class,
            "registerNativeCallback",
            "(J)V",
            &[JValue::Long(ptr)],
        )
        .map_err(|e| MprisError::AttachFailed(format!("registerNativeCallback: {e:?}")))?;

        // Start the service (creates the notification).
        drop(guard);
        self.start_service()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_metadata(&mut self, metadata: MprisPlayerDetails) -> Result<(), MprisError> {
        let mut guard = self.jni_attach()?;
        let env = &mut *guard;
        let service_class =
            unsafe { JClass::from_raw(self.context.service_class.as_obj().as_raw()) };

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
        .map_err(|e| MprisError::SetMetadataFailed(format!("updateMetadata JNI: {e:?}")))?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_playback_state(&mut self, state: PlayerState, duration: u64) -> Result<(), MprisError> {
        let is_playing = state == PlayerState::Playing;

        let mut guard = self.jni_attach()?;
        let env = &mut *guard;
        let service_class =
            unsafe { JClass::from_raw(self.context.service_class.as_obj().as_raw()) };

        env.call_static_method(
            &service_class,
            "updatePlayerState",
            "(ZJ)V",
            &[
                JValue::Bool(is_playing as u8),
                JValue::Long(duration as i64),
            ],
        )
        .map_err(|e| MprisError::SetPlaybackFailed(format!("updatePlayerState JNI: {e:?}")))?;

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────── //
//  Helpers                                                                 //
// ─────────────────────────────────────────────────────────────────────── //

#[tracing::instrument(level = "debug", skip_all)]
fn new_nullable_jstring<'a>(
    env: &mut jni::JNIEnv<'a>,
    s: Option<&str>,
) -> Result<Option<JString<'a>>, MprisError> {
    match s {
        Some(v) => env
            .new_string(v)
            .map(Some)
            .map_err(|e| MprisError::SetMetadataFailed(format!("new_string: {e:?}"))),
        None => Ok(None),
    }
}

// ─────────────────────────────────────────────────────────────────────── //
//  Native callback functions called FROM Kotlin via JNI.                  //
//                                                                         //
//  Kotlin MoosyncService stores the pointer returned by                   //
//  `registerNativeCallback` and passes it back on each media event.       //
unsafe extern "C" {
    fn __android_log_print(
        prio: std::os::raw::c_int,
        tag: *const std::os::raw::c_char,
        fmt: *const std::os::raw::c_char,
        ...
    ) -> std::os::raw::c_int;
}

#[tracing::instrument(level = "debug", skip_all)]
fn log_to_android(msg: &str) {
    use std::ffi::CString;
    if let Ok(tag) = CString::new("MoosyncAndroidRust") {
        if let Ok(fmt) = CString::new("%s") {
            if let Ok(message) = CString::new(msg) {
                unsafe {
                    __android_log_print(4, tag.as_ptr(), fmt.as_ptr(), message.as_ptr());
                }
            }
        }
    }
}

/// # Safety: `callback_ptr` must be the value stored by `registerNativeCallback` or 0.
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Java_app_moosync_android_services_MoosyncService_nativeOnPlay(
    mut env: jni::JNIEnv,
    _obj: JObject,
    callback_ptr: i64,
) {
    unsafe {
        dispatch_event(&mut env, callback_ptr, MediaControlEvent::Play);
    }
}

/// # Safety
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Java_app_moosync_android_services_MoosyncService_nativeOnPause(
    mut env: jni::JNIEnv,
    _obj: JObject,
    callback_ptr: i64,
) {
    unsafe {
        dispatch_event(&mut env, callback_ptr, MediaControlEvent::Pause);
    }
}

/// # Safety
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Java_app_moosync_android_services_MoosyncService_nativeOnStop(
    mut env: jni::JNIEnv,
    _obj: JObject,
    callback_ptr: i64,
) {
    unsafe {
        dispatch_event(&mut env, callback_ptr, MediaControlEvent::Stop);
    }
}

/// # Safety
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Java_app_moosync_android_services_MoosyncService_nativeOnSeekTo(
    mut env: jni::JNIEnv,
    _obj: JObject,
    callback_ptr: i64,
    pos_ms: i64,
) {
    unsafe {
        dispatch_event(
            &mut env,
            callback_ptr,
            MediaControlEvent::SetPosition(MediaPosition(Duration::from_millis(
                pos_ms.max(0) as u64
            ))),
        );
    }
}

/// # Safety
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Java_app_moosync_android_services_MoosyncService_nativeOnSkipToNext(
    mut env: jni::JNIEnv,
    _obj: JObject,
    callback_ptr: i64,
) {
    unsafe {
        dispatch_event(&mut env, callback_ptr, MediaControlEvent::Next);
    }
}

/// # Safety
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub unsafe extern "C" fn Java_app_moosync_android_services_MoosyncService_nativeOnSkipToPrevious(
    mut env: jni::JNIEnv,
    _obj: JObject,
    callback_ptr: i64,
) {
    unsafe {
        dispatch_event(&mut env, callback_ptr, MediaControlEvent::Previous);
    }
}

/// # Safety: `ptr` must be a valid `*const Sender<MediaControlEvent>` or 0.
unsafe fn dispatch_event(_env: &mut jni::JNIEnv, ptr: i64, event: MediaControlEvent) {
    let log_msg = format!(
        "dispatch_event: JNI received media control event: {:?}",
        event
    );
    log_to_android(&log_msg);
    info!("{}", log_msg);

    if ptr == 0 {
        let warn_msg = format!(
            "dispatch_event: null callback pointer, dropping {:?}",
            event
        );
        log_to_android(&warn_msg);
        warn!("{}", warn_msg);
        return;
    }
    // SAFETY: created by Box::into_raw in `attach`, lives for app lifetime.
    let sender = unsafe { &*(ptr as *const Sender<MediaControlEvent>) };
    if let Err(e) = sender.send(event) {
        let err_msg = format!("dispatch_event: channel closed: {e}");
        log_to_android(&err_msg);
        warn!("{}", err_msg);
    } else {
        log_to_android("dispatch_event: successfully sent event to Rust mpsc channel");
        info!("dispatch_event: successfully sent event to Rust mpsc channel");
    }
}
