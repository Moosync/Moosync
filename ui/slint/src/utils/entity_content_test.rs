use rstest::rstest;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use songs_proto::moosync::types::{InnerSong, Song};
use state_manager::StateManager;
use tracing_test::traced_test;

use crate::{
    AlbumContentPageProps, ExtensionProviderItem, MainWindow, SongModel,
    error::UiError,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
    utils::{
        EntityContentCoordinator, EntitySongProvider, ExtensionPaginationManager,
        update_provider_list_enabled,
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
    fn set_songs(main_window: &MainWindow, model: ModelRc<SongModel>) {
        main_window
            .global::<AlbumContentPageProps>()
            .set_songs(model);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_extensions(main_window: &MainWindow, extensions: ModelRc<ExtensionProviderItem>) {
        main_window
            .global::<AlbumContentPageProps>()
            .set_extension_providers(extensions);
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
    let dummy_songs = vec![SongModel::default(), SongModel::default()];
    main_window
        .global::<AlbumContentPageProps>()
        .set_songs(ModelRc::new(VecModel::from(dummy_songs)));
    let coordinator = EntityContentCoordinator::<MockEntitySongProvider>::new(&main_window, &sm);

    coordinator.on_hide();

    assert_eq!(
        main_window
            .global::<AlbumContentPageProps>()
            .get_songs()
            .row_count(),
        0
    );
}
