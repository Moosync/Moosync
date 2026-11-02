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
    sync::{Mutex, OnceLock, mpsc::Sender},
    time::Duration,
};

use extensions_proto::moosync::types::PlayerState;
use jni::{
    Env, EnvUnowned, NativeMethod, jni_sig, jni_str,
    objects::{JClass, JValue},
};
use songs_proto::moosync::types::Song;
use tracing::{debug, error, warn};
use types::{android::AndroidJNIContext, prelude::SongsExt};

use crate::{MediaControlEvent, MediaPosition, context::MprisContext, error::MprisError};

static MPRIS_CALLBACK: OnceLock<Mutex<Option<Sender<MediaControlEvent>>>> = OnceLock::new();

#[tracing::instrument(level = "debug", skip_all)]
pub fn register_mpris_sender(sender: Sender<MediaControlEvent>) {
    if let Ok(mut lock) = MPRIS_CALLBACK.get_or_init(|| Mutex::new(None)).lock() {
        *lock = Some(sender);
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn dispatch_event(event: MediaControlEvent) {
    debug!(
        "dispatch_event: JNI received media control event: {:?}",
        event
    );

    if let Ok(guard) = MPRIS_CALLBACK.get_or_init(|| Mutex::new(None)).lock() {
        if let Some(ref sender) = *guard {
            if let Err(e) = sender.send(event) {
                error!("dispatch_event: channel closed: {e}");
            } else {
                debug!("dispatch_event: successfully sent event to Rust mpsc channel");
            }
        } else {
            warn!("dispatch_event: no registered sender, dropping {:?}", event);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────── //
//  AndroidMprisContext                                                     //
// ─────────────────────────────────────────────────────────────────────── //

#[derive(Debug)]
pub struct AndroidMprisContext {
    context: AndroidJNIContext,
}

impl From<jni::errors::Error> for MprisError {
    fn from(err: jni::errors::Error) -> Self {
        MprisError::InitFailed(format!("JNI error: {err:?}"))
    }
}

impl AndroidMprisContext {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(android_context: AndroidJNIContext) -> Self {
        Self {
            context: android_context,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn start_service(&self) -> Result<(), MprisError> {
        self.context
            .with_env(|env, ctx| start_foreground_service(env, ctx))
            .map_err(|e| MprisError::InitFailed(format!("start_service JNI: {e:?}")))?;
        Ok(())
    }
}

impl MprisContext for AndroidMprisContext {
    #[tracing::instrument(level = "debug", skip_all)]
    fn attach(&mut self, sender: Sender<MediaControlEvent>) -> Result<(), MprisError> {
        register_mpris_sender(sender);
        self.start_service()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_metadata(&mut self, song: &Song) -> Result<(), MprisError> {
        update_metadata(&self.context, song)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_playback_state(&mut self, state: PlayerState, duration: u64) -> Result<(), MprisError> {
        let is_playing = state == PlayerState::Playing;
        update_player_state(&self.context, is_playing, duration)
    }
}

// ─────────────────────────────────────────────────────────────────────── //
//  Helpers                                                                 //
// ─────────────────────────────────────────────────────────────────────── //

#[tracing::instrument(level = "debug", skip_all)]
fn start_foreground_service(
    env: &mut Env,
    context: &AndroidJNIContext,
) -> Result<(), jni::errors::Error> {
    let activity = context.activity.as_obj();
    let intent_class = env.find_class(jni_str!("android/content/Intent"))?;
    let service_class = context.service_class.as_obj();

    let intent = env.new_object(
        &intent_class,
        jni_sig!("(Landroid/content/Context;Ljava/lang/Class;)V"),
        &[JValue::Object(activity), JValue::Object(service_class)],
    )?;

    env.call_method(
        activity,
        jni_str!("startForegroundService"),
        jni_sig!("(Landroid/content/Intent;)Landroid/content/ComponentName;"),
        &[JValue::Object(&intent)],
    )?;
    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
fn update_metadata(context: &AndroidJNIContext, song: &Song) -> Result<(), MprisError> {
    let title_str = song.get_title();
    let artist_str = song.get_artist_string();
    let album_str = song.get_album_string();
    let thumb_str = song.get_cover_high().or_else(|| song.get_cover_low());
    let duration_ms = song.get_duration_or_default().as_millis() as i64;

    context
        .call_static::<_, ()>(
            "app.moosync.android.services.MoosyncService",
            "updateMetadata",
            (
                title_str.as_deref(),
                artist_str.as_deref(),
                album_str.as_deref(),
                duration_ms,
                thumb_str.as_deref(),
            ),
        )
        .map_err(|e| MprisError::SetMetadataFailed(format!("updateMetadata JNI: {e:?}")))?;

    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
fn update_player_state(
    context: &AndroidJNIContext,
    is_playing: bool,
    duration: u64,
) -> Result<(), MprisError> {
    context
        .call_static::<_, ()>(
            "app.moosync.android.services.MoosyncService",
            "updatePlayerState",
            (is_playing, duration as i64),
        )
        .map_err(|e| MprisError::SetPlaybackFailed(format!("updatePlayerState JNI: {e:?}")))?;

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────── //
//  Dynamic native callback implementations (No package names in Rust)     //
// ─────────────────────────────────────────────────────────────────────── //

#[tracing::instrument(level = "debug", skip_all)]
pub extern "system" fn native_on_play(_env: EnvUnowned, _class: JClass) {
    dispatch_event(MediaControlEvent::Play);
}

#[tracing::instrument(level = "debug", skip_all)]
pub extern "system" fn native_on_pause(_env: EnvUnowned, _class: JClass) {
    dispatch_event(MediaControlEvent::Pause);
}

#[tracing::instrument(level = "debug", skip_all)]
pub extern "system" fn native_on_stop(_env: EnvUnowned, _class: JClass) {
    dispatch_event(MediaControlEvent::Stop);
}

#[tracing::instrument(level = "debug", skip_all)]
pub extern "system" fn native_on_seek_to(_env: EnvUnowned, _class: JClass, pos_ms: i64) {
    dispatch_event(MediaControlEvent::SetPosition(MediaPosition(
        Duration::from_millis(pos_ms.max(0) as u64),
    )));
}

#[tracing::instrument(level = "debug", skip_all)]
pub extern "system" fn native_on_skip_to_next(_env: EnvUnowned, _class: JClass) {
    dispatch_event(MediaControlEvent::Next);
}

#[tracing::instrument(level = "debug", skip_all)]
pub extern "system" fn native_on_skip_to_previous(_env: EnvUnowned, _class: JClass) {
    dispatch_event(MediaControlEvent::Previous);
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn register_mpris_natives(env: &mut Env, class: &JClass) -> Result<(), jni::errors::Error> {
    let methods = [
        unsafe {
            NativeMethod::from_raw_parts(
                jni_str!("nativeOnPlay"),
                jni_str!("()V"),
                native_on_play as *mut _,
            )
        },
        unsafe {
            NativeMethod::from_raw_parts(
                jni_str!("nativeOnPause"),
                jni_str!("()V"),
                native_on_pause as *mut _,
            )
        },
        unsafe {
            NativeMethod::from_raw_parts(
                jni_str!("nativeOnStop"),
                jni_str!("()V"),
                native_on_stop as *mut _,
            )
        },
        unsafe {
            NativeMethod::from_raw_parts(
                jni_str!("nativeOnSeekTo"),
                jni_str!("(J)V"),
                native_on_seek_to as *mut _,
            )
        },
        unsafe {
            NativeMethod::from_raw_parts(
                jni_str!("nativeOnSkipToNext"),
                jni_str!("()V"),
                native_on_skip_to_next as *mut _,
            )
        },
        unsafe {
            NativeMethod::from_raw_parts(
                jni_str!("nativeOnSkipToPrevious"),
                jni_str!("()V"),
                native_on_skip_to_previous as *mut _,
            )
        },
    ];
    unsafe { env.register_native_methods(class, &methods) }
}
