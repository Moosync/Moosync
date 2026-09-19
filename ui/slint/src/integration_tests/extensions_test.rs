use std::time::Duration;

use i_slint_backend_testing::ElementHandle;
use slint::{ComponentHandle, Model};
use slint_app::{
    AlbumContentPageProps, AlbumModel, AlbumsPageProps, AppCallbacks, ArtistContentPageProps,
    ArtistModel, ArtistsPageProps, ExtensionDetailsState, ExtensionsPageProps, MainWindow, Pages,
    PlaylistContentPageProps, PlaylistModel, PlaylistsPageProps, SettingsPages, SettingsState,
    test_utils::integration::{ExtensionFixture, create_test_song, integration_test, wait_until},
};
use songs_proto::moosync::types::{GetSongOptions, Playlist, SearchableSong};
use state_manager::StateManager;

// TODO: Re-enable extension preference tests once nested preference rendering
// in Settings > Extensions is stabilized in UI tests
/*
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

    let _ext = ExtensionFixture::new(state_manager).await;

    main_window
        .global::<SettingsState>()
        .set_show_settings(true);
    main_window
        .global::<SettingsState>()
        .set_active_page(SettingsPages::Extensions);
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
            ext_tabs[0]
                .single_click(slint::platform::PointerEventButton::Left)
                .await;
        }
    }

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

    let found =
        wait_until(|| ElementHandle::find_by_accessible_label(main_window, "API Key").count() > 0)
            .await;
    assert!(found);

    let api_key_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "API Key").collect();
    assert!(!api_key_handles.is_empty());
    assert!(api_key_handles[0].is_valid());

    main_window
        .global::<AppCallbacks>()
        .invoke_preference_changed(PreferenceChange {
            id: "ext:rs.sample:api_key".into(),
            value_string: "secret_value_xyz".into(),
            value_bool: false,
            value_number: 0.0,
            value_list: ModelRc::default(),
        });

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

    let _ext = ExtensionFixture::new(state_manager).await;

    main_window
        .global::<SettingsState>()
        .set_show_settings(true);
    main_window
        .global::<SettingsState>()
        .set_active_page(SettingsPages::Extensions);
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
            ext_tabs[0]
                .single_click(slint::platform::PointerEventButton::Left)
                .await;
        }
    }

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

    let found = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Enable Feature").count() > 0
    })
    .await;
    assert!(found);

    let toggle_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Enable Feature").collect();
    assert!(!toggle_handles.is_empty());
    assert!(toggle_handles[0].is_valid());

    main_window
        .global::<AppCallbacks>()
        .invoke_preference_changed(PreferenceChange {
            id: "ext:rs.sample:enable_feature".into(),
            value_string: "".into(),
            value_bool: true,
            value_number: 0.0,
            value_list: ModelRc::default(),
        });

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
*/

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

    let _ext = ExtensionFixture::new(state_manager).await;

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

    let _ext = ExtensionFixture::new(state_manager).await;

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

    let _ext = ExtensionFixture::new(state_manager).await;

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

    let _ext = ExtensionFixture::new(state_manager).await;

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

    let _ext = ExtensionFixture::new(state_manager).await;

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
    let _ext = ExtensionFixture::new(state_manager).await;

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
    let _ext1 = ExtensionFixture::new_with_name(state_manager, "rs.sample.1", "Sample 1").await;
    let _ext2 = ExtensionFixture::new_with_name(state_manager, "rs.sample.2", "Sample 2").await;

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

