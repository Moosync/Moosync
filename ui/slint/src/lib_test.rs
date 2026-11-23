use extensions_proto::moosync::types::player_event::Event as PlayerEvent;
use rstest::rstest;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use songs_proto::{
    duration_proto::google::protobuf::Duration as ProtoDuration,
    moosync::types::{InnerSong, Song},
};
use tracing_test::traced_test;

use crate::{
    AllSongsPageProps, AppCallbacks, AppProps, BottomBarCallbacks, CoverHelper,
    ExtensionProviderItem, MainWindow, Pages, PlayerProps, QueuePageProps, SettingsPages,
    SongDetailAction, SongModel, SongSortCriterion, TopBarProps, UtilCallbacks,
    apply_updated_song_to_ui,
    pages::PageLifecycleManager,
    setup_ui,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
    update_player_event_props,
    utils::lazy_model::LazySongVecModel,
};

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_ui_get_all_pages_and_setup(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window: MainWindow = main_window;
    let state_manager = sm;

    let main_pages = PageLifecycleManager::get_main_pages(&main_window, &state_manager);
    for (_, page) in main_pages.iter() {
        page.initialize();
        page.on_show();
        page.on_hide();
    }
    let settings_pages = PageLifecycleManager::get_settings_pages(&main_window, &state_manager);
    for (_, page) in settings_pages.iter() {
        page.initialize();
        page.on_show();
        page.on_hide();
    }
    let queue_page = PageLifecycleManager::get_queue_page(&main_window, &state_manager);
    queue_page.initialize();
    queue_page.on_show();
    queue_page.on_hide();

    setup_ui(&main_window, &state_manager);

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Albums);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Artists);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Playlists);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Genres);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Explore);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Search);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::PlaylistContent);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AlbumContent);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::ArtistContent);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::GenreContent);

    assert!(main_window.global::<TopBarProps>().get_can_go_back());
    assert!(!main_window.global::<TopBarProps>().get_can_go_forward());

    main_window.global::<AppCallbacks>().invoke_navigate_back();

    assert_eq!(
        main_window.global::<AppProps>().get_active_page(),
        Pages::ArtistContent
    );
    assert!(main_window.global::<TopBarProps>().get_can_go_back());
    assert!(main_window.global::<TopBarProps>().get_can_go_forward());

    main_window
        .global::<AppCallbacks>()
        .invoke_navigate_forward();

    assert_eq!(
        main_window.global::<AppProps>().get_active_page(),
        Pages::GenreContent
    );
    assert!(main_window.global::<TopBarProps>().get_can_go_back());
    assert!(!main_window.global::<TopBarProps>().get_can_go_forward());

    main_window
        .global::<AppCallbacks>()
        .invoke_settings_toggled(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_active_page_changed(SettingsPages::Paths);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_active_page_changed(SettingsPages::System);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_active_page_changed(SettingsPages::Extensions);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_active_page_changed(SettingsPages::Themes);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_active_page_changed(SettingsPages::Cache);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_toggled(false);

    main_window
        .global::<AppCallbacks>()
        .invoke_queue_toggled(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_queue_toggled(false);

    let song_model = SongModel::from(Song::default());
    main_window
        .global::<AppCallbacks>()
        .invoke_play_song(song_model.clone());
    main_window
        .global::<AppCallbacks>()
        .invoke_add_song_to_queue(song_model.clone());
    main_window
        .global::<AppCallbacks>()
        .invoke_song_detail_action(
            SongDetailAction::Play,
            ModelRc::new(VecModel::from(vec![song_model.clone()])),
        );
    main_window
        .global::<AppCallbacks>()
        .invoke_song_detail_action(
            SongDetailAction::AddToQueue,
            ModelRc::new(VecModel::from(vec![song_model.clone()])),
        );

    main_window
        .global::<BottomBarCallbacks>()
        .invoke_play_pause_clicked();
    main_window
        .global::<BottomBarCallbacks>()
        .invoke_toggle_repeat();
    main_window
        .global::<BottomBarCallbacks>()
        .invoke_next_song();
    main_window
        .global::<BottomBarCallbacks>()
        .invoke_prev_song();
    main_window
        .global::<BottomBarCallbacks>()
        .invoke_set_volume(75);
    main_window.global::<BottomBarCallbacks>().invoke_shuffle();
    main_window.global::<BottomBarCallbacks>().invoke_seek(30);

    let _ = main_window
        .global::<CoverHelper>()
        .invoke_fetch_cover_high(song_model.clone());
    let _ = main_window
        .global::<CoverHelper>()
        .invoke_fetch_cover_low(song_model.clone());

    let _ = main_window
        .global::<AppCallbacks>()
        .invoke_filter_and_sort_songs(
            ModelRc::new(VecModel::from(vec![song_model.clone()])),
            "test".into(),
            SongSortCriterion::Title,
            true,
        );

    assert_eq!(main_pages.len(), 11);
    assert_eq!(settings_pages.len(), 5);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_update_player_event_props(main_window: MainWindow) {
    let player_props = main_window.global::<PlayerProps>();

    update_player_event_props(&player_props, &PlayerEvent::Play(Default::default()));
    assert!(player_props.get_playing());

    update_player_event_props(&player_props, &PlayerEvent::Pause(Default::default()));
    assert!(!player_props.get_playing());

    update_player_event_props(
        &player_props,
        &PlayerEvent::TimeUpdate(ProtoDuration {
            seconds: 125,
            nanos: 0,
        }),
    );
    assert_eq!(player_props.get_current_duration_ms(), 125_000);
    assert_eq!(player_props.get_current_pos_str().as_str(), "02:05");
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_util_callbacks_selection_and_validation(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window: MainWindow = main_window;
    let state_manager = sm;
    setup_ui(&main_window, &state_manager);

    let util_callbacks = main_window.global::<UtilCallbacks>();

    let indices = ModelRc::new(VecModel::from(vec![0, 2]));
    assert!(util_callbacks.invoke_is_index_selected(indices.clone(), 0));
    assert!(!util_callbacks.invoke_is_index_selected(indices.clone(), 1));
    assert!(util_callbacks.invoke_is_index_selected(indices.clone(), 2));

    let song1 = SongModel {
        id: "s1".into(),
        title: "Song 1".into(),
        ..Default::default()
    };
    let song2 = SongModel {
        id: "s2".into(),
        title: "Song 2".into(),
        ..Default::default()
    };
    let song3 = SongModel {
        id: "s3".into(),
        title: "Song 3".into(),
        ..Default::default()
    };
    let display_songs = ModelRc::new(VecModel::from(vec![song1, song2, song3]));

    let selected = util_callbacks.invoke_get_selected_songs(display_songs.clone(), indices);
    assert_eq!(selected.row_count(), 2);
    assert_eq!(selected.row_data(0).unwrap().id.as_str(), "s1");
    assert_eq!(selected.row_data(1).unwrap().id.as_str(), "s3");

    let empty_indices = ModelRc::default();
    let empty = util_callbacks.invoke_get_selected_songs(display_songs, empty_indices);
    assert_eq!(empty.row_count(), 0);

    let is_valid = util_callbacks.invoke_validate_input("123".into(), "^[0-9]+$".into());
    assert!(is_valid);

    let provider = ExtensionProviderItem {
        id: "ext1".into(),
        name: "Ext 1".into(),
        enabled: true,
    };
    let providers = ModelRc::new(VecModel::from(vec![provider]));
    assert!(util_callbacks.invoke_has_active_extension_providers(providers));

    let no_providers = ModelRc::default();
    assert!(!util_callbacks.invoke_has_active_extension_providers(no_providers));
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_apply_updated_song_to_ui(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager = sm;
    let cache_dir = state_manager.get_cache_dir();
    let initial_song = SongModel {
        id: "song-1".into(),
        title: "Old Title".into(),
        song_cover_path_low: "".into(),
        ..Default::default()
    };
    main_window
        .global::<PlayerProps>()
        .set_current_song(initial_song.clone());
    main_window
        .global::<AllSongsPageProps>()
        .set_songs(ModelRc::new(LazySongVecModel::new(
            vec![initial_song.clone()],
            40,
            40,
            cache_dir.clone(),
            false,
        )));
    main_window
        .global::<QueuePageProps>()
        .set_queue(ModelRc::new(LazySongVecModel::new(
            vec![initial_song],
            40,
            40,
            cache_dir,
            false,
        )));
    let updated = Song {
        song: Some(InnerSong {
            id: Some("song-1".to_string()),
            title: Some("New Title".to_string()),
            song_cover_path_low: Some("/new/cover.webp".to_string()),
            song_cover_path_high: Some("/new/cover_high.webp".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    apply_updated_song_to_ui(&main_window, &state_manager, SongModel::from(updated));

    let current_song = main_window.global::<PlayerProps>().get_current_song();
    assert_eq!(current_song.title.as_str(), "New Title");
    assert_eq!(current_song.song_cover_path_low.as_str(), "/new/cover.webp");
    let all_songs = main_window.global::<AllSongsPageProps>().get_songs();
    assert_eq!(all_songs.row_data(0).unwrap().title.as_str(), "New Title");
    let queue_songs = main_window.global::<QueuePageProps>().get_queue();
    assert_eq!(queue_songs.row_data(0).unwrap().title.as_str(), "New Title");
}
