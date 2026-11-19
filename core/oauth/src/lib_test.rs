use assertables::{assert_none, assert_some_eq_x};
use tracing_test::traced_test;

use crate::OauthManager;

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_oauth_manager_register_and_find_host() {
    let manager = OauthManager::new();

    manager.register_path("package_a".to_string(), "spotify".to_string());
    manager.register_path("package_b".to_string(), "moosync://youtube".to_string());

    assert_some_eq_x!(manager.find_package_for_host("spotify"), "package_a");
    assert_some_eq_x!(manager.find_package_for_host("youtube"), "package_b");
    assert_none!(manager.find_package_for_host("unknown"));
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_oauth_manager_register_path_fallback() {
    let manager = OauthManager::new();

    manager.register_path("pkg_fallback".to_string(), "://fallback_host/".to_string());

    assert_some_eq_x!(
        manager.find_package_for_host("fallback_host"),
        "pkg_fallback"
    );
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_oauth_manager_find_host_for_package() {
    let manager = OauthManager::new();

    manager.register_path("pkg_multi".to_string(), "first_host".to_string());
    manager.register_path("pkg_multi".to_string(), "second_host".to_string());

    let found_host = manager.find_host_for_package("pkg_multi");
    assert!(matches!(
        found_host.as_deref(),
        Some("first_host") | Some("second_host")
    ));
    assert_some_eq_x!(manager.find_package_for_host("first_host"), "pkg_multi");
    assert_some_eq_x!(manager.find_package_for_host("second_host"), "pkg_multi");
    assert_none!(manager.find_host_for_package("unknown_pkg"));
}
