use std::{collections::HashMap, env, fs};

use assertables::assert_some;
use extensions_proto::moosync::types::ManifestPermissions;
use tempdir::TempDir;
use tracing_test::traced_test;

use crate::context::extism_context::ExtismContext;

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_allowed_paths() {
    let temp_dir = TempDir::new("test_allowed_paths").unwrap();
    let cache_dir = temp_dir.path().join("cache");
    let existing_sub = cache_dir.join("sub_dir");
    fs::create_dir_all(&existing_sub).unwrap();
    let env_sub = temp_dir.path().join("env_dir");
    fs::create_dir_all(&env_sub).unwrap();
    unsafe {
        env::set_var("MOOSYNC_TEST_PATH_VAR", &env_sub);
    }
    let mut paths_map = HashMap::new();
    paths_map.insert(
        format!("{}/sub_dir", "{CACHE_DIR}"),
        "/guest/cache_sub".to_string(),
    );
    paths_map.insert(
        "{MOOSYNC_TEST_PATH_VAR}".to_string(),
        "/guest/env_dir".to_string(),
    );
    paths_map.insert(
        "/non/existent/path/for/sure".to_string(),
        "/guest/invalid".to_string(),
    );
    let permissions = ManifestPermissions {
        paths: paths_map,
        hosts: vec![],
        binaries: vec![],
    };

    let allowed = ExtismContext::get_allowed_paths(&permissions, &cache_dir);

    assert_some!(allowed.get(&existing_sub.to_string_lossy().to_string()));
    assert_some!(allowed.get(&env_sub.to_string_lossy().to_string()));
    assert_eq!(allowed.len(), 2);
}
