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

use std::path::PathBuf;

use android_scanner_proto::moosync::types::{ScanRequest, ScanResult, ScannedPlaylist};
use async_trait::async_trait;
use jni::{
    Env,
    errors::Error as JniError,
    jni_sig, jni_str,
    objects::{JByteArray, JClass, JValue},
};
use prost::Message;
use songs_proto::moosync::types::{Playlist, Song};
use types::android::AndroidJNIContext;

use crate::{
    OnPlaylistScanned, OnProgressUpdated, OnSongScanned, PlaylistSongId, ScanProgress,
    context::{ScanParameters, ScannerContext},
    error::ScannerError,
};

impl From<JniError> for ScannerError {
    fn from(err: JniError) -> Self { ScannerError::Jni(format!("JNI error: {err:?}")) }
}

#[tracing::instrument(level = "debug", skip_all)]
fn load_scanner_class<'local>(
    env: &mut Env<'local>,
    context: &AndroidJNIContext,
) -> Result<JClass<'local>, ScannerError> {
    let activity = context.activity.as_obj();
    let activity_class = env
        .call_method(
            activity,
            jni_str!("getClass"),
            jni_sig!("()Ljava/lang/Class;"),
            &[],
        )
        .map_err(|e| ScannerError::Jni(format!("getClass: {e:?}")))?
        .l()
        .map_err(|e| ScannerError::Jni(format!("getClass object: {e:?}")))?;

    let class_loader = env
        .call_method(
            &activity_class,
            jni_str!("getClassLoader"),
            jni_sig!("()Ljava/lang/ClassLoader;"),
            &[],
        )
        .map_err(|e| ScannerError::Jni(format!("getClassLoader: {e:?}")))?
        .l()
        .map_err(|e| ScannerError::Jni(format!("getClassLoader object: {e:?}")))?;

    let class_name = env
        .new_string("app.moosync.android.scanner.MediaScanner")
        .map_err(|e| ScannerError::Jni(format!("new_string class_name: {e:?}")))?;

    let loaded_class = env
        .call_method(
            &class_loader,
            jni_str!("loadClass"),
            jni_sig!("(Ljava/lang/String;)Ljava/lang/Class;"),
            &[JValue::Object(&class_name)],
        )
        .map_err(|e| ScannerError::Jni(format!("loadClass MediaScanner: {e:?}")))?
        .l()
        .map_err(|e| ScannerError::Jni(format!("loadClass object: {e:?}")))?;

    Ok(unsafe { JClass::from_raw(env, loaded_class.as_raw().cast()) })
}

#[tracing::instrument(level = "debug", skip_all)]
fn call_scan_media(
    env: &mut Env,
    context: &AndroidJNIContext,
    request_bytes: &[u8],
) -> Result<Vec<u8>, ScannerError> {
    let scanner_class = load_scanner_class(env, context)?;
    let activity = context.activity.as_obj();
    let request_jbytes = env
        .byte_array_from_slice(request_bytes)
        .map_err(|e| ScannerError::Jni(format!("byte_array_from_slice: {e:?}")))?;

    let result_obj = env
        .call_static_method(
            &scanner_class,
            jni_str!("scanMedia"),
            jni_sig!("(Landroid/content/Context;[B)[B"),
            &[JValue::Object(activity), JValue::Object(&request_jbytes)],
        )
        .map_err(|e| ScannerError::Jni(format!("call_static_method scanMedia: {e:?}")))?
        .l()
        .map_err(|e| ScannerError::Jni(format!("scanMedia return object: {e:?}")))?;

    let result_array = unsafe { JByteArray::from_raw(env, result_obj.as_raw().cast()) };
    env.convert_byte_array(&result_array)
        .map_err(|e| ScannerError::Jni(format!("convert_byte_array: {e:?}")))
}

static PERMISSION_TX: std::sync::Mutex<Option<tokio::sync::oneshot::Sender<bool>>> =
    std::sync::Mutex::new(None);

