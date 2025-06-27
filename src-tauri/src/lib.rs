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

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::fs;

use clap::Parser;
use db::is_song_in_playlist;
use extensions::get_extension_state;
use librespot::{
    get_canvaz, get_librespot_state, is_initialized, librespot_close, librespot_get_token,
    librespot_load, librespot_pause, librespot_play, librespot_seek, librespot_volume,
    register_event,
};
use logger::{get_logger_state, renderer_write};
use lyrics::{get_lyrics, get_lyrics_state};
use mobile_player::{
    mobile_load, mobile_pause, mobile_play, mobile_seek, mobile_stop, MobilePlayer,
};
use mpris::{get_mpris_state, set_metadata, set_playback_state, set_position};
use preference_holder::{
    get_preference_state, get_secure, handle_pref_changes, initial, load_selective,
    load_selective_array, save_selective, set_secure,
};
use providers::handler::get_provider_handler_state;
use rodio::{
    get_rodio_state, rodio_get_volume, rodio_load, rodio_pause, rodio_play, rodio_seek,
    rodio_set_volume, rodio_stop,
};
use tauri_plugin_deep_link::DeepLinkExt;
use themes::{
    download_theme, export_theme, get_css, get_theme_handler_state, get_themes_manifest,
    import_theme, load_all_themes, load_theme, remove_theme, save_theme,
};

use extensions::{
    download_extension, get_extension_icon, get_extension_manifest, get_installed_extensions,
    install_extension, remove_extension, send_extra_event,
};
use providers::handler::{
    fetch_playback_url, fetch_playlist_content, fetch_user_playlists, get_album_content,
    get_all_status, get_artist_content, get_playlist_context_menu, get_provider_key_by_id,
    get_provider_keys, get_provider_lyrics, get_song_context_menu, get_song_from_id,
    get_suggestions, initialize_all_providers, match_url, playlist_from_url, provider_authorize,
    provider_login, provider_search, provider_signout, song_from_url, trigger_context_menu_action,
};
use scanner::{get_scanner_state, start_scan, ScanTask};
use tauri::{Listener, Manager, State};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    filter::EnvFilter,
    fmt::{self},
    layer::SubscriberExt,
};
use updater::{fetch_update, get_updater_state, install_update};
use window::handler::{build_tray_menu, handle_window_close};

#[cfg(mobile)]
use tauri_plugin_audioplayer::AudioplayerExt;

use {
    db::{
        get_cache_state,
        {
            add_to_playlist, create_playlist, export_playlist, get_db_state, get_entity_by_options,
            get_songs_by_options, get_top_listened_songs, increment_play_count,
            increment_play_time, insert_songs, remove_from_playlist, remove_playlist, remove_songs,
            search_all, update_album, update_artist, update_lyrics, update_playlist, update_song,
            update_songs,
        },
    },
    oauth::handler::{get_oauth_state, OAuthHandler},
    window::handler::{
        close_window, disable_fullscreen, enable_fullscreen, get_platform, get_window_state,
        has_frame, is_maximized, maximize_window, minimize_window, open_external,
        open_file_browser, open_window, restart_app, toggle_dev_tools, toggle_fullscreen,
        update_zoom,
    },
    youtube::get_youtube_scraper_state,
};

mod db;
mod extensions;
mod librespot;
mod logger;
mod lyrics;
mod mobile_player;
mod mpris;
mod oauth;
mod preference_holder;
mod providers;
mod rodio;
mod scanner;
mod themes;
mod updater;
mod window;
mod youtube;

#[cfg(desktop)]
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    mobile: bool,
}

