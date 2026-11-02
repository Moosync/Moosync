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
use jni::{Env, EnvUnowned, NativeMethod, jni_str, objects::JClass, sys::jboolean};
use prost::Message;
use songs_proto::moosync::types::{Playlist, Song};
use types::android::AndroidJNIContext;

use crate::{
    OnPlaylistScanned, OnProgressUpdated, OnSongScanned, PlaylistSongId, ScanProgress,
    context::{ScanParameters, ScannerContext},
    error::ScannerError,
};

#[tracing::instrument(level = "debug", skip_all)]
fn call_scan_media(
    context: &AndroidJNIContext,
    request_bytes: &[u8],
) -> Result<Vec<u8>, ScannerError> {
    context
        .call_static(
            "app.moosync.android.scanner.MediaScanner",
            "scanMedia",
            (&context.activity, request_bytes),
        )
        .map_err(|e| ScannerError::Jni(format!("scanMedia: {e:?}")))
}

#[tracing::instrument(level = "debug", skip_all)]
pub extern "system" fn native_on_permission_result(
    _env: EnvUnowned,
    _class: JClass,
    request_code: i32,
    granted: jboolean,
) {
    types::android::dispatch_permission_result(request_code, granted);
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn register_scanner_natives(env: &mut Env, class: &JClass) -> Result<(), jni::errors::Error> {
    let methods = [unsafe {
        NativeMethod::from_raw_parts(
            jni_str!("nativeOnPermissionResult"),
            jni_str!("(IZ)V"),
            native_on_permission_result as *mut _,
        )
    }];
    unsafe { env.register_native_methods(class, &methods) }
}

#[tracing::instrument(level = "debug", skip_all)]
async fn check_and_request_audio_permission(
    context: &AndroidJNIContext,
) -> Result<bool, ScannerError> {
    let is_granted: bool = context
        .call_activity("hasAudioPermission", ())
        .map_err(|e| ScannerError::Jni(format!("hasAudioPermission: {e:?}")))?;

    if is_granted {
        tracing::debug!("Audio permission already granted");
        return Ok(true);
    }

    tracing::debug!("Audio permission not granted, requesting from user and waiting...");
    let (tx, rx) = tokio::sync::oneshot::channel();
    types::android::set_permission_waiter(tx);

    context
        .call_activity::<_, ()>("requestAudioPermission", ())
        .map_err(|e| ScannerError::Jni(format!("requestAudioPermission: {e:?}")))?;

    let granted = rx.await.unwrap_or(false);
    tracing::debug!("Audio permission response received: granted={}", granted);
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

        let response_bytes =
            tokio::task::spawn_blocking(move || call_scan_media(&jni_context, &request_bytes))
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
