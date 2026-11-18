use std::{env, fs};

use assertables::assert_some;
use extensions_proto::moosync::types::{ManifestPermissions, PathPermission};
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
    let paths = vec![
        PathPermission {
            source: format!("{}/sub_dir", "{CACHE_DIR}"),
            target: "/guest/cache_sub".to_string(),
            justification: "Cache sub dir".to_string(),
        },
        PathPermission {
            source: "{MOOSYNC_TEST_PATH_VAR}".to_string(),
            target: "/guest/env_dir".to_string(),
            justification: "Env dir".to_string(),
        },
        PathPermission {
            source: "/non/existent/path/for/sure".to_string(),
            target: "/guest/invalid".to_string(),
            justification: "Invalid dir".to_string(),
        },
    ];
    let permissions = ManifestPermissions {
        paths,
        hosts: vec![],
        binaries: vec![],
    };

    let allowed = ExtismContext::get_allowed_paths(&permissions, &cache_dir);

    assert_some!(allowed.get(&existing_sub.to_string_lossy().to_string()));
    assert_some!(allowed.get(&env_sub.to_string_lossy().to_string()));
    assert_eq!(allowed.len(), 2);
}
