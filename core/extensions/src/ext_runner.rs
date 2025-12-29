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

#[cfg(any(target_os = "android", target_os = "ios"))]
use std::path::Path;
use std::{
    collections::{BTreeMap, HashMap},
    env, fs,
    io::{Read, Write},
    path::PathBuf,
    process,
    str::FromStr,
    sync::Arc,
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use crypto::sha1::Sha1;
use crypto::{
    digest::Digest,
    sha2::{Sha256, Sha512},
};
use extism::{Error, Manifest, PTR, Plugin, PluginBuilder, UserData, ValType::I64, Wasm, host_fn};
use extism_convert::Json;
use futures::executor::block_on;
use interprocess::local_socket::{
    GenericFilePath, GenericNamespaced, NameType, ToFsName, ToNsName, prelude::LocalSocketStream,
    traits::Stream,
};
use regex::{Captures, Regex};
use serde_json::Value;
use tokio::sync::{
    Mutex,
    mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
};
use tracing::{debug, error, info};
use types::{
    errors::Result as MoosyncResult, extensions::ExtensionManifest, preferences::PreferenceUIData,
};
use types::{
    extensions::{sanitize_album, sanitize_artist, sanitize_playlist, sanitize_song},
    ui::extensions::ExtensionDetail,
};

use crate::models::{
    ExtensionCommand, ExtensionCommandResponse, ExtensionExtraEventResponse,
    GenericExtensionHostRequest, MainCommand, MainCommandResponse, RunnerCommand,
    RunnerCommandResp,
};

// Ext handler inner
pub type MainCommandReplySender = UnboundedSender<ExtensionCommandResponse>;
pub type ExtCommandSender = UnboundedSender<GenericExtensionHostRequest<MainCommand>>;
pub type ExtCommandReplySender = UnboundedSender<GenericExtensionHostRequest<MainCommandResponse>>;

// Outer handler

pub type ExtCommandReceiver = UnboundedReceiver<GenericExtensionHostRequest<MainCommand>>;

struct MainCommandUserData {
    reply_map: Arc<std::sync::Mutex<HashMap<String, ExtCommandReplySender>>>,
    ext_command_tx: ExtCommandSender,
    package_name: String,
}

host_fn!(send_main_command(user_data: MainCommandUserData; command: Json<MainCommand>) -> Option<Value> {
    let user_data = user_data.get()?;
    let user_data = user_data.lock().unwrap();

    let mut command = command.0;
    tracing::debug!("Got extension command {:?}", command);
    match command.to_request(gen_channel_id(), user_data.package_name.clone()) {
        Ok(request) => {
            let reply_map = user_data.reply_map.clone();
            let (tx, mut rx) = unbounded_channel();
            {
                let mut reply_map = reply_map.lock().unwrap();
                reply_map.insert(request.channel.clone(), tx);
            }

            let ext_command_tx = user_data.ext_command_tx.clone();
            tracing::trace!("Sending request {:?}", request);
            ext_command_tx.send(request.clone()).unwrap();

            if let MainCommand::UpdateAccounts(_) = command {
                return Ok(Some(Json(MainCommandResponse::UpdateAccounts(true))));
            }

            tracing::trace!("waiting on response for {:?}", command);
            if let Some(resp) = block_on(rx.recv()) {
                {
                    let mut reply_map = reply_map.lock().unwrap();
                    reply_map.remove(&request.channel);
                }
                tracing::debug!("Got response for {:?}: {:?}", command, resp);
                return Ok(resp.data.map(Json))
            } else {
                return Err(Error::msg("Failed to receive response"))
            }
        }
        Err(e) => {
            tracing::error!("Failed to map command {:?}", command);
            return Err(Error::msg(e.to_string()))
        }
    }
});

host_fn!(system_time() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
   Ok(since_the_epoch.as_secs())
});

struct SocketUserData {
    socks: Vec<LocalSocketStream>,
    allowed_paths: Option<BTreeMap<String, PathBuf>>,
}