#[tracing::instrument(level = "debug", skip())]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = rustls::crypto::ring::default_provider().install_default();

    let filter = if cfg!(mobile) {
        EnvFilter::try_new("debug").unwrap()
    } else {
        EnvFilter::from_env("MOOSYNC_LOG")
    };

    #[cfg(desktop)]
    let args = Args::parse();

    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        // #[cfg(debug_assertions)]
        // {
        //     builder = tauri::Builder::default().plugin(tauri_plugin_devtools::init());
        // }

        builder = builder
            .plugin(tauri_plugin_updater::Builder::new().build())
            .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
                if let Some(url) = argv.get(1) {
                    tracing::info!("Got url {}", url);
                    let state: State<OAuthHandler> = app.state();
                    state.handle_oauth(app.clone(), url.to_string()).unwrap();
                }
            }))
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                None,
            ))
            .on_window_event(|window, event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    if let Ok(should_close) = handle_window_close(window.app_handle()) {
                        if !should_close {
                            window.hide().unwrap();
                            api.prevent_close();
                        }
                    }
                }
            });
    }

    #[cfg(mobile)]
    {
        builder = builder
            .plugin(tauri_plugin_file_scanner::init())
            .plugin(tauri_plugin_audioplayer::init());
    }

    #[cfg(desktop)]
    let is_mobile_init_script = format!(
        r#"
    window.is_mobile = {};
    window.is_mobile_player = false;
    "#,
        args.mobile
    );

    #[cfg(mobile)]
    let is_mobile_init_script = r#"
    window.is_mobile = true;
    window.is_mobile_player = true;
    "#
    .to_string();

    builder = builder
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .append_invoke_initialization_script(format!(
            r#"
            window.LOGGING_FILTER = "{}";
            {}
            "#,
            filter, is_mobile_init_script
        ))
        .invoke_handler(tauri::generate_handler![
            // Preferences
            save_selective,
            load_selective,
            load_selective_array,
            get_secure,
            set_secure,
            // DB
            insert_songs,
            remove_songs,
            get_songs_by_options,
            get_entity_by_options,
            search_all,
            create_playlist,
            add_to_playlist,
            is_song_in_playlist,
            remove_from_playlist,
            remove_playlist,
            update_album,
            update_artist,
            update_playlist,
            update_songs,
            update_song,
            update_lyrics,
            increment_play_count,
            increment_play_time,
            export_playlist,
            get_top_listened_songs,
            // Window
            is_maximized,
            has_frame,
            close_window,
            get_platform,
            maximize_window,
            minimize_window,
            update_zoom,
            open_external,
            open_window,
            enable_fullscreen,
            disable_fullscreen,
            toggle_fullscreen,
            toggle_dev_tools,
            restart_app,
            open_file_browser,
            // Scanner
            start_scan,
            // Librespot
            is_initialized,
            librespot_play,
            librespot_pause,
            librespot_close,
            librespot_load,
            librespot_seek,
            librespot_volume,
            librespot_get_token,
            register_event,
            get_canvaz,
            // Themes
            load_all_themes,
            load_theme,
            save_theme,
            remove_theme,
            import_theme,
            export_theme,
            get_css,
            get_themes_manifest,
            download_theme,
            // MPRIS
            set_metadata,
            set_playback_state,
            set_position,
            // Lyrics
            get_lyrics,
            // Extensions
            install_extension,
            remove_extension,
            download_extension,
            get_installed_extensions,
            get_extension_manifest,
            get_extension_icon,
            send_extra_event,
            //Provider Handler
            get_provider_keys,
            initialize_all_providers,
            provider_login,
            provider_signout,
            provider_authorize,
            get_provider_key_by_id,
            fetch_user_playlists,
            fetch_playlist_content,
            fetch_playback_url,
            provider_search,
            get_all_status,
            match_url,
            playlist_from_url,
            song_from_url,
            get_song_from_id,
            get_suggestions,
            get_album_content,
            get_artist_content,
            get_provider_lyrics,
            get_song_context_menu,
            get_playlist_context_menu,
            trigger_context_menu_action,
            // Rodio player
            rodio_get_volume,
            rodio_load,
            rodio_pause,
            rodio_play,
            rodio_seek,
            rodio_set_volume,
            rodio_stop,
            // Logger
            renderer_write,
            // Mobile player
            mobile_load,
            mobile_play,
            mobile_pause,
            mobile_stop,
            mobile_seek,
            // Updater
            fetch_update,
            install_update
        ])
        .setup(|app| {
            let layer = fmt::layer()
                .pretty()
                .with_target(true)
                .with_ansi(!cfg!(mobile));
            let log_path = app.path().app_log_dir()?;
            if !log_path.exists() {
                fs::create_dir_all(log_path.clone())?;
            }
            let file_appender = RollingFileAppender::new(Rotation::DAILY, log_path, "moosync");
            let log_layer = fmt::layer()
                .pretty()
                .with_ansi(false)
                .with_target(true)
                .with_writer(file_appender);
            let subscriber = tracing_subscriber::registry()
                .with(layer)
                .with(log_layer)
                .with(filter);

            tracing::subscriber::set_global_default(subscriber).unwrap();

            let db = get_db_state(app);
            app.manage(db);

            let cache = get_cache_state(app);
            app.manage(cache);

            let config = get_preference_state(app)?;
            app.manage(config);

            let oauth = get_oauth_state()?;
            app.manage(oauth);

            let window_state = get_window_state();
            app.manage(window_state);

            let yt_state = get_youtube_scraper_state();
            app.manage(yt_state);

            let scanner_state = get_scanner_state();
            app.manage(scanner_state);

            let scan_task = ScanTask::default();
            app.manage(scan_task);

            let librespot_state = get_librespot_state();
            app.manage(librespot_state);

            let theme_handler_state = get_theme_handler_state(app);
            app.manage(theme_handler_state);

            let mpris_state = get_mpris_state(app.app_handle().clone())?;
            app.manage(mpris_state);

            let lyrics_state = get_lyrics_state();
            app.manage(lyrics_state);

            let provider_handler_state = get_provider_handler_state(app.app_handle().clone());
            app.manage(provider_handler_state);

            let rodio_state = get_rodio_state(app.app_handle().clone());
            app.manage(rodio_state);

            let logger = get_logger_state(app.app_handle().clone());
            app.manage(logger);

            let mobile_player = MobilePlayer::new();
            app.manage(mobile_player);

            let updater_state = get_updater_state();
            app.manage(updater_state);

            #[cfg(mobile)]
            {
                use tauri::plugin::PermissionState;
                let audioplayer = app.audioplayer();
                if let Ok(permissions) = audioplayer.check_permissions() {
                    if permissions.read_media != PermissionState::Granted {
                        if let Err(e) = audioplayer.request_read_media_permission() {
                            tracing::error!("Error requesting permissions {:?}", e);
                        }
                    }
                }
            }

            let ext_state = get_extension_state(app.app_handle().clone())?;
            app.manage(ext_state);

            initial(app);
            handle_pref_changes(app.handle().clone());

            let app_cl = app.handle().clone();
            app.deep_link().on_open_url(move |ev| {
                if let Some(url) = ev.urls().first() {
                    let url = url.as_str().to_string();
                    tracing::info!("Got url {}", url);
                    let state: State<OAuthHandler> = app_cl.state();
                    state.handle_oauth(app_cl.clone(), url.to_string()).unwrap();
                }
            });

            build_tray_menu(app)?;

            Ok(())
        });

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
}