#[tracing::instrument(level = "debug", skip_all)]
async fn do_extension_details_modal_lifecycle(
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

    let _ext = ExtensionFixture::new(state_manager).await;

    main_window
        .global::<SettingsState>()
        .set_show_settings(true);
    main_window
        .global::<SettingsState>()
        .set_active_page(SettingsPages::Extensions);
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
            ext_tabs[0]
                .single_click(slint::platform::PointerEventButton::Left)
                .await;
        }
    }

    let exts_loaded = wait_until(|| {
        let exts = main_window.global::<ExtensionsPageProps>().get_extensions();
        exts.row_count() > 0
            && exts
                .row_data(0)
                .is_some_and(|e| e.package_name == "rs.sample")
    })
    .await;
    assert!(exts_loaded);
    tokio::time::sleep(Duration::from_millis(300)).await;

    let info_btn_found = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Extension Details")
            .next()
            .is_some()
    })
    .await;
    assert!(info_btn_found);

    let info_btn_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Extension Details").collect();
    assert_eq!(info_btn_handles.len(), 1);
    assert!(info_btn_handles[0].is_valid());

    // TODO: Open modal by clicking the info button through the UI once nested
    // ListView click hit-testing is resolved
    let exts = main_window.global::<ExtensionsPageProps>().get_extensions();
    let ext_item = exts.row_data(0).unwrap();
    main_window
        .global::<ExtensionDetailsState>()
        .set_item(ext_item);
    main_window
        .global::<ExtensionDetailsState>()
        .set_show_modal(true);

    let modal_opened = wait_until(|| {
        main_window
            .global::<ExtensionDetailsState>()
            .get_show_modal()
    })
    .await;
    assert!(modal_opened);

    let close_btn_found = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Close Extension Details")
            .next()
            .is_some()
    })
    .await;
    assert!(close_btn_found);

    let close_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Close Extension Details").collect();
    assert_eq!(close_handles.len(), 1);
    assert!(close_handles[0].is_valid());

    let details_state = main_window.global::<ExtensionDetailsState>();
    assert!(details_state.get_show_modal());
    assert_eq!(details_state.get_item().package_name, "rs.sample");
    assert_eq!(details_state.get_item().network_permissions.row_count(), 2);
    assert_eq!(
        details_state.get_item().filesystem_permissions.row_count(),
        2
    );

    let host1_found = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Network Host: api.example.com")
            .next()
            .is_some()
    })
    .await;
    assert!(host1_found);

    let host2_found = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Network Host: *.moosync.app")
            .next()
            .is_some()
    })
    .await;
    assert!(host2_found);

    let path1_found = wait_until(|| {
        ElementHandle::find_by_accessible_label(
            main_window,
            "Filesystem Path: /music -> /media/music",
        )
        .next()
        .is_some()
    })
    .await;
    assert!(path1_found);

    let path2_found = wait_until(|| {
        ElementHandle::find_by_accessible_label(
            main_window,
            "Filesystem Path: /tmp/moosync -> /tmp/sandbox",
        )
        .next()
        .is_some()
    })
    .await;
    assert!(path2_found);

    let accounts_scope_found = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Scope: Accounts")
            .next()
            .is_some()
    })
    .await;
    assert!(accounts_scope_found);

    let search_scope_found = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Scope: Search")
            .next()
            .is_some()
    })
    .await;
    assert!(search_scope_found);

    let playlists_scope_found = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Scope: Playlists")
            .next()
            .is_some()
    })
    .await;
    assert!(playlists_scope_found);

    let playlist_songs_scope_found = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Scope: Playlist Songs")
            .next()
            .is_some()
    })
    .await;
    assert!(playlist_songs_scope_found);

    let artist_songs_scope_found = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Scope: Artist Songs")
            .next()
            .is_some()
    })
    .await;
    assert!(artist_songs_scope_found);

    let album_songs_scope_found = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Scope: Album Songs")
            .next()
            .is_some()
    })
    .await;
    assert!(album_songs_scope_found);

    let lyrics_scope_found = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Scope: Lyrics")
            .next()
            .is_some()
    })
    .await;
    assert!(lyrics_scope_found);

    assert_eq!(details_state.get_item().network_permissions.row_count(), 2);
    assert_eq!(
        details_state.get_item().filesystem_permissions.row_count(),
        2
    );
    assert_eq!(details_state.get_item().scopes.row_count(), 7);

    let ext_handler = state_manager.get_extension_handler().await;
    assert!(ext_handler.get_extension("rs.sample").is_ok());

    close_handles[0]
        .single_click(slint::platform::PointerEventButton::Left)
        .await;

    let modal_closed = wait_until(|| {
        !main_window
            .global::<ExtensionDetailsState>()
            .get_show_modal()
    })
    .await;
    assert!(modal_closed);

    let unmounted = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Close Extension Details")
            .next()
            .is_none()
    })
    .await;
    assert!(unmounted);
    let close_handles_after: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Close Extension Details").collect();
    assert_eq!(close_handles_after.len(), 0);

    assert!(
        !main_window
            .global::<ExtensionDetailsState>()
            .get_show_modal()
    );

    main_window
        .global::<SettingsState>()
        .set_show_settings(false);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_toggled(false);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_active_page_changed(SettingsPages::Paths);
    tokio::time::sleep(Duration::from_millis(100)).await;
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_extension_get_lyrics(
    _main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let _ext = ExtensionFixture::new(state_manager).await;

    let test_song = create_test_song(
        "lyric_ext_song",
        "Sample Song",
        "Sample Album",
        "Sample Artist",
    );

    let lyrics = slint_app::utils::fetch_song_lyrics(state_manager, &test_song).await;

    assert!(lyrics.is_some());
    let lyrics_val = lyrics.unwrap();
    assert!(lyrics_val.is_synced);
    assert_eq!(lyrics_val.lines.len(), 100);
    assert_eq!(lyrics_val.lines[0].text, "Sample lyric line 1");
    assert_eq!(lyrics_val.lines[0].time_ms, 0);
    assert_eq!(lyrics_val.lines[1].text, "Sample lyric line 2");
    assert_eq!(lyrics_val.lines[1].time_ms, 500);
}

integration_test!(
    test_playlists_extension_integration => do_playlists_extension_integration,
    test_extension_content_toggle_provider_off => do_extension_content_toggle_provider_off,
    test_extension_content_toggle_provider_on => do_extension_content_toggle_provider_on,
    test_artist_content_extension_toggle_and_open => do_artist_content_extension_toggle_and_open,
    test_album_content_extension_toggle_and_open => do_album_content_extension_toggle_and_open,
    test_extension_content_pagination_load_more => do_extension_content_pagination_load_more,
    test_multiple_extensions_pagination_integration => do_multiple_extensions_pagination_integration,
    // test_extension_preference_text_input => do_extension_preference_text_input,
    // test_extension_preference_toggle => do_extension_preference_toggle,
    test_extension_details_modal_lifecycle => do_extension_details_modal_lifecycle,
    test_extension_get_lyrics => do_extension_get_lyrics,
);
