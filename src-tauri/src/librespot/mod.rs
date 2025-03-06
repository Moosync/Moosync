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

use database::cache::CacheHolder;
use librespot::{
    spirc::ParsedToken, utils::event_to_map, Cache, ConnectConfig, Credentials, DeviceType,
    LibrespotHolder, PlayerConfig, PlayerEvent, REGISTERED_EVENTS,
};
use macros::{generate_command, generate_command_cached};

use tauri::{AppHandle, Emitter, Manager, State};
use types::{canvaz::CanvazResponse, errors::Result};

#[tracing::instrument(level = "trace", skip())]
pub fn get_librespot_state() -> LibrespotHolder {
    LibrespotHolder::new()
}

#[tracing::instrument(level = "trace", skip(app))]
#[tauri::command()]
// #[cfg(desktop)]
pub fn initialize_librespot(app: AppHandle, access_token: String) -> Result<()> {
    tracing::debug!("Initializing librespot with {:?}", access_token);
    let credentials = Credentials::with_access_token(access_token);

    let player_config = PlayerConfig::default();

    let connect_config = ConnectConfig {
        name: "Moosync".into(),
        device_type: DeviceType::Computer,
        initial_volume: 0,
        is_group: false,
        ..Default::default()
    };

    let credentials_path = app.path().app_config_dir()?;
    let audio_path = app.path().app_cache_dir()?;
    let cache_config = Cache::new(
        Some(credentials_path.clone()),
        Some(credentials_path),
        Some(audio_path),
        None,
    )?;

    let librespot: State<LibrespotHolder> = app.state();
    librespot.initialize(
        credentials,
        player_config,
        connect_config,
        cache_config,
        "".to_string(),
        "".to_string(),
    )?;

    // TODO: Check if event loop ends on closing librespot
    let events_channel = librespot.get_events_channel()?;
    tauri::async_runtime::spawn(async move {
        let events_channel = events_channel.lock().unwrap();
        loop {
            tracing::trace!("Waiting for librespot player events");
            let event = events_channel.recv();
            match event {
                Ok(event) => {
                    if let PlayerEvent::Unavailable {
                        play_request_id: _,
                        track_id: _,
                    } = event
                    {
                        tracing::error!("Got track unavailable {:?}", event);
                        continue;
                    }

                    let parsed_event = event_to_map(event.clone());

                    let registered_events = REGISTERED_EVENTS.lock().unwrap();
                    if registered_events.contains(&format!(
                        "librespot_event_{}",
                        parsed_event.get("event").unwrap(),
                    )) {
                        tracing::info!("Emitting event {:?}", parsed_event);
                        app.emit(
                            format!("librespot_event_{}", parsed_event.get("event").unwrap(),)
                                .as_str(),
                            parsed_event,
                        )
                        .unwrap();
                    }
                }
                Err(e) => {
                    tracing::error!("Ending event loop {:?}", e);
                    break;
                }
            }
        }
    });

    Ok(())
}

// #[cfg(mobile)]
// pub fn initialize_librespot(app: AppHandle, access_token: String) -> Result<()> {
//     use tauri_plugin_audioplayer::AudioplayerExt;
//     let audioplayer = app.audioplayer();
//     audioplayer.initialize_librespot(access_token)
// }

generate_command!(is_initialized, LibrespotHolder, bool,);
generate_command!(librespot_play, LibrespotHolder, (),);
generate_command!(librespot_pause, LibrespotHolder, (),);
generate_command!(librespot_close, LibrespotHolder, (),);
generate_command!(librespot_load, LibrespotHolder, (), uri: String, autoplay: bool);
generate_command!(librespot_seek, LibrespotHolder, (), pos: u32);
generate_command!(librespot_volume, LibrespotHolder, (), volume: u16);
generate_command!(librespot_get_token, LibrespotHolder, ParsedToken, scopes: String);
generate_command!(register_event, LibrespotHolder, (), event: String);
generate_command_cached!(get_canvaz, LibrespotHolder, CanvazResponse, uri: String);
