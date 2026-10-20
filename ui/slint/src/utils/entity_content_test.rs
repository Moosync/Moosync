use rstest::rstest;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use songs_proto::moosync::types::{InnerSong, Song};
use state_manager::StateManager;
use tracing_test::traced_test;

use crate::{
    AlbumContentPageProps, ExtensionProviderItem, MainWindow, SongModel,
    error::UiError,
    main_content::album_content::AlbumSongProvider,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
    utils::{
        EntityContentCoordinator, EntitySongProvider, ExtensionPaginationManager, IntoVec,
        fetch_scope_providers, map_songs_to_models, update_provider_list_enabled,
    },
};

struct MockEntitySongProvider;

#[async_trait::async_trait]
impl EntitySongProvider for MockEntitySongProvider {
    type Entity = String;

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_entity(_main_window: &MainWindow) -> (Self::Entity, String) {
        ("test_entity_id".to_string(), String::new())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_songs(main_window: &MainWindow) -> Vec<SongModel> {
        AlbumSongProvider::get_songs(main_window)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_songs(main_window: &MainWindow, model: ModelRc<SongModel>) {
        AlbumSongProvider::set_songs(main_window, model);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_extensions(main_window: &MainWindow) -> ModelRc<ExtensionProviderItem> {
        AlbumSongProvider::get_extensions(main_window)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_extensions(main_window: &MainWindow, extensions: ModelRc<ExtensionProviderItem>) {
        AlbumSongProvider::set_extensions(main_window, extensions);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_local_songs(
        _state_manager: &StateManager,
        _entity: Self::Entity,
    ) -> Result<Vec<Song>, UiError> {
        Ok(vec![
            Song {
                song: Some(InnerSong {
                    id: Some("s1".to_string()),
                    title: Some("Song One".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            Song {
                song: Some(InnerSong {
                    id: Some("s2".to_string()),
                    title: Some("Song Two".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        ])
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_extension_songs(
        _state_manager: &StateManager,
        _entity: Self::Entity,
        extension: String,
        _page_token: Option<String>,
    ) -> Result<(Vec<Song>, Option<String>), UiError> {
        Ok((
            vec![Song {
                song: Some(InnerSong {
                    id: Some(format!("{}_ext_s", extension)),
                    title: Some("Extension Song".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            None,
        ))
    }
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_pagination_manager_register() {
    let mut manager = ExtensionPaginationManager::default();

    manager.register_extension("test.ext");
    let initial_load = manager.load_more();

    assert!(initial_load.is_empty());
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_pagination_manager_update_after_fetch_more() {
    let mut manager = ExtensionPaginationManager::default();
    manager.register_extension("test.ext");

    manager.update_after_fetch("test.ext", Some("token_1".to_string()));
    let next_load = manager.load_more();

    assert_eq!(next_load.len(), 1);
    assert_eq!(next_load[0].0, "test.ext");
    assert_eq!(next_load[0].1.as_deref(), Some("token_1"));
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_pagination_manager_load_more_while_loading() {
    let mut manager = ExtensionPaginationManager::default();
    manager.register_extension("test.ext");
    manager.update_after_fetch("test.ext", Some("token_1".to_string()));
    let _ = manager.load_more();

    let while_loading = manager.load_more();

    assert!(while_loading.is_empty());
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_pagination_manager_update_after_fetch_done() {
    let mut manager = ExtensionPaginationManager::default();
    manager.register_extension("test.ext");

    manager.update_after_fetch("test.ext", None);
    let done_load = manager.load_more();

    assert!(done_load.is_empty());
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_pagination_manager_mark_error() {
    let mut manager = ExtensionPaginationManager::default();
    manager.register_extension("test.ext");
    manager.update_after_fetch("test.ext", Some("retry_token".to_string()));
    let _ = manager.load_more();

    manager.mark_error("test.ext");
    let retry_load = manager.load_more();

    assert_eq!(retry_load.len(), 1);
    assert_eq!(retry_load[0].1.as_deref(), Some("retry_token"));
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_pagination_manager_remove_extension() {
    let mut manager = ExtensionPaginationManager::default();
    manager.register_extension("ext1");
    manager.register_extension("ext2");
    manager.update_after_fetch("ext1", Some("t1".to_string()));
    manager.update_after_fetch("ext2", Some("t2".to_string()));

    manager.remove_extension("ext1");
    let load = manager.load_more();

    assert_eq!(load.len(), 1);
    assert_eq!(load[0].0, "ext2");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_pagination_manager_reset() {
    let mut manager = ExtensionPaginationManager::default();
    manager.register_extension("ext1");
    manager.update_after_fetch("ext1", Some("t1".to_string()));

    manager.reset();
    let load = manager.load_more();

    assert!(load.is_empty());
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_update_provider_list_enabled() {
    let providers = vec![
        ExtensionProviderItem {
            id: "ext1".into(),
            name: "Extension One".into(),
            enabled: false,
        },
        ExtensionProviderItem {
            id: "ext2".into(),
            name: "Extension Two".into(),
            enabled: false,
        },
    ];

    let updated = update_provider_list_enabled(&providers, "ext1", true);

    assert_eq!(updated.len(), 2);
    assert!(updated[0].enabled);
    assert!(!updated[1].enabled);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_entity_content_coordinator_on_hide(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));
    let dummy_songs = vec![SongModel::default(), SongModel::default()];
    main_window
        .global::<AlbumContentPageProps>()
        .set_songs(ModelRc::new(VecModel::from(dummy_songs)));
    let coordinator =
        EntityContentCoordinator::<MockEntitySongProvider>::new(main_window, state_manager);

    coordinator.on_hide();

    assert_eq!(
        main_window
            .global::<AlbumContentPageProps>()
            .get_songs()
            .row_count(),
        0
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_entity_content_coordinator_clone(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));
    let coordinator =
        EntityContentCoordinator::<MockEntitySongProvider>::new(main_window, state_manager);

    let cloned = coordinator.clone();
    cloned.on_hide();

    assert_eq!(
        main_window
            .global::<AlbumContentPageProps>()
            .get_songs()
            .row_count(),
        0
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_entity_content_coordinator_toggle_extension_disabled(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));

    let song1 = SongModel {
        id: "s1".into(),
        extension: "ext_a".into(),
        ..Default::default()
    };
    let song2 = SongModel {
        id: "s2".into(),
        extension: "ext_b".into(),
        ..Default::default()
    };
    main_window
        .global::<AlbumContentPageProps>()
        .set_songs(ModelRc::new(VecModel::from(vec![song1, song2])));

    EntityContentCoordinator::<MockEntitySongProvider>::disable_extension_in_ui(
        main_window,
        state_manager,
        "ext_a",
    );

    let songs = main_window.global::<AlbumContentPageProps>().get_songs();
    assert_eq!(songs.row_count(), 1);
    assert_eq!(songs.row_data(0).unwrap().id.as_str(), "s2");
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_entity_content_coordinator_load_more_empty(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));
    let coordinator =
        EntityContentCoordinator::<MockEntitySongProvider>::new(main_window, state_manager);

    main_window
        .global::<AlbumContentPageProps>()
        .set_songs(ModelRc::default());

    coordinator.on_load_more_songs();

    assert_eq!(
        main_window
            .global::<AlbumContentPageProps>()
            .get_songs()
            .row_count(),
        0
    );
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_entity_content_map_songs_to_models() {
    let songs = vec![Song {
        song: Some(InnerSong {
            id: Some("mapped_s1".to_string()),
            title: Some("Mapped Song".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }];

    let models = map_songs_to_models(songs, None);

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id.as_str(), "mapped_s1");
    assert_eq!(models[0].title.as_str(), "Mapped Song");
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_entity_content_fetch_scope_providers_none(state_manager_fixture: TestSlintSmContext) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let handler = sm.get_extension_handler();

    let (providers, detail) = fetch_scope_providers(&handler, None, "").await;

    assert!(providers.is_empty());
    assert!(detail.is_none());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_entity_song_provider_update_extensions_enabled(main_window: MainWindow) {
    let provider = ExtensionProviderItem {
        id: "ext1".into(),
        name: "Extension 1".into(),
        enabled: false,
    };
    main_window
        .global::<AlbumContentPageProps>()
        .set_extension_providers(ModelRc::new(VecModel::from(vec![provider])));

    MockEntitySongProvider::update_extensions_enabled(&main_window, "ext1", true);

    let providers = main_window
        .global::<AlbumContentPageProps>()
        .get_extension_providers();
    assert_eq!(providers.row_count(), 1);
    assert!(providers.row_data(0).unwrap().enabled);
}
