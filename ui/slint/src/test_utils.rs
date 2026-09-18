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

use rstest::fixture;
use state_manager::StateManager;
use tempdir::TempDir;
use types::plugin::PluginContext;

use crate::MainWindow;

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
pub fn main_window() -> MainWindow {
    i_slint_backend_testing::init_no_event_loop();
    MainWindow::new().expect("failed to create MainWindow")
}

pub struct TestSlintSmContext {
    pub _temp_dir: TempDir,
    pub sm: StateManager,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
pub fn state_manager_fixture() -> TestSlintSmContext {
    let temp_dir = TempDir::new("moosync_slint_test").expect("failed to create temp dir");
    let test_dir = temp_dir.path().to_path_buf();
    let context = PluginContext {
        data_dir: test_dir.clone(),
        cache_dir: test_dir.clone(),
        tmp_dir: test_dir.clone(),
        #[cfg(target_os = "android")]
        android_context: types::android::AndroidJNIContext::default(),
    };
    let sm = StateManager::new_with_context(context).expect("failed to create state manager");
    TestSlintSmContext {
        _temp_dir: temp_dir,
        sm,
    }
}

#[macro_export]
macro_rules! integration_test {
    ($($test_name:ident => $async_fn:ident),* $(,)?) => {
        $(
            #[test]
            #[tracing_test::traced_test]
            #[tracing::instrument(level = "debug", skip_all)]
            fn $test_name() {
                $crate::test_utils::integration::run_slint_test(stringify!($test_name), $async_fn);
            }
        )*
    };
}

#[macro_export]
macro_rules! parameterized_test {
    ($handler_fn:ident, $(($test_name:ident, $($arg:expr),* $(,)?)),* $(,)?) => {
        $(
            #[test]
            #[tracing_test::traced_test]
            #[tracing::instrument(level = "debug", skip_all)]
            fn $test_name() {
                $crate::test_utils::integration::run_slint_test(stringify!($test_name), |mw, sm| $handler_fn(mw, sm, $($arg),*));
            }
        )*
    };
}

pub mod integration {
    use std::{env, fs, path::PathBuf, time::Duration};

    use futures::FutureExt;
    use i_slint_backend_testing::ElementHandle;
    use player::RepeatMode;
    use slint::{ComponentHandle, ModelRc, VecModel};
    use songs_proto::moosync::types::{
        Album, Artist, Genre, GetEntityOptions, GetSongOptions, InnerSong, Playlist,
        SearchableSong, Song, SongType, entity_result,
    };
    use state_manager::StateManager;

    use crate::{
        AccountsProps, AlbumContentPageProps, AlbumsPageProps, AllSongsPageProps, AppCallbacks,
        ArtistContentPageProps, ArtistsPageProps, ExtensionsPageProps, ExtensionsPreferenceProps,
        GenreContentPageProps, MainWindow, OAuthState, Pages, PlayerProps,
        PlaylistContentPageProps, PlaylistsPageProps, QueuePageProps, SavePlaylistState,
        SaveThemeState, SearchPageProps, SettingsPages, SettingsState, SongModel, ThemesPageProps,
        WindowInfo, setup_ui, test_utils::state_manager_fixture,
    };
    pub use crate::{integration_test, parameterized_test};

    pub type TestFuture = std::pin::Pin<Box<dyn std::future::Future<Output = ()> + 'static>>;
    pub type TestFn =
        Box<dyn FnOnce(&'static MainWindow, &'static StateManager) -> TestFuture + Send + 'static>;
    pub type Task = (
        &'static str,
        TestFn,
        std::sync::mpsc::Sender<std::thread::Result<()>>,
    );