const REQUEST_AUDIO_PERMISSION: i32 = 1001;

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
#[tracing::instrument(level = "debug", skip_all)]
pub unsafe extern "C" fn Java_app_moosync_android_MainActivity_nativeOnPermissionResult(
    _env: jni::EnvUnowned,
    _class: jni::objects::JClass,
    request_code: i32,
    granted: bool,
) {
    tracing::info!(
        "nativeOnPermissionResult: code={}, granted={}",
        request_code,
        granted
    );
    if request_code == REQUEST_AUDIO_PERMISSION {
        if let Some(tx) = PERMISSION_TX.lock().unwrap().take() {
            let _ = tx.send(granted);
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn get_audio_permission_string(env: &mut Env) -> Result<String, ScannerError> {
    let build_version_class = env
        .find_class(jni_str!("android/os/Build$VERSION"))
        .map_err(|e| ScannerError::Jni(format!("find Build$VERSION: {e:?}")))?;
    let sdk_int = env
        .get_static_field(&build_version_class, jni_str!("SDK_INT"), jni_sig!("I"))
        .map_err(|e| ScannerError::Jni(format!("get SDK_INT: {e:?}")))?
        .i()
        .map_err(|e| ScannerError::Jni(format!("get SDK_INT int: {e:?}")))?;

    if sdk_int >= 33 {
        Ok("android.permission.READ_MEDIA_AUDIO".to_string())
    } else {
        Ok("android.permission.READ_EXTERNAL_STORAGE".to_string())
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn is_audio_permission_granted(
    env: &mut Env,
    context: &AndroidJNIContext,
    permission: &str,
) -> Result<bool, ScannerError> {
    let activity = context.activity.as_obj();
    let perm_jstr = env
        .new_string(permission)
        .map_err(|e| ScannerError::Jni(format!("new_string permission: {e:?}")))?;

    let status = env
        .call_method(
            activity,
            jni_str!("checkSelfPermission"),
            jni_sig!("(Ljava/lang/String;)I"),
            &[JValue::Object(&perm_jstr)],
        )
        .map_err(|e| ScannerError::Jni(format!("checkSelfPermission: {e:?}")))?
        .i()
        .map_err(|e| ScannerError::Jni(format!("checkSelfPermission int: {e:?}")))?;

    Ok(status == 0)
}

#[tracing::instrument(level = "debug", skip_all)]
fn request_audio_permission(
    env: &mut Env,
    context: &AndroidJNIContext,
    permission: &str,
) -> Result<(), ScannerError> {
    let activity = context.activity.as_obj();
    let perm_jstr = env
        .new_string(permission)
        .map_err(|e| ScannerError::Jni(format!("new_string permission: {e:?}")))?;

    let string_class = env
        .find_class(jni_str!("java/lang/String"))
        .map_err(|e| ScannerError::Jni(format!("find java/lang/String: {e:?}")))?;

    let array = env
        .new_object_array(1, &string_class, &perm_jstr)
        .map_err(|e| ScannerError::Jni(format!("new_object_array: {e:?}")))?;

    env.call_method(
        activity,
        jni_str!("requestPermissions"),
        jni_sig!("([Ljava/lang/String;I)V"),
        &[
            JValue::Object(&array),
            JValue::Int(REQUEST_AUDIO_PERMISSION),
        ],
    )
    .map_err(|e| ScannerError::Jni(format!("requestPermissions: {e:?}")))?;

    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
async fn check_and_request_audio_permission(
    context: &AndroidJNIContext,
) -> Result<bool, ScannerError> {
    let (is_granted, permission_str) = context.jvm.attach_current_thread(|env| {
        let perm = get_audio_permission_string(env)?;
        let granted = is_audio_permission_granted(env, context, &perm)?;
        Ok::<_, ScannerError>((granted, perm))
    })?;

    if is_granted {
        tracing::debug!("Audio permission already granted");
        return Ok(true);
    }

    tracing::info!("Audio permission not granted, requesting from user and waiting...");
    let (tx, rx) = tokio::sync::oneshot::channel();
    {
        let mut guard = PERMISSION_TX.lock().unwrap();
        *guard = Some(tx);
    }

    context
        .jvm
        .attach_current_thread(|env| request_audio_permission(env, context, &permission_str))?;

    let granted = rx.await.unwrap_or(false);
    tracing::info!("Audio permission response received: granted={}", granted);
    Ok(granted)
}

#[tracing::instrument(level = "debug", skip_all)]
fn encode_request(request: &ScanRequest) -> Vec<u8> {
    let mut buf = Vec::with_capacity(request.encoded_len());
    if let Err(err) = request.encode(&mut buf) {
        tracing::error!("Failed to encode ScanRequest: {:?}", err);
    }
    buf
}

#[tracing::instrument(level = "debug", skip_all)]
fn map_playlist_song_id(entry: String) -> PlaylistSongId {
    if entry.starts_with("http://") || entry.starts_with("https://") {
        PlaylistSongId::Url(entry)
    } else {
        PlaylistSongId::Path(PathBuf::from(entry))
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn convert_scanned_playlists(
    playlists: Vec<ScannedPlaylist>,
) -> Vec<(Playlist, Vec<PlaylistSongId>)> {
    let mut converted = Vec::with_capacity(playlists.len());
    for item in playlists {
        if let Some(playlist) = item.playlist {
            let song_ids = item
                .song_paths
                .into_iter()
                .map(map_playlist_song_id)
                .collect();
            converted.push((playlist, song_ids));
        }
    }
    converted
}

#[tracing::instrument(level = "debug", skip_all)]
fn update_progress(total: usize, current: usize, on_progress: &OnProgressUpdated) {
    if let Some(div) = (current * 100).checked_div(total) {
        on_progress(ScanProgress::PROGRESS(div as u8));
    }
}

#[tracing::instrument(level = "debug", skip_all)]
async fn dispatch_scanned_songs(
    songs: Vec<Song>,
    on_song: &OnSongScanned,
    on_progress: &OnProgressUpdated,
) {
    let total = songs.len();
    for (idx, song) in songs.into_iter().enumerate() {
        on_song(None, vec![song]).await;
        update_progress(total, idx + 1, on_progress);
    }
}

pub struct AndroidScannerContext {
    context: AndroidJNIContext,
}

impl AndroidScannerContext {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(context: AndroidJNIContext) -> Self { Self { context } }
}

#[async_trait]
impl ScannerContext for AndroidScannerContext {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn scan(
        &self,
        params: ScanParameters,
        on_song: &OnSongScanned,
        on_playlist: &OnPlaylistScanned,
        on_progress: &OnProgressUpdated,
    ) -> Result<(), ScannerError> {
        if !check_and_request_audio_permission(&self.context).await? {
            tracing::warn!("Audio permission denied by user, skipping scan");
            on_progress(ScanProgress::STOPPED);
            return Err(ScannerError::PermissionDenied);
        }

        let request = ScanRequest {
            scan_dirs: params
                .scan_dirs
                .into_iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
            exclude_dirs: params
                .exclude_dirs
                .into_iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
            thumbnail_dir: params.thumbnail_dir.to_string_lossy().to_string(),
            artist_split: params.artist_split,
        };
        let request_bytes = encode_request(&request);
        let jni_context = self.context.clone();

        let response_bytes = tokio::task::spawn_blocking(move || {
            jni_context
                .jvm
                .attach_current_thread(|env| call_scan_media(env, &jni_context, &request_bytes))
        })
        .await
        .map_err(ScannerError::Join)??;

        let scan_result = ScanResult::decode(response_bytes.as_slice())?;

        dispatch_scanned_songs(scan_result.songs, on_song, on_progress).await;

        let playlists = convert_scanned_playlists(scan_result.playlists);
        if !playlists.is_empty() {
            on_playlist(playlists).await;
        }

        on_progress(ScanProgress::STOPPED);
        Ok(())
    }
}
