use std::time::Duration;

use i_slint_backend_testing::ElementHandle;
use slint::{ComponentHandle, Model};
use slint_app::{
    AllSongsPageProps, AppCallbacks, BottomBarCallbacks, MainWindow, Pages, PlayerProps, SongModel,
    test_utils::integration::{click_element, create_test_song, integration_test, wait_until},
};
use state_manager::StateManager;

#[tracing::instrument(level = "debug", skip_all)]
async fn do_playback_play_song(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let database = state_manager.get_database().await;
    let song = create_test_song("s_play", "Playback Track", "Album", "Artist");
    database.insert_songs(vec![song.clone()]).unwrap();

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AllSongs);
    let _ = wait_until(|| {
        main_window
            .global::<AllSongsPageProps>()
            .get_songs()
            .row_count()
            > 0
    })
    .await;

    main_window
        .global::<AppCallbacks>()
        .invoke_play_song(SongModel::from(song));

    let playing = wait_until(|| main_window.global::<PlayerProps>().get_playing()).await;
    assert!(playing);
    assert_eq!(
        main_window.global::<PlayerProps>().get_current_song().title,
        "Playback Track"
    );

    let song_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Playback Track").collect();
    assert_eq!(song_handles.len(), 1);
    assert!(song_handles[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_playback_pause_song(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let database = state_manager.get_database().await;
    let song = create_test_song("s_pause", "Playback Track", "Album", "Artist");
    database.insert_songs(vec![song.clone()]).unwrap();

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AllSongs);
    let _ = wait_until(|| {
        main_window
            .global::<AllSongsPageProps>()
            .get_songs()
            .row_count()
            > 0
    })
    .await;

    main_window
        .global::<AppCallbacks>()
        .invoke_play_song(SongModel::from(song));
    let _ = wait_until(|| main_window.global::<PlayerProps>().get_playing()).await;

    let pause_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Pause").collect();
    assert_eq!(pause_handles.len(), 1);
    assert!(pause_handles[0].is_valid());
    click_element(&pause_handles[0]).await;

    let paused = wait_until(|| !main_window.global::<PlayerProps>().get_playing()).await;
    assert!(paused);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_playback_resume_song(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let database = state_manager.get_database().await;
    let song = create_test_song("s_resume", "Playback Track", "Album", "Artist");
    database.insert_songs(vec![song.clone()]).unwrap();

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AllSongs);
    let _ = wait_until(|| {
        main_window
            .global::<AllSongsPageProps>()
            .get_songs()
            .row_count()
            > 0
    })
    .await;

    main_window
        .global::<AppCallbacks>()
        .invoke_play_song(SongModel::from(song));
    let _ = wait_until(|| main_window.global::<PlayerProps>().get_playing()).await;

    main_window
        .global::<BottomBarCallbacks>()
        .invoke_play_pause_clicked();
    let _ = wait_until(|| !main_window.global::<PlayerProps>().get_playing()).await;

    let play_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Play").collect();
    assert_eq!(play_handles.len(), 1);
    assert!(play_handles[0].is_valid());
    click_element(&play_handles[0]).await;

    let resumed = wait_until(|| main_window.global::<PlayerProps>().get_playing()).await;
    assert!(resumed);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_playback_skip_next(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let database = state_manager.get_database().await;
    let song1 = create_test_song("s_next1", "Track 1", "Album", "Artist");
    let song2 = create_test_song("s_next2", "Track 2", "Album", "Artist");
    database
        .insert_songs(vec![song1.clone(), song2.clone()])
        .unwrap();

    {
        let mut ph = state_manager.get_player_handler_mut().await;
        ph.play_now(vec![song1.clone(), song2.clone()]);
    }

    let _ =
        wait_until(|| main_window.global::<PlayerProps>().get_current_song().title == "Track 1")
            .await;

    let next_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Next").collect();
    assert_eq!(next_handles.len(), 1);
    assert!(next_handles[0].is_valid());
    click_element(&next_handles[0]).await;

    let skipped =
        wait_until(|| main_window.global::<PlayerProps>().get_current_song().title == "Track 2")
            .await;
    assert!(skipped);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_playback_skip_previous(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let database = state_manager.get_database().await;
    let song1 = create_test_song("s_prev1", "Track 1", "Album", "Artist");
    let song2 = create_test_song("s_prev2", "Track 2", "Album", "Artist");
    database
        .insert_songs(vec![song1.clone(), song2.clone()])
        .unwrap();

    {
        let mut ph = state_manager.get_player_handler_mut().await;
        ph.play_now(vec![song1.clone(), song2.clone()]);
        ph.next();
    }

    let _ =
        wait_until(|| main_window.global::<PlayerProps>().get_current_song().title == "Track 2")
            .await;

    let prev_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Previous").collect();
    assert_eq!(prev_handles.len(), 1);
    assert!(prev_handles[0].is_valid());
    click_element(&prev_handles[0]).await;

    let skipped =
        wait_until(|| main_window.global::<PlayerProps>().get_current_song().title == "Track 1")
            .await;
    assert!(skipped);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_playback_toggle_shuffle(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    let shuffle_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Shuffle").collect();
    assert_eq!(shuffle_handles.len(), 1);
    assert!(shuffle_handles[0].is_valid());
    click_element(&shuffle_handles[0]).await;
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_playback_toggle_repeat(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    let initial_mode = main_window.global::<PlayerProps>().get_repeat_mode();

    let repeat_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Repeat").collect();
    assert_eq!(repeat_handles.len(), 1);
    assert!(repeat_handles[0].is_valid());
    click_element(&repeat_handles[0]).await;

    let toggled =
        wait_until(|| main_window.global::<PlayerProps>().get_repeat_mode() != initial_mode).await;
    assert!(toggled);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_playback_volume_and_mute(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let _initial_volume = {
        let player_handler = state_manager.get_player_handler().await;
        player_handler.get_volume()
    };

    main_window
        .global::<BottomBarCallbacks>()
        .invoke_set_volume(42);

    let start = std::time::Instant::now();
    let mut changed = false;
    while start.elapsed() < Duration::from_secs(5) {
        let ph = state_manager.get_player_handler().await;
        if ph.get_volume() == 42 {
            changed = true;
            break;
        }
        drop(ph);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(changed);

    let mute_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Mute").collect();
    assert_eq!(mute_handles.len(), 1);
    assert!(mute_handles[0].is_valid());
    click_element(&mute_handles[0]).await;

    let start = std::time::Instant::now();
    let mut muted = false;
    while start.elapsed() < Duration::from_secs(5) {
        let ph = state_manager.get_player_handler().await;
        if ph.get_volume() == 0 {
            muted = true;
            break;
        }
        drop(ph);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(muted);

    let unmute_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Unmute").collect();
    assert_eq!(unmute_handles.len(), 1);
    assert!(unmute_handles[0].is_valid());
}

integration_test!(
    test_playback_play_song => do_playback_play_song,
    test_playback_pause_song => do_playback_pause_song,
    test_playback_resume_song => do_playback_resume_song,
    test_playback_skip_next => do_playback_skip_next,
    test_playback_skip_previous => do_playback_skip_previous,
    test_playback_toggle_shuffle => do_playback_toggle_shuffle,
    test_playback_toggle_repeat => do_playback_toggle_repeat,
    test_playback_volume_and_mute => do_playback_volume_and_mute,
);