    static RUNNER: std::sync::OnceLock<tokio::sync::mpsc::UnboundedSender<Task>> =
        std::sync::OnceLock::new();

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn runtime() -> &'static tokio::runtime::Runtime {
        static RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
        RUNTIME.get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
        })
    }

    static TEST_STEP: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_test_step(step: &str) { *TEST_STEP.lock().unwrap() = step.to_string(); }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn runner() -> &'static tokio::sync::mpsc::UnboundedSender<Task> {
        RUNNER.get_or_init(|| {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Task>();

            std::thread::Builder::new()
                .name("slint_test_runner".into())
                .spawn(move || {
                    let _guard = runtime().enter();
                    i_slint_backend_testing::init_integration_test_with_system_time();

                    let fixture = Box::leak(Box::new(state_manager_fixture()));
                    let state_manager: &'static StateManager = &fixture.sm;
                    let main_window: &'static MainWindow =
                        Box::leak(Box::new(MainWindow::new().unwrap()));

                    main_window
                        .window()
                        .set_size(slint::PhysicalSize::new(1920, 1080));
                    main_window.global::<WindowInfo>().set_window_width(1920.0);
                    main_window.global::<WindowInfo>().set_window_height(1080.0);

                    runtime().block_on(async {
                        setup_test_context(state_manager).await;
                        setup_ui(main_window, state_manager);
                    });

                    slint::spawn_local(async move {
                        while let Some((test_name, test_fn, result_tx)) = rx.recv().await {
                            set_test_step("reset_test_state");
                            let res = std::panic::AssertUnwindSafe(async {
                                reset_test_state(main_window, state_manager).await;
                                set_test_step("test_fn_start");
                                let test_future = test_fn(main_window, state_manager);

                                if tokio::time::timeout(Duration::from_secs(30), test_future)
                                    .await
                                    .is_err()
                                {
                                    let current_step = TEST_STEP.lock().unwrap().clone();
                                    panic!(
                                        "Test {} timed out after 30s at step: {}",
                                        test_name, current_step
                                    );
                                }
                            })
                            .catch_unwind()
                            .await;

                            let _ = result_tx.send(res);
                        }
                    })
                    .unwrap();

                    slint::run_event_loop().unwrap();
                })
                .expect("failed to spawn slint runner");

            tx
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn reset_test_state(
        main_window: &'static MainWindow,
        state_manager: &'static StateManager,
    ) {
        // 1. Remove all installed custom extensions
        {
            let ext_handler = state_manager.get_extension_handler().await;
            for detail in ext_handler.get_installed_extensions() {
                let _ = ext_handler.remove_extension(detail.package_name);
            }
            if let Ok(entries) = fs::read_dir(&ext_handler.extensions_dir) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        let _ = fs::remove_dir_all(entry.path());
                    }
                }
            }
            let _ = ext_handler.find_new_extensions();
            ext_handler.trigger_extensions_updated();
            ext_handler.trigger_accounts_updated(None);
            ext_handler.trigger_preferences_updated(String::new());
        }

        // 2. Clear Database
        {
            let database = state_manager.get_database().await;
            if let Ok(songs) = database.get_songs_by_options(GetSongOptions {
                song: Some(SearchableSong::default()),
                ..Default::default()
            }) {
                let ids: Vec<String> = songs
                    .into_iter()
                    .filter_map(|s| s.song.and_then(|is| is.id))
                    .collect();
                if !ids.is_empty() {
                    let _ = database.remove_songs(&ids);
                }
            }
            if let Ok(playlists_res) = database.get_entity_by_options(GetEntityOptions {
                playlist: Some(Playlist::default()),
                ..Default::default()
            }) {
                if let Some(entity_result::Result::Playlists(pl)) = playlists_res.result {
                    for p in pl.playlists {
                        if let Some(id) = p.playlist_id {
                            let _ = database.remove_playlist(&id);
                        }
                    }
                }
            }
        }

        // 3. Reset Player Handler
        {
            let mut ph = state_manager.get_player_handler_mut().await;
            ph.clear_queue();
            ph.clear_queue();
            let _ = ph.pause();
            ph.repeat(RepeatMode::None);
        }

        // 4. Reset Slint UI Modals and State
        main_window
            .global::<SettingsState>()
            .set_show_settings(false);
        main_window
            .global::<SettingsState>()
            .set_active_page(SettingsPages::Paths);
        main_window.global::<QueuePageProps>().set_show_queue(false);
        main_window
            .global::<OAuthState>()
            .set_show_oauth_modal(false);
        main_window
            .global::<SavePlaylistState>()
            .set_show_modal(false);
        main_window.global::<PlayerProps>().set_playing(false);
        main_window
            .global::<PlayerProps>()
            .set_current_song(SongModel::default());
        main_window.global::<PlayerProps>().set_repeat_mode(0);
        main_window
            .global::<QueuePageProps>()
            .set_queue(ModelRc::new(VecModel::default()));
        main_window
            .global::<AccountsProps>()
            .set_accounts(ModelRc::new(VecModel::default()));
        main_window
            .global::<AllSongsPageProps>()
            .set_songs(ModelRc::new(VecModel::default()));
        main_window
            .global::<AlbumsPageProps>()
            .set_albums(ModelRc::new(VecModel::default()));
        main_window
            .global::<ArtistsPageProps>()
            .set_artists(ModelRc::new(VecModel::default()));
        main_window
            .global::<PlaylistsPageProps>()
            .set_playlists(ModelRc::new(VecModel::default()));
        main_window
            .global::<PlaylistContentPageProps>()
            .set_songs(ModelRc::new(VecModel::default()));
        main_window
            .global::<PlaylistContentPageProps>()
            .set_extension_providers(ModelRc::new(VecModel::default()));
        main_window
            .global::<AlbumContentPageProps>()
            .set_songs(ModelRc::new(VecModel::default()));
        main_window
            .global::<AlbumContentPageProps>()
            .set_extension_providers(ModelRc::new(VecModel::default()));
        main_window
            .global::<ArtistContentPageProps>()
            .set_songs(ModelRc::new(VecModel::default()));
        main_window
            .global::<ArtistContentPageProps>()
            .set_extension_providers(ModelRc::new(VecModel::default()));
        main_window
            .global::<GenreContentPageProps>()
            .set_songs(ModelRc::new(VecModel::default()));
        main_window
            .global::<SearchPageProps>()
            .set_search_query("".into());
        main_window
            .global::<SearchPageProps>()
            .set_provider_results(ModelRc::new(VecModel::default()));
        main_window.global::<SearchPageProps>().set_active_tab(0);
        main_window
            .global::<SearchPageProps>()
            .set_selected_provider(0);
        main_window
            .global::<ExtensionsPreferenceProps>()
            .set_extension_preferences(ModelRc::new(VecModel::default()));
        main_window
            .global::<ExtensionsPageProps>()
            .set_extensions(ModelRc::new(VecModel::default()));
        main_window
            .global::<ThemesPageProps>()
            .set_available_themes(ModelRc::new(VecModel::default()));
        main_window
            .global::<ThemesPageProps>()
            .set_theme_constants(ModelRc::new(VecModel::default()));
        main_window.global::<SaveThemeState>().set_show_modal(false);
        main_window
            .global::<AppCallbacks>()
            .invoke_settings_toggled(false);
        main_window
            .global::<AppCallbacks>()
            .invoke_settings_active_page_changed(SettingsPages::Paths);
        main_window
            .global::<AppCallbacks>()
            .invoke_queue_toggled(false);
        main_window
            .global::<AppCallbacks>()
            .invoke_active_page_changed(Pages::AllSongs);

        tokio::time::sleep(Duration::from_millis(260)).await;
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn run_slint_test<F, Fut>(test_name: &'static str, test_fn: F)
    where
        F: FnOnce(&'static MainWindow, &'static StateManager) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + 'static,
    {
        let (tx, rx) = std::sync::mpsc::channel();
        let boxed_fn: TestFn = Box::new(move |mw, sm| Box::pin(test_fn(mw, sm)));

        runner()
            .send((test_name, boxed_fn, tx))
            .expect("failed to send task to slint runner");

        match rx.recv() {
            Ok(Ok(())) => {}
            Ok(Err(panic_err)) => {
                std::panic::resume_unwind(panic_err);
            }
            Err(e) => {
                panic!("runner panicked before completing test: {:?}", e);
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn wait_until<F>(mut condition: F) -> bool
    where
        F: FnMut() -> bool,
    {
        for _ in 0..1400 {
            if condition() {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        condition()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn click_element(handle: &ElementHandle) {
        click_element_with_button(handle, slint::platform::PointerEventButton::Left).await;
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn click_element_with_button(
        handle: &ElementHandle,
        button: slint::platform::PointerEventButton,
    ) {
        handle.single_click(button).await;
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn setup_test_context(state_manager: &StateManager) {
        state_manager.setup().await;
        let mut ph = state_manager.get_player_handler_mut().await;
        ph.set_context(Box::new(player::DummyAudioPlayerContext::new()));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn create_test_song(id: &str, title: &str, album: &str, artist: &str) -> Song {
        Song {
            song: Some(InnerSong {
                id: Some(id.to_string()),
                title: Some(title.to_string()),
                playback_url: Some(format!("https://example.com/{}", id)),
                path: Some(format!("/music/{}.mp3", id)),
                r#type: SongType::Local.into(),
                duration: Some(songs_proto::duration_proto::google::protobuf::Duration {
                    seconds: 180,
                    nanos: 0,
                }),
                ..Default::default()
            }),
            album: Some(Album {
                album_id: Some(format!("album_{}", id)),
                album_name: Some(album.to_string()),
                ..Default::default()
            }),
            artists: vec![Artist {
                artist_id: Some(format!("artist_{}", id)),
                artist_name: Some(artist.to_string()),
                ..Default::default()
            }],
            genre: vec![Genre {
                genre_id: Some(format!("genre_{}", id)),
                genre_name: Some("Rock".to_string()),
                ..Default::default()
            }],
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_sample_ext_path(subdir: &str) -> PathBuf {
        let runfiles_dir = env::var("TEST_SRCDIR").unwrap_or_else(|_| ".".to_string());
        let r = PathBuf::from(&runfiles_dir);
        let sub = if subdir.is_empty() { "rs" } else { subdir };
        let candidates = [
            r.join("moosync_ext+/sample_extensions").join(sub),
            r.join("moosync_ext/sample_extensions").join(sub),
            r.join("_main/sample_extensions").join(sub),
            r.join("_main/external/moosync_ext+/sample_extensions")
                .join(sub),
            r.join("_main/external/moosync_ext/sample_extensions")
                .join(sub),
            r.join("sample_extensions").join(sub),
        ];
        for c in &candidates {
            if c.join("sample_extension.wasm").exists() {
                return c.clone();
            }
        }
        for c in &candidates {
            if c.exists() {
                return c.clone();
            }
        }
        candidates[0].clone()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn load_custom_extension(
        state_manager: &StateManager,
        dir_name: &str,
        pkg_name: &str,
        display_name: &str,
    ) {
        let ext_handler = state_manager.get_extension_handler().await;
        let extensions_dir = ext_handler.extensions_dir.clone();
        let ext_dir = extensions_dir.join(dir_name);
        fs::create_dir_all(&ext_dir).unwrap();

        let src_ext_path = get_sample_ext_path("rs");
        assert!(
            src_ext_path.exists(),
            "Source extension path does not exist: {:?}",
            src_ext_path
        );
        for entry in fs::read_dir(&src_ext_path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                let dest = ext_dir.join(path.file_name().unwrap());
                fs::copy(&path, &dest).unwrap();
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let perms = fs::Permissions::from_mode(0o644);
                    fs::set_permissions(&dest, perms).unwrap();
                }
            }
        }

        let package_json_path = ext_dir.join("package.json");
        if package_json_path.exists() {
            let contents = fs::read_to_string(&package_json_path).unwrap();
            if let Ok(mut json_val) = serde_json::from_str::<serde_json::Value>(&contents) {
                if let Some(obj) = json_val.as_object_mut() {
                    obj.insert(
                        "name".to_string(),
                        serde_json::Value::String(pkg_name.to_string()),
                    );
                    obj.insert(
                        "displayName".to_string(),
                        serde_json::Value::String(display_name.to_string()),
                    );
                    if !obj.contains_key("icon") {
                        obj.insert(
                            "icon".to_string(),
                            serde_json::Value::String("icon.png".to_string()),
                        );
                    }
                }
                fs::write(
                    &package_json_path,
                    serde_json::to_string(&json_val).unwrap(),
                )
                .unwrap();
            }
        } else {
            let manifest = format!(
                r#"{{
            "name": "{}",
            "displayName": "{}",
            "version": "1.0.0",
            "extensionEntry": "sample_extension.wasm",
            "moosyncExtension": true,
            "description": "Sample Rust Extension",
            "icon": "icon.png",
            "author": "Moosync"
        }}"#,
                pkg_name, display_name
            );
            fs::write(&package_json_path, manifest).unwrap();
        }

        set_test_step("load_custom_ext_find_new");
        ext_handler.find_new_extensions().unwrap();
        set_test_step("load_custom_ext_wait_started");
        let loaded = wait_until(|| {
            ext_handler
                .get_active_extensions()
                .iter()
                .any(|e| e.get_package_name() == pkg_name && e.get_extension_detail().has_started)
        })
        .await;
        assert!(loaded);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn load_sample_extension(state_manager: &StateManager) {
        load_custom_extension(state_manager, "rs.sample", "rs.sample", "Sample Extension").await;
        let ext_handler = state_manager.get_extension_handler().await;
        if let Ok(ext) = ext_handler.get_extension("rs.sample") {
            ext.set_account(extensions_proto::moosync::types::ExtensionAccountDetail {
                id: "sample_spotify".into(),
                package_name: "rs.sample".into(),
                name: "Spotify".into(),
                bg_color: "#1ed760".into(),
                icon: "spotify".into(),
                logged_in: false,
                ..Default::default()
            });
            ext.register_ui_preferences(vec![
                preferences_proto::moosync::types::PreferenceItem {
                    id: "api_key".to_string(),
                    title: "API Key".to_string(),
                    subtitle: "API Key Description".to_string(),
                    pref_type: preferences_proto::moosync::types::PreferenceType::TextInputGroup
                        as i32,
                    default: Some(preferences_proto::moosync::types::PreferenceValue {
                        value: Some(
                            preferences_proto::moosync::types::preference_value::Value::StringValue(
                                "default_key".to_string(),
                            ),
                        ),
                    }),
                    ..Default::default()
                },
                preferences_proto::moosync::types::PreferenceItem {
                    id: "enable_feature".to_string(),
                    title: "Enable Feature".to_string(),
                    subtitle: "Enable Feature Description".to_string(),
                    pref_type: preferences_proto::moosync::types::PreferenceType::ToggleGroup
                        as i32,
                    default: Some(preferences_proto::moosync::types::PreferenceValue {
                        value: Some(
                            preferences_proto::moosync::types::preference_value::Value::BoolValue(
                                false,
                            ),
                        ),
                    }),
                    ..Default::default()
                },
            ]);
            ext_handler.trigger_preferences_updated("rs.sample".to_string());
            ext_handler.trigger_extensions_updated();
            ext_handler.trigger_accounts_updated(None);
        }
    }
}
