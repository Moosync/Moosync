use std::future::Future;

use extensions_proto::moosync::types::{ExtensionDetail, ExtensionProviderScope};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use songs_proto::moosync::types::{InnerSong, Song};
use state_manager::StateManager;
use tracing_test::traced_test;

use crate::{
    AlbumContentPageProps, ExtensionProviderItem, MainWindow, SongModel,
    error::UiError,
    test_utils::{TestSlintSmContext, run_slint_test, state_manager_fixture, wait_until},
    utils::{
        EntityContentCoordinator, EntitySongProvider, ExtensionPaginationManager, IntoVec,
        fetch_scope_providers, map_songs_to_models, update_provider_list_enabled,
    },
};

struct MockEntitySongProvider;

impl EntitySongProvider for MockEntitySongProvider {
    type Entity = String;

    #[tracing::instrument(level = "debug", skip_all)]
    fn extension_scope() -> Option<ExtensionProviderScope> { None }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_entity(_main_window: &MainWindow) -> (Self::Entity, String) {
        ("test_entity_id".to_string(), String::new())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_songs(main_window: &MainWindow) -> Vec<SongModel> {
        main_window
            .global::<AlbumContentPageProps>()
            .get_songs()
            .into_vec()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_songs(main_window: &MainWindow, model: ModelRc<SongModel>) {
        main_window
            .global::<AlbumContentPageProps>()
            .set_songs(model);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn update_extensions_enabled(main_window: &MainWindow, package_name: &str, enabled: bool) {
        let props = main_window.global::<AlbumContentPageProps>();
        let providers = props.get_extension_providers().into_vec();
        let updated = update_provider_list_enabled(&providers, package_name, enabled);
        props.set_extension_providers(ModelRc::new(VecModel::from(updated)));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_extensions(main_window: &MainWindow, extensions: ModelRc<ExtensionProviderItem>) {
        main_window
            .global::<AlbumContentPageProps>()
            .set_extension_providers(extensions);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn clear_ui(main_window: &MainWindow) {
        let props = main_window.global::<AlbumContentPageProps>();
        props.set_songs(ModelRc::default());
        props.set_extension_providers(ModelRc::default());
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn fetch_local_songs(
        _state_manager: &StateManager,
        _entity: Self::Entity,
    ) -> impl Future<Output = Result<Vec<Song>, UiError>> + Send {
        async {
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
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn fetch_extension_songs(
        _state_manager: &StateManager,
        _entity: Self::Entity,
        extension: String,
        _page_token: Option<String>,
    ) -> impl Future<Output = Result<(Vec<Song>, Option<String>), UiError>> + Send {
        async move {
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
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_pagination_manager_register_and_fetch() {
    let mut manager = ExtensionPaginationManager::default();

    manager.register_extension("test.ext");
    let initial_load = manager.load_more();
    assert!(initial_load.is_empty());

    manager.update_after_fetch("test.ext", Some("token_1".to_string()));
    let next_load = manager.load_more();
    assert_eq!(next_load.len(), 1);
    assert_eq!(next_load[0].0, "test.ext");
    assert_eq!(next_load[0].1.as_deref(), Some("token_1"));

    let while_loading = manager.load_more();
    assert!(while_loading.is_empty());

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
    let load = manager.load_more();
    assert_eq!(load.len(), 1);

    manager.mark_error("test.ext");
    let retry_load = manager.load_more();
    assert_eq!(retry_load.len(), 1);
    assert_eq!(retry_load[0].1.as_deref(), Some("retry_token"));
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_pagination_manager_remove_and_reset() {
    let mut manager = ExtensionPaginationManager::default();

    manager.register_extension("ext1");
    manager.register_extension("ext2");
    manager.update_after_fetch("ext1", Some("t1".to_string()));
    manager.update_after_fetch("ext2", Some("t2".to_string()));

    manager.remove_extension("ext1");
    let load_after_remove = manager.load_more();
    assert_eq!(load_after_remove.len(), 1);
    assert_eq!(load_after_remove[0].0, "ext2");

    manager.reset();
    let load_after_reset = manager.load_more();
    assert!(load_after_reset.is_empty());
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

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_map_songs_to_models() {
    let songs = vec![
        Song {
            song: Some(InnerSong {
                id: Some("s1".to_string()),
                title: Some("Song 1".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        },
        Song {
            song: Some(InnerSong {
                id: Some("s2".to_string()),
                title: Some("Song 2".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        },
    ];

    let detail = ExtensionDetail {
        package_name: "test.pkg".to_string(),
        name: "Test Extension".to_string(),
        ..Default::default()
    };

    let models = map_songs_to_models(songs, Some(&detail));

    assert_eq!(models.len(), 2);
    assert_eq!(models[0].title, "Song 1");
    assert_eq!(models[0].extension, "test.pkg");
    assert_eq!(models[1].title, "Song 2");
    assert_eq!(models[1].extension, "test.pkg");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_entity_content_coordinator_lifecycle() {
    run_slint_test(do_test_entity_content_coordinator_lifecycle);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_test_entity_content_coordinator_lifecycle(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let coordinator = EntityContentCoordinator::<MockEntitySongProvider>::new(main_window, &sm);
    let cloned_coordinator = coordinator.clone();

    cloned_coordinator.on_show();

    let loaded = wait_until(|| {
        main_window
            .global::<AlbumContentPageProps>()
            .get_songs()
            .row_count()
            == 2
    })
    .await;

    assert!(loaded);
    assert_eq!(
        main_window
            .global::<AlbumContentPageProps>()
            .get_songs()
            .row_count(),
        2
    );

    coordinator.on_toggle_extension("test.pkg".to_string(), false);
    coordinator.on_load_more_songs();

    coordinator.on_hide();

    assert_eq!(
        main_window
            .global::<AlbumContentPageProps>()
            .get_songs()
            .row_count(),
        0
    );
}

#[rstest::rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_scope_providers_none_scope(state_manager_fixture: TestSlintSmContext) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let ext_handler = sm.get_extension_handler().await;

    let (providers, detail) = fetch_scope_providers(&ext_handler, None, "").await;

    assert!(providers.is_empty());
    assert!(detail.is_none());
}
