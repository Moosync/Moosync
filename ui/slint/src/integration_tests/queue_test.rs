use i_slint_backend_testing::ElementHandle;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use slint_app::{
    AppCallbacks, MainWindow, QueuePageProps, SavePlaylistState, SongModel,
    test_utils::integration::{click_element, create_test_song, integration_test, wait_until},
};
use state_manager::StateManager;

#[tracing::instrument(level = "debug", skip_all)]
async fn do_queue_toggle_overlay(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    let found =
        wait_until(|| ElementHandle::find_by_accessible_label(main_window, "Queue").count() > 0)
            .await;
    assert!(found);

    let queue_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Queue").collect();
    assert!(!queue_handles.is_empty());
    assert!(queue_handles[0].is_valid());
    click_element(&queue_handles[0]).await;

    let opened = wait_until(|| main_window.global::<QueuePageProps>().get_show_queue()).await;
    assert!(opened);

    click_element(&queue_handles[0]).await;
    let closed = wait_until(|| !main_window.global::<QueuePageProps>().get_show_queue()).await;
    assert!(closed);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_queue_display_items(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let s1 = create_test_song("q1", "Queue Track 1", "Album Q", "Artist Q");
    let s2 = create_test_song("q2", "Queue Track 2", "Album Q", "Artist Q");

    {
        let mut ph = state_manager.get_player_handler_mut().await;
        ph.add_to_queue(vec![s1.clone(), s2.clone()]);
    }

    main_window
        .global::<QueuePageProps>()
        .set_queue(ModelRc::new(VecModel::from(vec![
            SongModel::from(s1),
            SongModel::from(s2),
        ])));

    main_window.global::<QueuePageProps>().set_show_queue(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_queue_toggled(true);

    let loaded = wait_until(|| {
        let q = main_window.global::<QueuePageProps>().get_queue();
        q.row_count() == 2
            && q.row_data(0).is_some_and(|s| s.title == "Queue Track 1")
            && q.row_data(1).is_some_and(|s| s.title == "Queue Track 2")
    })
    .await;
    assert!(loaded);

    let found1 = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Queue Track 1").count() > 0
    })
    .await;
    assert!(found1);

    let handles1: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Queue Track 1").collect();
    assert_eq!(handles1.len(), 1);
    assert!(handles1[0].is_valid());

    let found2 = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Queue Track 2").count() > 0
    })
    .await;
    assert!(found2);

    let handles2: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Queue Track 2").collect();
    assert_eq!(handles2.len(), 1);
    assert!(handles2[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_queue_remove_item(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let s1 = create_test_song("qr1", "Remove Track 1", "Album", "Artist");
    let s2 = create_test_song("qr2", "Remove Track 2", "Album", "Artist");

    {
        let mut ph = state_manager.get_player_handler_mut().await;
        ph.add_to_queue(vec![s1.clone(), s2.clone()]);
    }

    main_window
        .global::<QueuePageProps>()
        .set_queue(ModelRc::new(VecModel::from(vec![
            SongModel::from(s1),
            SongModel::from(s2.clone()),
        ])));
    main_window.global::<QueuePageProps>().set_show_queue(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_queue_toggled(true);

    let _ = wait_until(|| {
        main_window
            .global::<QueuePageProps>()
            .get_queue()
            .row_count()
            == 2
    })
    .await;

    main_window
        .global::<AppCallbacks>()
        .invoke_remove_from_queue(0);

    main_window
        .global::<QueuePageProps>()
        .set_queue(ModelRc::new(VecModel::from(vec![SongModel::from(s2)])));

    let removed = wait_until(|| {
        let q = main_window.global::<QueuePageProps>().get_queue();
        q.row_count() == 1 && q.row_data(0).is_some_and(|s| s.title == "Remove Track 2")
    })
    .await;
    assert!(removed);

    let not_found = wait_until(|| {
        !ElementHandle::find_by_accessible_label(main_window, "Remove Track 1")
            .any(|h| h.is_valid())
    })
    .await;
    assert!(not_found);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_queue_clear(main_window: &'static MainWindow, state_manager: &'static StateManager) {
    let s1 = create_test_song("qc1", "Clear Track 1", "Album", "Artist");

    {
        let mut ph = state_manager.get_player_handler_mut().await;
        ph.add_to_queue(vec![s1.clone()]);
    }

    main_window
        .global::<QueuePageProps>()
        .set_queue(ModelRc::new(VecModel::from(vec![SongModel::from(s1)])));
    main_window.global::<QueuePageProps>().set_show_queue(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_queue_toggled(true);

    let _ = wait_until(|| {
        main_window
            .global::<QueuePageProps>()
            .get_queue()
            .row_count()
            == 1
    })
    .await;

    let found_clear =
        wait_until(|| ElementHandle::find_by_accessible_label(main_window, "Clear").count() > 0)
            .await;
    assert!(found_clear);

    let clear_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Clear").collect();
    assert_eq!(clear_handles.len(), 1);
    assert!(clear_handles[0].is_valid());
    click_element(&clear_handles[0]).await;

    let cleared = wait_until(|| {
        main_window
            .global::<QueuePageProps>()
            .get_queue()
            .row_count()
            == 0
    })
    .await;
    assert!(cleared);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_queue_save_as_playlist_modal_lifecycle(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let s = create_test_song("qs", "Queue Song", "Album", "Artist");
    let mut ph = state_manager.get_player_handler_mut().await;
    ph.add_to_queue(vec![s.clone()]);
    drop(ph);

    main_window
        .global::<QueuePageProps>()
        .set_queue(ModelRc::new(VecModel::from(vec![SongModel::from(s)])));
    main_window.global::<QueuePageProps>().set_show_queue(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_queue_toggled(true);

    let _ = wait_until(|| main_window.global::<QueuePageProps>().get_show_queue()).await;

    let found_save = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Save as playlist").count() > 0
    })
    .await;
    assert!(found_save);

    let save_btn_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Save as playlist").collect();
    assert_eq!(save_btn_handles.len(), 1);
    assert!(save_btn_handles[0].is_valid());
    click_element(&save_btn_handles[0]).await;

    let modal_opened =
        wait_until(|| main_window.global::<SavePlaylistState>().get_show_modal()).await;
    assert!(modal_opened);

    let found_save_modal =
        wait_until(|| ElementHandle::find_by_accessible_label(main_window, "Save").count() > 0)
            .await;
    assert!(found_save_modal);

    let save_modal_btn: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Save").collect();
    assert_eq!(save_modal_btn.len(), 1);
    assert!(save_modal_btn[0].is_valid());

    main_window
        .global::<SavePlaylistState>()
        .set_show_modal(false);

    let modal_closed =
        wait_until(|| !main_window.global::<SavePlaylistState>().get_show_modal()).await;
    assert!(modal_closed);
}

integration_test!(
    test_queue_toggle_overlay => do_queue_toggle_overlay,
    test_queue_display_items => do_queue_display_items,
    test_queue_remove_item => do_queue_remove_item,
    test_queue_clear => do_queue_clear,
    test_queue_save_as_playlist_modal_lifecycle => do_queue_save_as_playlist_modal_lifecycle,
);
