use std::time::Duration;

use i_slint_backend_testing::ElementHandle;
use preferences::keys::PreferenceItemExt;
use slint::{ComponentHandle, Model, ModelRc};
use slint_app::{
    AlbumContentPageProps, AlbumModel, AlbumsPageProps, AppCallbacks, ArtistContentPageProps,
    ArtistModel, ArtistsPageProps, ExtensionsPreferenceProps, MainWindow, Pages,
    PlaylistContentPageProps, PlaylistModel, PlaylistsPageProps, PreferenceChange, SettingsPages,
    SettingsState,
    test_utils::integration::{
        click_element, create_test_song, integration_test, load_custom_extension,
        load_sample_extension, set_test_step, wait_until,
    },
};
use songs_proto::moosync::types::{GetSongOptions, Playlist, SearchableSong};
use state_manager::StateManager;

#[tracing::instrument(level = "debug", skip_all)]
async fn do_extension_preference_text_input(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    main_window
        .global::<SettingsState>()
        .set_show_settings(false);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_toggled(false);
    tokio::time::sleep(Duration::from_millis(50)).await;

    set_test_step("text_input_load_sample_ext");
    load_sample_extension(state_manager).await;

    set_test_step("text_input_show_settings");
    main_window
        .global::<SettingsState>()
        .set_show_settings(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_toggled(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_active_page_changed(SettingsPages::Extensions);

    let tab_found = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Settings Extensions")
            .next()
            .is_some()
    })
    .await;
    if tab_found {
        let ext_tabs: Vec<ElementHandle> =
            ElementHandle::find_by_accessible_label(main_window, "Settings Extensions").collect();
        if !ext_tabs.is_empty() {
            click_element(&ext_tabs[0]).await;
        }
    }

    set_test_step("text_input_wait_preferences_loaded");
    let loaded = wait_until(|| {
        let groups = main_window
            .global::<ExtensionsPreferenceProps>()
            .get_extension_preferences();
        groups.row_count() > 0
            && groups
                .row_data(0)
                .is_some_and(|g| g.package_name == "rs.sample" && g.preferences.row_count() == 2)
    })
    .await;
    assert!(loaded);

    set_test_step("text_input_wait_api_key_handle");
    let found =
        wait_until(|| ElementHandle::find_by_accessible_label(main_window, "API Key").count() > 0)
            .await;
    assert!(found);

    let api_key_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "API Key").collect();
    assert!(!api_key_handles.is_empty());
    assert!(api_key_handles[0].is_valid());

    set_test_step("text_input_invoke_preference_changed");
    main_window
        .global::<AppCallbacks>()
        .invoke_preference_changed(PreferenceChange {
            id: "ext:rs.sample:api_key".into(),
            value_string: "secret_value_xyz".into(),
            value_bool: false,
            value_number: 0.0,
            value_list: ModelRc::default(),
        });

    set_test_step("text_input_wait_config_updated");
    let start = std::time::Instant::now();
    let mut updated = false;
    while start.elapsed() < Duration::from_secs(5) {
        let config = state_manager.get_preference_config().await;
        if config
            .get("ext:rs.sample:api_key")
            .and_then(|p| p.value::<String>())
            == Some("secret_value_xyz".to_string())
        {
            updated = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(updated);

    main_window
        .global::<SettingsState>()
        .set_show_settings(false);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_toggled(false);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_extension_preference_toggle(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    main_window
        .global::<SettingsState>()
        .set_show_settings(false);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_toggled(false);
    tokio::time::sleep(Duration::from_millis(50)).await;

    set_test_step("toggle_load_sample_ext");
    load_sample_extension(state_manager).await;

    set_test_step("toggle_show_settings");
    main_window
        .global::<SettingsState>()
        .set_show_settings(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_toggled(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_active_page_changed(SettingsPages::Extensions);

    let tab_found = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Settings Extensions")
            .next()
            .is_some()
    })
    .await;
    if tab_found {
        let ext_tabs: Vec<ElementHandle> =
            ElementHandle::find_by_accessible_label(main_window, "Settings Extensions").collect();
        if !ext_tabs.is_empty() {
            click_element(&ext_tabs[0]).await;
        }
    }

    set_test_step("toggle_wait_preferences_loaded");
    let loaded = wait_until(|| {
        let groups = main_window
            .global::<ExtensionsPreferenceProps>()
            .get_extension_preferences();
        groups.row_count() > 0
            && groups
                .row_data(0)
                .is_some_and(|g| g.package_name == "rs.sample" && g.preferences.row_count() == 2)
    })
    .await;
    assert!(loaded);

    set_test_step("toggle_wait_enable_feature_handle");
    let found = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Enable Feature").count() > 0
    })
    .await;
    assert!(found);

    let toggle_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Enable Feature").collect();
    assert!(!toggle_handles.is_empty());
    assert!(toggle_handles[0].is_valid());

    set_test_step("toggle_invoke_preference_changed");
    main_window
        .global::<AppCallbacks>()
        .invoke_preference_changed(PreferenceChange {
            id: "ext:rs.sample:enable_feature".into(),
            value_string: "".into(),
            value_bool: true,
            value_number: 0.0,
            value_list: ModelRc::default(),
        });

    set_test_step("toggle_wait_config_updated");
    let start = std::time::Instant::now();
    let mut updated = false;
    while start.elapsed() < Duration::from_secs(5) {
        let config = state_manager.get_preference_config().await;
        if config
            .get("ext:rs.sample:enable_feature")
            .and_then(|p| p.value::<bool>())
            == Some(true)
        {
            updated = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(updated);

    main_window
        .global::<SettingsState>()
        .set_show_settings(false);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_toggled(false);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_playlists_extension_integration(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let database = state_manager.get_database().await;
    let song = create_test_song("p1", "Local Song", "Local Album", "Local Artist");
    database.insert_songs(vec![song.clone()]).unwrap();
    let playlist = Playlist {
        playlist_id: Some("pl_local".into()),
        playlist_name: "Local Playlist".into(),
        ..Default::default()
    };
    database
        .create_playlist_with_songs(playlist, &[song])
        .unwrap();

    load_sample_extension(state_manager).await;

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Playlists);

    let loaded = wait_until(|| {
        let playlists = main_window.global::<PlaylistsPageProps>().get_playlists();
        playlists.row_count() == 3
    })
    .await;

    assert!(loaded);
    assert_eq!(
        main_window
            .global::<PlaylistsPageProps>()
            .get_playlists()
            .row_count(),
        3
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_extension_content_toggle_provider_off(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let database = state_manager.get_database().await;
    let song = create_test_song("p_ext_t", "Local Song", "Local Album", "Local Artist");
    database.insert_songs(vec![song.clone()]).unwrap();
    let playlist = Playlist {
        playlist_name: "Toggle Playlist".into(),
        ..Default::default()
    };
    let playlist_id = database.create_playlist(playlist).unwrap();
    database.add_to_playlist(&playlist_id, &[song]).unwrap();

    load_sample_extension(state_manager).await;

    main_window
        .global::<PlaylistsPageProps>()
        .set_selected_playlist(PlaylistModel {
            id: playlist_id.into(),
            title: "Toggle Playlist".into(),
            ..Default::default()
        });
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::PlaylistContent);

    let initial = wait_until(|| {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_extension_providers()
            .row_count()
            == 1
            && main_window
                .global::<PlaylistContentPageProps>()
                .get_songs()
                .row_count()
                == 1
    })
    .await;
    assert!(initial);

    main_window
        .global::<AppCallbacks>()
        .invoke_toggle_playlist_content_extension("rs.sample".into(), true);

    let loaded = wait_until(|| {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count()
            >= 26
    })
    .await;
    assert!(loaded);

    main_window
        .global::<AppCallbacks>()
        .invoke_toggle_playlist_content_extension("rs.sample".into(), false);

    let toggled_off = wait_until(|| {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count()
            == 1
    })
    .await;
    assert!(toggled_off);
    assert_eq!(
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count(),
        1
    );

    let song1_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Playlist Song 1").collect();
    assert_eq!(song1_handles.len(), 0);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_extension_content_toggle_provider_on(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let database = state_manager.get_database().await;
    let song = create_test_song("p_ext_on", "Local Song", "Local Album", "Local Artist");
    database.insert_songs(vec![song.clone()]).unwrap();
    let playlist = Playlist {
        playlist_name: "Toggle Playlist On".into(),
        ..Default::default()
    };
    let playlist_id = database.create_playlist(playlist).unwrap();
    database.add_to_playlist(&playlist_id, &[song]).unwrap();

    load_sample_extension(state_manager).await;

    main_window
        .global::<PlaylistsPageProps>()
        .set_selected_playlist(PlaylistModel {
            id: playlist_id.into(),
            title: "Toggle Playlist On".into(),
            ..Default::default()
        });
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::PlaylistContent);

    let initial = wait_until(|| {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_extension_providers()
            .row_count()
            == 1
            && main_window
                .global::<PlaylistContentPageProps>()
                .get_songs()
                .row_count()
                == 1
    })
    .await;
    assert!(initial);

    main_window
        .global::<AppCallbacks>()
        .invoke_toggle_playlist_content_extension("rs.sample".into(), true);

    let loaded = wait_until(|| {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count()
            >= 26
    })
    .await;
    assert!(loaded);

    main_window
        .global::<AppCallbacks>()
        .invoke_toggle_playlist_content_extension("rs.sample".into(), false);

    let _ = wait_until(|| {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count()
            == 1
    })
    .await;

    main_window
        .global::<AppCallbacks>()
        .invoke_toggle_playlist_content_extension("rs.sample".into(), true);

    let toggled_on = wait_until(|| {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count()
            >= 26
    })
    .await;
    assert!(toggled_on);
    assert!(
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count()
            >= 26
    );

    let songs = main_window.global::<PlaylistContentPageProps>().get_songs();
    assert_eq!(
        songs.row_data(0).map(|s| s.title),
        Some("Local Song".into())
    );
    assert_eq!(
        songs.row_data(1).map(|s| s.title),
        Some("Playlist Song 1".into())
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_artist_content_extension_toggle_and_open(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let database = state_manager.get_database().await;
    let song = create_test_song("art_ext", "Local Song", "Local Album", "Artist Ext");
    database.insert_songs(vec![song]).unwrap();
    let songs = database
        .get_songs_by_options(GetSongOptions {
            song: Some(SearchableSong {
                id: Some("art_ext".into()),
                ..Default::default()
            }),
            ..Default::default()
        })
        .unwrap();
    let artist_id = songs[0].artists[0].artist_id.clone().unwrap();

    load_sample_extension(state_manager).await;

    main_window
        .global::<ArtistsPageProps>()
        .set_selected_artist(ArtistModel {
            id: artist_id.into(),
            title: "Artist Ext".into(),
            ..Default::default()
        });
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::ArtistContent);

    let initial = wait_until(|| {
        main_window
            .global::<ArtistContentPageProps>()
            .get_extension_providers()
            .row_count()
            == 1
            && main_window
                .global::<ArtistContentPageProps>()
                .get_songs()
                .row_count()
                == 1
    })
    .await;
    assert!(initial);

    main_window
        .global::<AppCallbacks>()
        .invoke_toggle_artist_content_extension("rs.sample".into(), true);

    let ext_loaded = wait_until(|| {
        main_window
            .global::<ArtistContentPageProps>()
            .get_songs()
            .row_count()
            >= 26
    })
    .await;
    assert!(ext_loaded);
    assert!(
        main_window
            .global::<ArtistContentPageProps>()
            .get_songs()
            .row_count()
            >= 26
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_album_content_extension_toggle_and_open(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let database = state_manager.get_database().await;
    let song = create_test_song("alb_ext", "Local Song", "Album Ext", "Local Artist");
    database.insert_songs(vec![song]).unwrap();
    let songs = database
        .get_songs_by_options(GetSongOptions {
            song: Some(SearchableSong {
                id: Some("alb_ext".into()),
                ..Default::default()
            }),
            ..Default::default()
        })
        .unwrap();
    let album_id = songs[0].album.as_ref().unwrap().album_id.clone().unwrap();

    load_sample_extension(state_manager).await;

    main_window
        .global::<AlbumsPageProps>()
        .set_selected_album(AlbumModel {
            id: album_id.into(),
            title: "Album Ext".into(),
            ..Default::default()
        });
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AlbumContent);

    let initial = wait_until(|| {
        main_window
            .global::<AlbumContentPageProps>()
            .get_extension_providers()
            .row_count()
            == 1
            && main_window
                .global::<AlbumContentPageProps>()
                .get_songs()
                .row_count()
                == 1
    })
    .await;
    assert!(initial);

    main_window
        .global::<AppCallbacks>()
        .invoke_toggle_album_content_extension("rs.sample".into(), true);

    let ext_loaded = wait_until(|| {
        main_window
            .global::<AlbumContentPageProps>()
            .get_songs()
            .row_count()
            >= 26
    })
    .await;
    assert!(ext_loaded);
    assert!(
        main_window
            .global::<AlbumContentPageProps>()
            .get_songs()
            .row_count()
            >= 26
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_extension_content_pagination_load_more(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    load_sample_extension(state_manager).await;

    main_window
        .global::<PlaylistsPageProps>()
        .set_selected_playlist(PlaylistModel {
            id: "sample_playlist_1".into(),
            title: "Sample Playlist".into(),
            extension: "rs.sample".into(),
            ..Default::default()
        });
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::PlaylistContent);

    let loaded = wait_until(|| {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count()
            >= 25
    })
    .await;
    assert!(loaded);

    main_window
        .global::<AppCallbacks>()
        .invoke_load_more_playlist_content();

    let paginated = wait_until(|| {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count()
            >= 50
    })
    .await;
    assert!(paginated);
    assert!(
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count()
            >= 50
    );

    let songs = main_window.global::<PlaylistContentPageProps>().get_songs();
    assert_eq!(
        songs.row_data(25).map(|s| s.title),
        Some("Playlist Song 26".into())
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_multiple_extensions_pagination_integration(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    load_custom_extension(state_manager, "rs.sample.1", "rs.sample.1", "Sample 1").await;
    load_custom_extension(state_manager, "rs.sample.2", "rs.sample.2", "Sample 2").await;

    main_window
        .global::<PlaylistsPageProps>()
        .set_selected_playlist(PlaylistModel {
            id: "sample_playlist_multi".into(),
            title: "Multi Extension Playlist".into(),
            ..Default::default()
        });
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::PlaylistContent);

    let _ = wait_until(|| {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_extension_providers()
            .row_count()
            == 2
    })
    .await;

    main_window
        .global::<AppCallbacks>()
        .invoke_toggle_playlist_content_extension("rs.sample.1".into(), true);
    main_window
        .global::<AppCallbacks>()
        .invoke_toggle_playlist_content_extension("rs.sample.2".into(), true);

    let initial_loaded = wait_until(|| {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count()
            >= 50
    })
    .await;
    assert!(initial_loaded);

    main_window
        .global::<AppCallbacks>()
        .invoke_load_more_playlist_content();

    let fully_paginated = wait_until(|| {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count()
            >= 100
    })
    .await;
    assert!(fully_paginated);
    assert!(
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count()
            >= 100
    );
}

integration_test!(
    test_playlists_extension_integration => do_playlists_extension_integration,
    test_extension_content_toggle_provider_off => do_extension_content_toggle_provider_off,
    test_extension_content_toggle_provider_on => do_extension_content_toggle_provider_on,
    test_artist_content_extension_toggle_and_open => do_artist_content_extension_toggle_and_open,
    test_album_content_extension_toggle_and_open => do_album_content_extension_toggle_and_open,
    test_extension_content_pagination_load_more => do_extension_content_pagination_load_more,
    test_multiple_extensions_pagination_integration => do_multiple_extensions_pagination_integration,
    test_extension_preference_text_input => do_extension_preference_text_input,
    test_extension_preference_toggle => do_extension_preference_toggle,
);