host_fn!(open_clientfd(user_data: SocketUserData; sock_path: String) -> i64 {
    let user_data = user_data.get()?;
    let mut user_data = user_data.lock().unwrap();

    if user_data.socks.len() > u8::MAX as usize {
        error!("Cannot open more sockets");
        return Ok(-1);
    }


    // Check if path is allowed
    if user_data.allowed_paths.is_none() {
        error!("Not enough permissions to access {}", sock_path);
        return Ok(-1)
    }

    let sock_path_parsed = PathBuf::from_str(sock_path.as_str())?;
    if let Some(allowed_paths) = user_data.allowed_paths.as_ref() {
        for (key, value) in allowed_paths {
            if let Some(sock_path) = sock_path_parsed.to_str() {
                if let Some(allowed_path) = value.to_str() {
                    debug!("Checking {:?}, {:?}", sock_path, key);
                    if sock_path.starts_with(allowed_path) {
                        // Resultant path is the mapped_path + (passed path - prefix)
                        let mapped_path = PathBuf::from_str(format!("{}/{}", key, sock_path.replacen(allowed_path, "", 1)).as_str())?;
                        if !mapped_path.exists() {
                            debug!("Path {:?} does not exist", mapped_path);
                            continue;
                        }

                        let mapped_path_name = if GenericNamespaced::is_supported() && key.starts_with("\\\\.\\pipe\\") {
                            mapped_path.file_name().unwrap()
                                .to_ns_name::<GenericNamespaced>()
                        } else {
                            mapped_path.to_fs_name::<GenericFilePath>()
                        }?;

                        if let Ok(sock) = LocalSocketStream::connect(mapped_path_name) {
                            user_data.socks.push(sock);
                            return Ok((user_data.socks.len() - 1) as i64);
                        }
                    }
                } else {
                   error!("Failed to convert mapped path: {:?} to string", value);
                }
            } else {
                error!("Failed to convert passed path to string");
                return Ok(-1);
            }
        }
    }

    error!("Sock path not specified in allowed_paths");
    Ok(-1)

});

host_fn!(write_sock(user_data: SocketUserData; sock_id: i64, buf: Vec<u8>) -> i64 {
    info!("Here");
    let user_data = user_data.get()?;
    let mut user_data = user_data.lock().unwrap();

    let sock = user_data.socks.get_mut(sock_id as usize);
    if let Some(sock) = sock {
        info!("Writing {:?}", buf);
        let res = sock.write_all(&buf);
        if let Err(e) = res {
            error!("Failed to write data to sock {}", e);
            return Ok(-1);
        } else {
            info!("Wrote all");
            return Ok(-1);
        }
    }

    error!("Invalid sock id");
    return Ok(-1);
});

host_fn!(read_sock(user_data: SocketUserData; sock_id: i64, read_len: u64) -> Vec<u8> {
    let user_data = user_data.get()?;
    let mut user_data = user_data.lock().unwrap();

    let sock = user_data.socks.get_mut(sock_id as usize);
    if let Some(sock) = sock {
        let mut read_len = read_len;
        if read_len == 0 || read_len > 1024 {
            read_len = 1024
        }

        info!("Reading {}", read_len);
        let mut ret = vec![0; read_len as usize];
        let read = sock.read(&mut ret);
        if let Ok(read) = read {
            if read >= 1024 {
                error!("Read out of bounds");
                return Ok(vec![]);
            }
            let mut ret = ret.to_vec();
            ret.truncate(read);
            return Ok(ret);
        }
    }

    error!("Invalid sock id");
    return Ok(vec![]);
});

host_fn!(hash(hash_type: String, data: Vec<u8>) -> Vec<u8> {
    info!("Calling hash function {} type {:?}", hash_type, data);
    let mut hasher: Box<dyn Digest> = match hash_type.as_str() {
        "SHA256" => {
            Box::new(Sha256::new())
        },
        "SHA512" => {
            Box::new(Sha512::new())
        },
        _ => {
            Box::new(Sha1::new())
        },
    };

    hasher.input(&data);
    let mut buf = vec![0u8; hasher.output_bytes()];
    hasher.result(&mut buf);
    return Ok(buf);
});

