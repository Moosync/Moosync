use tracing_test::traced_test;

use crate::{
    ExtensionProviderItem,
    utils::{ExtensionPaginationManager, update_provider_list_enabled},
};

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