#[derive(Debug, Clone)]
struct Extension {
    plugin: Option<Arc<Mutex<Plugin>>>,
    package_name: String,
    name: String,
    icon: String,
    author: Option<String>,
    version: String,
    path: PathBuf,
    preferences: HashMap<String, PreferenceUIData>,
    active: bool,
}

impl From<&Extension> for ExtensionDetail {
    #[tracing::instrument(level = "debug", skip(val))]
    fn from(val: &Extension) -> Self {
        ExtensionDetail {
            name: val.name.clone(),
            package_name: val.package_name.clone(),
            desc: None,
            author: val.author.clone(),
            version: val.version.clone(),
            has_started: true,
            entry: val.path.clone().to_str().unwrap().to_string(),
            preferences: val.preferences.clone().into_values().collect(),
            extension_path: val.path.clone().to_str().unwrap().to_string(),
            extension_icon: Some(val.icon.clone()),
            active: val.active,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ExtensionHandlerInner {
    extensions_path: String,
    ext_command_tx: ExtCommandSender,
    extensions_map: Arc<Mutex<HashMap<String, Extension>>>,
    reply_map: Arc<std::sync::Mutex<HashMap<String, ExtCommandReplySender>>>,
    // #[cfg(any(target_os = "android", target_os = "ios"))]
    cache_path: PathBuf,
}

impl ExtensionHandlerInner {
    #[tracing::instrument(level = "debug", skip(ext_command_tx))]
    pub fn new(
        extensions_path: &PathBuf,
        cache_path: &PathBuf,
        ext_command_tx: ExtCommandSender,
    ) -> Self {
        Self {
            extensions_path: extensions_path.to_string_lossy().to_string(),
            ext_command_tx,
            extensions_map: Default::default(),
            reply_map: Arc::new(std::sync::Mutex::new(HashMap::new())),
            // #[cfg(any(target_os = "android", target_os = "ios"))]
            cache_path: cache_path.clone(),
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn find_extension_manifests(&self) -> Vec<PathBuf> {
        let mut package_json_paths = Vec::new();

        if let Ok(entries) = fs::read_dir(self.extensions_path.clone()) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Check only the first level subdirectories
                    if let Ok(sub_entries) = fs::read_dir(&path) {
                        for sub_entry in sub_entries.flatten() {
                            let sub_path = sub_entry.path();
                            if sub_path.is_file()
                                && sub_path.file_name() == Some("package.json".as_ref())
                            {
                                package_json_paths.push(sub_path);
                            }
                        }
                    }
                } else if path.is_file() && path.file_name() == Some("package.json".as_ref()) {
                    package_json_paths.push(path);
                }
            }
        }
        package_json_paths
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn find_extensions(&self) -> Vec<ExtensionManifest> {
        let manifests = self.find_extension_manifests();
        let mut parsed_manifests = vec![];

        let extensions_map = self.extensions_map.lock().await;
        for manifest_path in manifests {
            if let Ok(contents) = fs::read(manifest_path.clone()) {
                match serde_json::from_slice::<ExtensionManifest>(&contents) {
                    Ok(mut manifest) => {
                        manifest.extension_entry = manifest_path
                            .parent()
                            .unwrap()
                            .join(manifest.extension_entry);
                        if !extensions_map.contains_key(&manifest.name)
                            && manifest.extension_entry.extension().unwrap() == "wasm"
                            && manifest.extension_entry.exists()
                        {
                            manifest.icon = manifest_path
                                .parent()
                                .unwrap()
                                .join(manifest.icon)
                                .to_string_lossy()
                                .to_string();
                            parsed_manifests.push(manifest);
                        }
                    }
                    Err(e) => tracing::error!("Error parsing manifest: {:?}", e),
                }
            }
        }

        parsed_manifests
    }

    fn get_empty_extension(manifest: ExtensionManifest) -> Extension {
        Extension {
            plugin: None,
            name: manifest.display_name,
            package_name: manifest.name,
            icon: manifest.icon,
            author: manifest.author,
            version: manifest.version,
            path: manifest.extension_entry.clone(),
            preferences: Default::default(),
            active: false,
        }
    }

    #[tracing::instrument(level = "debug", skip())]
    fn spawn_extension(
        manifest: ExtensionManifest,
        reply_map: Arc<std::sync::Mutex<HashMap<String, ExtCommandReplySender>>>,
        ext_command_tx: ExtCommandSender,
        cache_path: PathBuf,
    ) -> Arc<Mutex<Plugin>> {
        let url = Wasm::file(manifest.extension_entry.clone());
        let mut plugin_manifest = Manifest::new([url]);
        if let Some(permissions) = manifest.permissions {
            let re = Regex::new(r"\{([A-Z_][A-Z0-9_]*)\}").unwrap();
            let mut allowed_paths = HashMap::new();
            for (key, value) in permissions.paths {
                // Replace all matches with corresponding env variable values
                let parsed = re
                    .replace_all(key.as_str(), |caps: &Captures| {
                        let var_name = &caps[1];
                        if var_name == "CACHE_DIR" {
                            let ext_cache_dir =
                                cache_path.join("extensions").join(manifest.name.clone());
                            if let Err(e) = fs::create_dir_all(&ext_cache_dir) {
                                tracing::error!(
                                    "Failed to create cache dir for extension {}: {:?}",
                                    manifest.name,
                                    e
                                );
                            }
                            return ext_cache_dir.to_string_lossy().to_string();
                        }
                        env::var(var_name).unwrap_or_else(|_| "".to_string())
                    })
                    .to_string();

                let Ok(parsed_path) = PathBuf::from_str(parsed.as_str());
                if !parsed_path.exists() {
                    tracing::warn!("Path {:?} does not exist", parsed_path);
                    continue;
                }
                allowed_paths.insert(parsed, value);
            }

            info!("Got allowed paths {:?}", allowed_paths);
            plugin_manifest = plugin_manifest
                .with_allowed_hosts(permissions.hosts.into_iter())
                .with_allowed_paths(allowed_paths.into_iter())
                .with_config_key("pid", format!("{}", process::id()));
        }

        let user_data = UserData::new(MainCommandUserData {
            reply_map,
            ext_command_tx,
            package_name: manifest.name,
        });

        let sock_data = UserData::new(SocketUserData {
            socks: vec![],
            allowed_paths: plugin_manifest.allowed_paths.clone(),
        });

        #[allow(unused_mut)]
        let mut plugin_builder = PluginBuilder::new(plugin_manifest)
            .with_wasi(true)
            .with_function(
                "send_main_command",
                [PTR],
                [PTR],
                user_data,
                send_main_command,
            )
            .with_function("system_time", [], [PTR], UserData::default(), system_time)
            .with_function(
                "open_clientfd",
                [PTR],
                [I64],
                sock_data.clone(),
                open_clientfd,
            )
            .with_function(
                "write_sock",
                [I64, PTR],
                [I64],
                sock_data.clone(),
                write_sock,
            )
            .with_function("read_sock", [I64, I64], [PTR], sock_data, read_sock)
            .with_function("hash", [PTR, PTR], [PTR], UserData::default(), hash);

        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            let cache_path = cache_path.join("wasmtime").join("config.toml");
            if !cache_path.exists() {
                fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
            }
            fs::write(
                &cache_path,
                format!(
                    r#"
            [cache]
            enabled = true
            directory = "{}"
            cleanup-interval = "30m"
            files-total-size-soft-limit = "1Gi"
            "#,
                    cache_path.parent().unwrap().join("cache").to_string_lossy()
                ),
            )
            .unwrap();

            plugin_builder = plugin_builder.with_cache_config(cache_path);
        }

        let plugin = plugin_builder.build().unwrap();

        Arc::new(Mutex::new(plugin))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn spawn_extensions(&mut self) {
        let manifests = self.find_extensions().await;
        for manifest in manifests {
            let cache_path = self.cache_path.clone();
            let package_name = manifest.name.clone();
            let extension_map = self.extensions_map.clone();
            let reply_map = self.reply_map.clone();
            let ext_command_tx = self.ext_command_tx.clone();

            {
                let mut extensions_map = extension_map.lock().await;
                extensions_map.insert(
                    package_name.clone(),
                    Self::get_empty_extension(manifest.clone()),
                );
                ext_command_tx
                    .send(
                        MainCommand::ExtensionsUpdated()
                            .to_request(gen_channel_id(), "".into())
                            .unwrap(),
                    )
                    .unwrap();
            }

            thread::spawn(move || {
                let plugin_mutex = Self::spawn_extension(
                    manifest,
                    reply_map,
                    ext_command_tx.clone(),
                    cache_path.clone(),
                );
                {
                    let mut plugin = block_on(plugin_mutex.lock());

                    tracing::trace!("Callign entry");
                    if let Err(e) = plugin.call::<(), ()>("entry", ()) {
                        tracing::error!("Failed to called extension entry: {:?}", e);
                        return;
                    }
                }
                {
                    let mut extensions_map = block_on(extension_map.lock());
                    if let Some(ext) = extensions_map.get_mut(&package_name) {
                        ext.plugin = Some(plugin_mutex);
                        ext.active = true;
                    } else {
                        panic!("Where the fuck did the extension disappear");
                    }
                }

                ext_command_tx
                    .send(
                        MainCommand::ExtensionsUpdated()
                            .to_request(gen_channel_id(), "".into())
                            .unwrap(),
                    )
                    .unwrap();
            });
        }

        if let Err(e) = self.ext_command_tx.send(
            MainCommand::ExtensionsUpdated()
                .to_request(gen_channel_id(), "".into())
                .unwrap(),
        ) {
            tracing::error!("Failed to send extension update command: {:?}", e);
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn get_extensions(&self, package_name: String) -> Vec<Extension> {
        let mut plugins = vec![];
        let extensions_map = self.extensions_map.lock().await;
        if package_name.is_empty() {
            plugins.extend(extensions_map.values().cloned());
        } else {
            let plugin = extensions_map.get(&package_name).cloned();
            if let Some(plugin) = plugin {
                plugins.push(plugin);
            }
        }
        plugins
    }

    fn sanitize_response(response: &mut ExtensionCommandResponse, package_name: String) {
        match response {
            ExtensionCommandResponse::GetProviderScopes(_) => {}
            ExtensionCommandResponse::GetAccounts(accounts) => {
                for account in accounts {
                    account.package_name = package_name.clone();
                }
            }
            ExtensionCommandResponse::PerformAccountLogin(_) => {}
            ExtensionCommandResponse::ExtraExtensionEvent(resp) => {
                let prefix = format!("{}:", package_name);
                let resp = resp.as_mut();
                match resp {
                    ExtensionExtraEventResponse::RequestedPlaylists(playlist_return_type) => {
                        playlist_return_type
                            .playlists
                            .iter_mut()
                            .for_each(|p| sanitize_playlist(&prefix, p));
                    }
                    ExtensionExtraEventResponse::RequestedPlaylistSongs(
                        songs_with_page_token_return_type,
                    ) => {
                        songs_with_page_token_return_type
                            .songs
                            .iter_mut()
                            .for_each(|s| sanitize_song(&prefix, s));
                    }
                    ExtensionExtraEventResponse::OauthCallback => {}
                    ExtensionExtraEventResponse::SongQueueChanged => {}
                    ExtensionExtraEventResponse::Seeked => {}
                    ExtensionExtraEventResponse::VolumeChanged => {}
                    ExtensionExtraEventResponse::PlayerStateChanged => {}
                    ExtensionExtraEventResponse::SongChanged => {}
                    ExtensionExtraEventResponse::PreferenceChanged => {}
                    ExtensionExtraEventResponse::PlaybackDetailsRequested(_) => {}
                    ExtensionExtraEventResponse::CustomRequest(_) => {}
                    ExtensionExtraEventResponse::RequestedSongFromURL(song_return_type) => {
                        if let Some(song) = song_return_type.song.as_mut() {
                            sanitize_song(&prefix, song);
                        }
                    }
                    ExtensionExtraEventResponse::RequestedPlaylistFromURL(
                        playlist_and_songs_return_type,
                    ) => {
                        if let Some(playlist) = playlist_and_songs_return_type.playlist.as_mut() {
                            sanitize_playlist(&prefix, playlist);
                        }

                        if let Some(songs) = playlist_and_songs_return_type.songs.as_mut() {
                            songs.iter_mut().for_each(|s| sanitize_song(&prefix, s));
                        }
                    }
                    ExtensionExtraEventResponse::RequestedSearchResult(search_return_type) => {
                        search_return_type
                            .songs
                            .iter_mut()
                            .for_each(|s| sanitize_song(&prefix, s));
                        search_return_type
                            .albums
                            .iter_mut()
                            .for_each(|s| sanitize_album(&prefix, s));
                        search_return_type
                            .artists
                            .iter_mut()
                            .for_each(|s| sanitize_artist(&prefix, s));
                        search_return_type
                            .playlists
                            .iter_mut()
                            .for_each(|s| sanitize_playlist(&prefix, s));
                    }
                    ExtensionExtraEventResponse::RequestedRecommendations(
                        recommendations_return_type,
                    ) => {
                        recommendations_return_type
                            .songs
                            .iter_mut()
                            .for_each(|s| sanitize_song(&prefix, s));
                    }
                    ExtensionExtraEventResponse::RequestedLyrics(_) => {}
                    ExtensionExtraEventResponse::RequestedArtistSongs(
                        songs_with_page_token_return_type,
                    ) => {
                        songs_with_page_token_return_type
                            .songs
                            .iter_mut()
                            .for_each(|s| sanitize_song(&prefix, s));
                    }
                    ExtensionExtraEventResponse::RequestedAlbumSongs(
                        songs_with_page_token_return_type,
                    ) => {
                        songs_with_page_token_return_type
                            .songs
                            .iter_mut()
                            .for_each(|s| sanitize_song(&prefix, s));
                    }
                    ExtensionExtraEventResponse::SongAdded => {}
                    ExtensionExtraEventResponse::SongRemoved => {}
                    ExtensionExtraEventResponse::PlaylistAdded => {}
                    ExtensionExtraEventResponse::PlaylistRemoved => {}
                    ExtensionExtraEventResponse::RequestedSongFromId(song_return_type) => {
                        if let Some(song) = song_return_type.song.as_mut() {
                            sanitize_song(&prefix, song);
                        }
                    }
                    ExtensionExtraEventResponse::GetRemoteURL(_) => {}
                    ExtensionExtraEventResponse::Scrobble => {}
                    ExtensionExtraEventResponse::RequestedSongContextMenu(
                        _context_menu_return_type,
                    ) => {}
                    ExtensionExtraEventResponse::RequestedPlaylistContextMenu(
                        _context_menu_return_type,
                    ) => {}
                    ExtensionExtraEventResponse::ContextMenuAction => {}
                }
            }
            ExtensionCommandResponse::Empty => {}
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn execute_command(
        &mut self,
        command: &ExtensionCommand,
        tx: MainCommandReplySender,
    ) -> MoosyncResult<()> {
        let (package_name, fn_name, args) = command.to_plugin_call();
        let plugins = self.get_extensions(package_name.clone()).await;

        let plugins_len = plugins.len();

        for extension in plugins {
            let command = command.clone();
            let args = args.clone();
            let extension = extension.clone();
            let package_name = package_name.clone();
            let tx = tx.clone();
            thread::spawn(move || {
                if !extension.active {
                    return;
                }

                if let Some(plugin) = &extension.plugin {
                    let mut plugin = block_on(plugin.lock());
                    let res = plugin.call::<_, Value>(fn_name, args.clone());
                    match res {
                        Ok(res) => match command.parse_response(res.clone()) {
                            Ok(mut parsed_response) => {
                                Self::sanitize_response(&mut parsed_response, package_name.clone());
                                if plugins_len == 1 {
                                    let _ = tx.send(parsed_response);
                                }
                            }
                            Err(e) => {
                                if plugins_len == 1 {
                                    let _ = tx.send(ExtensionCommandResponse::Empty);
                                    tracing::error!(
                                        "Failed to parse response from extension {} {:?}: {:?}",
                                        package_name,
                                        e,
                                        res
                                    );
                                }
                            }
                        },
                        Err(e) => {
                            if plugins_len == 1 {
                                let _ = tx.send(ExtensionCommandResponse::Empty);
                                tracing::error!(
                                    "Extension {} responsed with error: {:?}",
                                    extension.package_name,
                                    e
                                );
                            }
                        }
                    }
                }
            });
        }

        if plugins_len > 1 {
            let _ = tx.send(ExtensionCommandResponse::Empty);
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn remove_extension(&mut self, package_name: &String) {
        let mut extensions_map = self.extensions_map.lock().await;
        extensions_map.remove(package_name);
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn handle_extension_command(
        &mut self,
        command: &ExtensionCommand,
        tx: MainCommandReplySender,
    ) -> MoosyncResult<()> {
        tracing::debug!("Executing command {:?}", command);
        return self.execute_command(command, tx).await;
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn handle_runner_command(
        &mut self,
        command: RunnerCommand,
    ) -> MoosyncResult<RunnerCommandResp> {
        tracing::info!("Got runner command {:?}", command);
        let ret = match command {
            RunnerCommand::GetInstalledExtensions => {
                let extensions_map = self.extensions_map.lock().await;
                let extensions = extensions_map
                    .values()
                    .map(|e| e.into())
                    .collect::<Vec<ExtensionDetail>>();
                tracing::debug!("Extension map: {:?}", extensions);
                RunnerCommandResp::ExtensionList(extensions)
            }
            RunnerCommand::FindNewExtensions => {
                self.spawn_extensions().await;
                RunnerCommandResp::Empty()
            }
            RunnerCommand::GetExtensionIcon(p) => RunnerCommandResp::ExtensionIcon(
                self.get_extensions(p.package_name)
                    .await
                    .first()
                    .map(|e| e.icon.clone()),
            ),
            RunnerCommand::ToggleExtensionStatus(_) => todo!(),
            RunnerCommand::RemoveExtension(p) => {
                self.remove_extension(&p.package_name).await;
                RunnerCommandResp::Empty()
            }
            RunnerCommand::StopProcess => {
                todo!()
            }
            RunnerCommand::GetDisplayName(p) => RunnerCommandResp::ExtensionIcon(
                self.get_extensions(p.package_name)
                    .await
                    .first()
                    .map(|e| e.name.clone()),
            ),
        };

        tracing::debug!("Got runner command response {:?}", ret);
        Ok(ret)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub(crate) fn handle_main_command_reply(
        &self,
        resp: &GenericExtensionHostRequest<MainCommandResponse>,
    ) -> MoosyncResult<()> {
        let reply_map = self.reply_map.lock().unwrap();

        tracing::trace!("Inside reply {:?} {:?}", reply_map, resp);
        if let Some(tx) = reply_map.get(&resp.channel) {
            tracing::trace!("Handling as reply");
            tx.send(resp.clone()).unwrap();
            return Ok(());
        }

        Ok(())
    }

    pub async fn register_ui_preferences(
        &self,
        package_name: String,
        prefs: Vec<PreferenceUIData>,
    ) -> MoosyncResult<()> {
        let mut extensions = self.extensions_map.lock().await;
        if let Some(ext) = extensions.get_mut(&package_name) {
            for pref in prefs {
                ext.preferences.insert(pref.key.clone(), pref);
            }

            return Ok(());
        }

        Err("Extension not found".into())
    }

    pub async fn unregister_ui_preferences(
        &self,
        package_name: String,
        pref_keys: Vec<String>,
    ) -> MoosyncResult<()> {
        let mut extensions = self.extensions_map.lock().await;
        if let Some(ext) = extensions.get_mut(&package_name) {
            for pref in pref_keys {
                ext.preferences.remove(&pref);
            }

            return Ok(());
        }

        Err("Extension not found".into())
    }
    // #[tracing::instrument(level = "debug", skip(self))]
    // pub async fn listen_command_once(&mut self) {
    //     if let Some(resp) = &self.main_command_rx.recv().await {
    //         tracing::debug!("Got command {:?}", resp);
    //         self.handle_extension_command(resp).await
    //     }
    // }
}

fn gen_channel_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
